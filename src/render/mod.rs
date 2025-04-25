use chrono::NaiveDate;

use crate::{
    dimensions::{self as dims, Dimensions, Xy, payment, receipt},
    format_amount, label, AddressExt, Group, Language, Line, QRBill, Reference, ClassExt, Text, Error,
};

pub mod cut;
pub mod qr;

/// Render one part (receipt or payment) of a QRBill
pub struct Render {

    /// The part (receipt or payment) being rendered
    part: Part,

    /// Positions, sizes, and fonts of elements to be rendered
    dims: Dimensions,

    /// Collection of text styles for this part (receipt or payment) of the bill
    sty: Styles,

    /// The labels translated into the language of the bill being rendered
    label: label::Labels,
}

impl Render {

    pub fn bill(bill: &QRBill, which: What) -> Result<Group, Error> {
        let mut group = Group::new();
        let parts = match which {
            What::OnlyReceipt => vec![Part::Receipt],
            What::OnlyPayment => vec![Part::Payment],
            What::ReceiptAndPayment => vec![Part::Receipt, Part::Payment],
        };
        for part in parts {
            group = group.add(Self::new(part, bill.language).render_all(bill)?);
        }
        Ok(group)
    }

    pub fn new(part: Part, language: Language) -> Self {
        let (dims, classes) = match part {
            Part::Receipt => (receipt(), PartStyleClasses::receipt()),
            Part::Payment => (payment(), PartStyleClasses::payment()),
        };
        let label = label::Labels::for_language(language);
        macro_rules! sty { ($a:ident) => {                    Style { class: classes.$a,          text_size: dims.font.$a          }  }; }
        macro_rules! opt { ($a:ident) => { classes.$a.map(|_| Style { class: classes.$a.unwrap(), text_size: dims.font.$a.unwrap() } )}; }
        let sty = Styles {
            title:   sty!(title),
            heading: sty!(heading),
            value:   sty!(value),
            accept:  opt!(acceptance_pt),
        };
        Self { part, dims, sty, label }
    }

    pub fn render_all(&self, bill: &QRBill) -> Result<Group, Error> {
        Ok(Group::new()
            .add(self.section_title            (    ) )
            .add(self.section_qr               (bill)?)
            .add(self.section_information      (bill) )
            .add(self.section_amount           (bill) )
            .add(self.section_acceptance_point (    ) )
            .add(self.section_alternative_procs(bill) )
        )
    }

    fn section_title(&self) -> Text {
        let Self { dims, label, part, sty, .. } = self;
        let text = match part {
            Part::Receipt => label.receipt,
            Part::Payment => label.payment_part,
        };
        let mut cursor = dims.section.title;
        txt(&mut cursor, &sty.title, text)
    }

    fn section_qr(&self, bill: &QRBill) -> Result<Group, Error> {
        match self.part {
            Part::Receipt => Ok(Group::new()),
            Part::Payment => bill.section_qr(),
        }
    }

