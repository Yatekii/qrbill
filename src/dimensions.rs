// Dimensions taken from
//
// https://www.six-group.com/dam/download/banking-services/standardization/qr-bill/style-guide-qr-bill-en.pdf
//
// which is stored locally in
//
//   qr-standard-docs/style-guide-qr-bill-en.pdf
//
// The dimensions of the blank rectangles are on page 7, the other dimensions on
// page 15.



// 3.4 // Fonts and font sizes
//
// Only the sans-serif fonts Arial, Frutiger, Helvetica and Liberation Sans are
// permitted in black. Text must not be in italics nor underlined.
//
// The font size for headings and their associated values on the payment part
// must be at least 6 pt, and maximum 10 pt. Headings in the "Amount" and
// "Details" sections must always be the same size. They should be printed in
// bold and 2 pt smaller than the font size for their associated values. The
// recommended font size for headings is 8 pt and for the associated values 10
// pt. The only exception, in font size 11 pt (bold), is the title "Payment
// part".
//
// When filling in the "Alternative procedures" element, the font size is 7 pt,
// with the name of the alternative procedure printed in bold type.
//
// The "Ultimate creditor" element is intended for use in the future but will
// not be used for the QR-bill and should therefore not be filled in. If
// approval is given for the field to be filled in, the font size is expected to
// be 7 pt with the designation in bold type.
//
// The font sizes for the receipt are 6 pt for the headings (bold) and 8 pt for
// the associated values. The exception, in font size 11 pt (bold), is the title
// "Receipt".



// TODO replace this with Length(f64), but then the mm/pt constructors become
// non-const functions and the we cannot make the RECEIPT/PAYMENT consts
#[derive(Debug, Copy, Clone)]
pub enum Length {
    Mm(f64),
    Pt(f64),
}

impl Length {

    pub (crate) fn as_mm(self) -> f64 {
        match self {
            Mm(mm) => mm,
            Pt(_ ) => todo!(),
        }
    }

    pub (crate) fn as_pt(self) -> f64 {
        match self {
            Mm(_ ) => todo!(),
            Pt(pt) => pt,
        }
    }

    pub (crate) fn as_uu(self) -> f64 {
        match self {
            Mm(mm) => mm * MM_TO_UU,
            Pt(pt) => pt * PT_TO_UU,
        }
    }

}

impl From<Length> for svg::node::Value {
    fn from(value: Length) -> Self {
        match value {
            Mm(mm) => format!("{:.1}", mm * MM_TO_UU),
            Pt(pt) => format!("{:.1}", pt * 666.0)
        }.into()
    }
}

const PT_TO_MM: f64 = 0.3527777778;

// Todo, need to rethink the approach to storing mm and pt
impl std::ops::AddAssign for Length {
    fn add_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Mm(a), Mm(b)) => Mm(*a + b),
            (Mm(m), Pt(p)) => Mm(*m + p * PT_TO_MM),
            (Pt(_), Mm(_)) => todo!(),
            (Pt(a), Pt(b)) => Pt(*a + b),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Xy { pub x: Length, pub y: Length }

impl Xy {
    pub (crate) const fn mm(left: f64, top: f64) -> Self {
        Self { x: Mm(left), y: Mm(top) }
    }
}

pub struct Dimensions {
    pub section: Sections,
    pub font: Fonts,
    // Dimensions of blank rectangles
    pub blank_payable:  Xy,
    pub blank_amount:   Xy,
    pub max_chars_line: usize,
}

use Length::*;

const RCT_X: f64 =   5.0; // mm x-position of RECEIPT part sections
const PAY_X: f64 =  67.0; // mm x-position of PAYMENT part sections except INFORMATION
const INF_X: f64 = 118.0; // mm x-position of INFORMATION section in PAYMENT part
const ACC_E: f64 =  57.0; // mm x-position of RHS of ACCEPTANCE POINT section

pub const RECEIPT: Dimensions = Dimensions {
    section: Sections {
        title:             Xy::mm(RCT_X,  5.0),
        information:       Xy::mm(RCT_X, 12.0),
        amount:            Xy::mm(RCT_X, 68.0),
        acceptance:   Some(Xy::mm(ACC_E, 82.0)),
        qr_code:      None,
        further_info: None,
    },

    // The font sizes for the receipt are 6 pt for the headings (bold) and 8 pt
    // for the associated values. The exception, in font size 11 pt (bold), is
    // the title "Receipt".
    font: Fonts {           //    size  line-spacing
        title:              font( 11.0, 11.0), // bold
        heading:            font(  6.0,  9.0), // bold
        value:              font(  8.0,  9.0),
        amount:             font(  8.0, 11.0),
        acceptance_pt: Some(font(  6.0,  8.0)), // bold
        further_info:  None,
    },

    blank_payable: Xy::mm( 52.0, 20.0),
    blank_amount:  Xy::mm( 30.0, 10.0),

    max_chars_line: 38,
};

pub const PAYMENT: Dimensions = Dimensions {
    section: Sections {
        title:             Xy::mm(PAY_X,  5.0),
        information:       Xy::mm(INF_X,  5.0),
        amount:            Xy::mm(PAY_X, 68.0),
        acceptance:   None,
        qr_code:      Some(Xy::mm(PAY_X, 17.0)),
        further_info: Some(Xy::mm(PAY_X, 90.0)),
    },

    // The font size for headings and their associated values on the payment
    // part must be at least 6 pt, and maximum 10 pt.
    //
    // Headings in the "Amount" and "Details" sections must always be the same
    // size. They should be printed in bold and 2 pt smaller than the font size
    // for their associated values.
    //
    // The recommended font size for headings is 8 pt and for the associated
    // values 10 pt.
    //
    // The only exception, in font size 11 pt (bold), is the title "Payment
    // part".
    //
    // When filling in the "Alternative procedures" element, the font size is 7
    // pt, with the name of the alternative procedure printed in bold type.
    font: Fonts {           //    size  line-spacing
        title:              font( 11.0, 11.0), // bold
        heading:            font(  8.0, 11.0), // bold
        value:              font( 10.0, 11.0),
        amount:             font( 10.0, 13.0),
        acceptance_pt: None,
        further_info:  Some(font(  7.0,  8.0)), // bold & normal
    },

    blank_payable: Xy::mm( 65.0, 25.0),
    blank_amount:  Xy::mm( 40.0, 15.0),

    max_chars_line: 72,
};

pub struct Sections {
    pub title:               Xy,
    pub information:         Xy,
    pub amount:              Xy,
    pub acceptance:   Option<Xy>,
    pub qr_code:      Option<Xy>,
    pub further_info: Option<Xy>,
}

pub struct Fonts {
    pub title:                Font,
    pub heading:              Font,
    pub value:                Font,
    pub amount:               Font,
    pub acceptance_pt: Option<Font>,
    pub further_info:  Option<Font>,
}

#[derive(Debug, Clone, Copy)]
pub struct Font { pub (crate) size: Length, pub (crate) line_spacing: Length }

const fn font(size_in_pt: f64, line_spacing_in_pt: f64) -> Font {
    Font {
        size: Pt(size_in_pt),
        line_spacing: Pt(line_spacing_in_pt),
    }
}

pub mod blank_rectangle {
    use super::*;
    pub const LINE_LENGTH: Length = Mm(3.0);
    pub const LINE_WIDTH:  Length = Pt(0.75);
    
}

pub const MM_TO_UU: f64 = 3.543307;
pub const PT_TO_UU: f64 = PT_TO_MM * MM_TO_UU;
