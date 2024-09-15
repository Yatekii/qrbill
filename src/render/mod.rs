use chrono::NaiveDate;

use crate::{
    dimensions::{self as dims, Dimensions, Xy, PAYMENT, RECEIPT},
    format_amount, label, AddressExt, Group, Language, Line, QRBill, Reference, Style, StyleExt, Text, Error,
};

struct Styles {
    title:                    Style,
    heading:                  Style,
    value:                    Style,
    amount:                   Style,
    acceptance_pt:     Option<Style>,
    further_info_bold: Option<Style>,
    further_info:      Option<Style>,
}

pub mod cut;
pub mod qr;

impl Styles {
    fn new(family: &'static str, dim: &Dimensions) -> Self {
        const BOLD: Option<&str> = Some("bold");
        let font_family = Some(family);

        macro_rules! style {
            ($attr:ident, $weight:ident) => {
                Style {
                    font_size_in_pt: Some(dim.font.$attr.size.as_pt()),
                    font_family,
                    font_weight: $weight,
                }
            };
        }

        macro_rules! opt_style {
            ($attr:ident, $weight:ident) => {
                dim.font.$attr
                   .map(|font| Style {
                       font_size_in_pt: Some(font.size.as_pt()),
                       font_family,
                       font_weight: $weight,
                   })
            };
        }

        Self {
            title:                 style!(         title, BOLD),
            heading:               style!(       heading, BOLD),
            value:                 style!(         value, None),
            amount:                style!(        amount, None),
            acceptance_pt:     opt_style!( acceptance_pt, BOLD),
            further_info_bold: opt_style!(  further_info, BOLD),
            further_info:      opt_style!(  further_info, None),
        }
    }
}


pub struct Render {
    part: Part,
    dims: Dimensions,
    font: Styles,
    label: label::Labels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part { Receipt, Payment }

struct Sty<'s> {
    style: &'s Style,
    dy: dims::Font,
}

fn txt(cursor: &mut Xy, sty: &Sty, text: impl Into<String>) -> Text {
    cursor.y += sty.dy.line_spacing;
    let Xy { x, y } = cursor;
    Text::new("")
        .add(svg::node::Text::new(text))
        .set("x", *x)
        .set("y", *y)
        .style(sty.style)
}

macro_rules! sty {
    ($self:ident $name:ident) => {
        Sty { style: &$self.font.$name, dy: $self.dims.font.$name }
    };
}

macro_rules! sty_opt {
    ($self:ident $name:ident) => {
        Sty { style: &$self.font.$name.as_ref().unwrap(), dy: $self.dims.font.$name.unwrap() }
    };
}