    fn section_information(&self, bill: &QRBill) -> Group {
        let Self { dims, label, sty, .. } = self;

        let mut g = Group::new();
        let mut cursor = dims.section.information;
        macro_rules! skip_one_line { () => (g = g.add(txt(&mut cursor, &sty.value, ""))); }

        // ----- Account / Payable to ------------------------------------------
        g = g
            .add(txt(&mut cursor, &sty.heading, label.payable_to))
            .add(txt(&mut cursor, &sty.value  , format!("{}", bill.account)));

        for line in bill.creditor.as_paragraph(dims.max_chars_line) {
            g = g.add(txt(&mut cursor, &sty.value, line));
        }
        skip_one_line!();
        // ----- Reference -----------------------------------------------------
        if !matches!(bill.reference, Reference::None) {
            g = g.add(txt(&mut cursor, &sty.heading,              label.reference))
                 .add(txt(&mut cursor, &sty.value  , format!("{}", bill.reference)));
            skip_one_line!();
        }
        // ----- Additional Information ----------------------------------------
        if let (Part::Payment, Some(info)) = (self.part, &bill.extra_infos) {
            g = g.add(txt(&mut cursor, &sty.heading, label.additional_information));
            // TODO cheating on additional information content: see Ustrd and StrdBkginf in spec
            for line in info.as_paragraph().unwrap_or_default() {
                g = g.add(txt(&mut cursor, &sty.value, line));
            }
            skip_one_line!();
        }
        // ----- Due date ------------------------------------------------------
        // Can't find anything about due date in the standard! Is it some
        // specific kind of additional information that exists only in this
        // crate?
        if let Some(date) = bill.due_date {
            g = g.add(txt(&mut cursor, &sty.heading, label.payable_by_date))
                 .add(txt(&mut cursor, &sty.value  , format_date(date)));
            skip_one_line!();
        }
        // ----- Debtor --------------------------------------------------------
        if let Some(debtor) = &bill.debtor {
            g = g.add(txt(&mut cursor, &sty.heading, label.payable_by));
            for line in debtor.as_paragraph(dims.max_chars_line) {
                g = g.add(txt(&mut cursor, &sty.value, line));
            }
        } else {
            g = g.add(txt(&mut cursor, &sty.heading, label.payable_by_extended));
            /*TODO why do we need this hack? */cursor.y += dims::Length::mm(1.5);
            let (Xy { x, y }, Xy { x: w, y: h }) = (cursor, dims.blank_payable);
            g = g.add(self.blank_rect(x.as_uu(), y.as_uu(), w.as_uu(), h.as_uu()));
        }
        g
        // No need to skip_one_line at end
    }

    fn section_amount(&self, bill: &QRBill) -> Group {
        let mut g = Group::new();
        let Self { dims, label, part, sty, .. } = self;

        // Easier to have two cursors, than to adjust x value of single cursor
        let mut cursor_cur = dims.section.amount;
        let mut cursor_amt = dims.section.amount;

        use crate::dimensions::Length;

        // TODO where is x-pos of AMOUNT stated in the standard?
        cursor_amt.x += match (*part, bill.amount) {
            (Part::Receipt, None   ) => Length::mm(12.0),
            (Part::Receipt, Some(_)) => Length::mm(23.0),
            (Part::Payment, None   ) => Length::mm(15.0),
            (Part::Payment, Some(_)) => Length::mm(23.0),
        };
        g = g.add(txt(&mut cursor_cur, &sty.heading, label.currency))
             .add(txt(&mut cursor_amt, &sty.heading, label.amount))
             .add(txt(&mut cursor_cur, &sty.value, format!("{}", bill.currency)));
        if let Some(amount) = bill.amount {
            g = g.add(txt(&mut cursor_amt, &sty.value, format_amount(amount)));
        } else {
            if *part == Part::Receipt {
                cursor_amt = dims.section.amount;
                cursor_amt.x += Length::mm(22.0); // TODO where is x-pos of AMOUNT stated in the standard?
            } else {
                cursor_amt.x += Length::mm(-4.0); // TODO position this at the end of the QR width?
            }
            let Xy { x: w, y: h } = self.dims.blank_amount;
            /*TODO eliminate need for this hack */cursor_amt.y += dims::Length::mm(2.0);
            let Xy { x, y } = cursor_amt;
            g = g.add(self.blank_rect(x.as_uu(), y.as_uu(), w.as_uu(), h.as_uu()));
        }
        g
    }

    fn section_acceptance_point(&self) -> Group {
        let g = Group::new();
        if self.part != Part::Receipt { return g; }
        let Self { dims, label, sty, .. } = self;
        let mut cursor = dims.section.acceptance.unwrap();
        g.add(txt(&mut cursor, &sty.accept.unwrap(), label.acceptance_point)
              .set("text-anchor", "end")
        )
    }

    #[allow(unused)]
    /*TODO*/fn section_alternative_procs(&self, bill: &QRBill) -> Group {
        let g = Group::new();
        if self.part != Part::Payment { return g }
        if ! bill.alternative_processes.is_empty() {
            let Self { label, .. } = self;
            let mut cursor = self.dims.section.alt_proc.unwrap();
            panic!("Alternative processes not implemented yet.");
            // g
            //     .add(txt(&mut cursor, &plain, "TODO"))
            //.add(txt(&mut cursor, &plain, "stuff"))
        } else {
        g
        }
    }

    fn blank_rect(&self, x: f64, y: f64, w: f64, h: f64) -> Group {
        let mut group = Group::new();
        macro_rules! corner {
            ($x:expr, $y:expr, $dx:expr, $dy:expr) => {
                group = self.draw_line(group, $x, $y, $x+$dx, $y    );
                group = self.draw_line(group, $x, $y, $x    , $y+$dy);
            };
        }
        let dw = dims::blank_rectangle::line_length().as_uu();
        let dh = dw;
        corner!(x  , y  ,  dw,  dh); // top left
        corner!(x+w, y  , -dw,  dh); // top right
        corner!(x  , y+h,  dw, -dh); // bottom left
        corner!(x+w, y+h, -dw, -dh); // bottom right
        group
    }

    // Draws a single solid black line.
    fn draw_line(&self, group: Group, x1: f64, y1: f64, x2: f64, y2: f64) -> Group {
        group.add(
            Line::new()
                .set("x1", x1)
                .set("y1", y1)
                .set("x2", x2)
                .set("y2", y2)
                .set("stroke", "black")
                .set("stroke-width", format!("{}pt", dims::blank_rectangle::line_width().as_pt()))
                .set("stroke-linecap", "square"),
        )
    }
}

/// The CSS class names representing the styles of the elements in one part
/// (receipt or payment) being rendered
struct PartStyleClasses {
    title:                &'static str,
    heading:              &'static str,
    value:                &'static str,
    acceptance_pt: Option<&'static str>,
    alt_proc_bold: Option<&'static str>,
    alt_proc:      Option<&'static str>,
}

impl PartStyleClasses {

    /// Construct the styles for the receipt part of the bill
    fn receipt() -> Self { Self {
        title:              "r-title",
        heading:            "r-heading",
        value:              "r-value",
        acceptance_pt: Some("r-acceptance-pt"),
        alt_proc:      None,
        alt_proc_bold: None,
    }}

    /// Construct the styles for the payment part of the bill
    fn payment() -> Self { Self {
        title:              "p-title",
        heading:            "p-heading",
        value:              "p-value",
        acceptance_pt: None,
        alt_proc:      None, // TODO implement alternative processes
        alt_proc_bold: None, // TODO implement alternative processes
    }}

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part { Receipt, Payment }

#[derive(Debug, Clone, Copy)]
struct Style {
    class: &'static str,
    text_size: dims::Font,
}

/// Styles for rendering text in one part (receipt or payment) of QRBill
struct Styles {
    title:          Style,
    heading:        Style,
    value:          Style,
    accept:  Option<Style>,
    // TODO alternatie processes
}

/// Which parts of the QRBill should be rendered
pub enum What { OnlyReceipt, OnlyPayment, ReceiptAndPayment  }

/// Render some `text` at the position indicated by `cursor`, with the given
/// `style`. Advance the cursor downwards by `style`'s line spacing *before*
/// rendering the text.
fn txt(cursor: &mut Xy, style: &Style, text: impl Into<String>) -> Text {
    cursor.y += style.text_size.line_spacing;
    let Xy { x, y } = cursor;
    Text::new("")
        .add(svg::node::Text::new(text))
        .set("x", *x)
        .set("y", *y)
        .class(style.class)
}

/// Format the due date according to spec.
fn format_date(date: NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}