pub enum What { OnlyReceipt, OnlyPayment, ReceiptAndPayment  }

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
        let (dims, font) = match part {
            Part::Receipt => (RECEIPT, Styles::new("Arial, Frutiger, Helvetica, Liberation Sans", &RECEIPT)),
            Part::Payment => (PAYMENT, Styles::new("Arial, Frutiger, Helvetica, Liberation Sans", &PAYMENT)),
        };
        let label = label::Labels::for_language(language);
        Self { part, dims, font, label }
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
        let Self { dims, label, part, .. } = self;
        let text = match part {
            Part::Receipt => label.receipt,
            Part::Payment => label.payment_part,
        };
        let sty = sty!(self title);
        let mut cursor = dims.section.title;
        txt(&mut cursor, &sty, text)
    }

    fn section_qr(&self, bill: &QRBill) -> Result<Group, Error> {
        match self.part {
            Part::Receipt => Ok(Group::new()),
            Part::Payment => bill.section_qr(),
        }
    }

    fn section_information(&self, bill: &QRBill) -> Group {
        let Self { dims, label, .. } = self;

        let sty_title = sty!(self heading);
        let sty_value = sty!(self value);

        let mut g = Group::new();
        let mut cursor = dims.section.information;
        macro_rules! skip_one_line { () => (g = g.add(txt(&mut cursor, &sty_value, ""))); }

        // ----- Account / Payable to ------------------------------------------
        g = g
            .add(txt(&mut cursor, &sty_title, label.payable_to))
            .add(txt(&mut cursor, &sty_value, format!("{}", bill.account)));

        for line in bill.creditor.as_paragraph(dims.max_chars_line) {
            g = g.add(txt(&mut cursor, &sty_value, line));
        }
        skip_one_line!();
        // ----- Reference -----------------------------------------------------
        if !matches!(bill.reference, Reference::None) {
            g = g.add(txt(&mut cursor, &sty_title,              label.reference))
                 .add(txt(&mut cursor, &sty_value, format!("{}", bill.reference)));
            skip_one_line!();
        }
        // ----- Additional Information ----------------------------------------
        if let (Part::Payment, Some(info)) = (self.part, &bill.extra_infos) {
            g = g.add(txt(&mut cursor, &sty_title, label.additional_information));
            // TODO cheating on additional information content: see Ustrd and StrdBkginf in spec
            for line in info.lines() {
                g = g.add(txt(&mut cursor, &sty_value, line));
            }
            skip_one_line!();
        }
        // ----- Due date ------------------------------------------------------
        // Can't find anything about due date in the standard! Is it some
        // specific kind of additional information that exists only in this
        // crate?
        if let Some(date) = bill.due_date {
            g = g.add(txt(&mut cursor, &sty_title, label.payable_by_date))
                 .add(txt(&mut cursor, &sty_value, format_date(date)));
            skip_one_line!();
        }
        // ----- Debtor --------------------------------------------------------
        if let Some(debtor) = &bill.debtor {
            g = g.add(txt(&mut cursor, &sty_title, label.payable_by));
            for line in debtor.as_paragraph(dims.max_chars_line) {
                g = g.add(txt(&mut cursor, &sty_value, line));
            }
        } else {
            g = g.add(txt(&mut cursor, &sty_title, label.payable_by_extended));
            let Xy { x: w, y: h } = dims.blank_payable;
            /*TODO fix this hack */cursor.y += dims::Length::Mm(1.5);
            let Xy { x, y } = cursor;
            g = g.add(self.blank_rect(x.as_uu(), y.as_uu(), w.as_uu(), h.as_uu()));
            cursor.y += h;
        }
        g
        // No need to skip line at end
    }

    fn section_amount(&self, bill: &QRBill) -> Group {
        let mut g = Group::new();
        let Self { dims, label, part, .. } = self;

        let sty_title = sty!(self heading);
        let sty_value = sty!(self amount);

        // Easier to have two cursors, than to adjust x value of single cursor
        let mut cursor_cur = dims.section.amount;
        let mut cursor_amt = dims.section.amount;

        use crate::dimensions::Length::Mm;

        // TODO where is x-pos of AMOUNT stated in the standard?
        cursor_amt.x += match (*part, bill.amount) {
            (Part::Receipt, None   ) => Mm(12.0),
            (Part::Receipt, Some(_)) => Mm(23.0),
            (Part::Payment, None   ) => Mm(15.0),
            (Part::Payment, Some(_)) => Mm(23.0),
        };
        g = g.add(txt(&mut cursor_cur, &sty_title, label.currency))
             .add(txt(&mut cursor_amt, &sty_title, label.amount))
             .add(txt(&mut cursor_cur, &sty_value, format!("{}", bill.currency)));
        if let Some(amount) = bill.amount {
            g = g.add(txt(&mut cursor_amt, &sty_value, format_amount(amount)));
        } else {
            if *part == Part::Receipt {
                cursor_amt = dims.section.amount;
                cursor_amt.x += Mm(22.0); // TODO where is x-pos of AMOUNT stated in the standard?
            } else {
                cursor_amt.x += Mm(-4.0); // TODO position this at the end of the QR width?
            }
            let Xy { x: w, y: h } = self.dims.blank_amount;
            /*TODO eliminate need for this hack */cursor_amt.y += dims::Length::Mm(2.0);
            let Xy { x, y } = cursor_amt;
            g = g.add(self.blank_rect(x.as_uu(), y.as_uu(), w.as_uu(), h.as_uu()));
        }
        g
    }

    fn section_acceptance_point(&self) -> Group {
        let g = Group::new();
        if self.part != Part::Receipt { return g; }
        let Self { dims, label, .. } = self;
        let sty = sty_opt!(self acceptance_pt);
        let mut cursor = dims.section.acceptance.unwrap();
        g.add(txt(&mut cursor, &sty, label.acceptance_point)
              .set("text-anchor", "end")
        )
    }

    /*TODO*/fn section_alternative_procs(&self, bill: &QRBill) -> Group {
        let g = Group::new();
        if self.part != Part::Payment { return g }
        if ! bill.alternative_processes.is_empty() {
            let Self { label, .. } = self;
            let mut cursor = self.dims.section.further_info.unwrap();
            //let bold  = sty_opt!(self further_info_bold);
            let plain = sty_opt!(self further_info);
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
        let dw = dims::blank_rectangle::LINE_LENGTH.as_uu();
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
                .set("stroke-width", format!("{}pt", dims::blank_rectangle::LINE_WIDTH.as_pt()))
                .set("stroke-linecap", "square"),
        )
    }
}


/// Format the due date according to spec.
fn format_date(date: NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}
