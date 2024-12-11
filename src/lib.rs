use chrono::NaiveDate;
pub use iban::Iban;
use iban::IbanLike;
use isocountry::CountryCode;
use qrcode::{self, types::QrError, QrCode};
use svg::{
    node::element::{Group, Line, Path, Polygon, Rectangle, Text},
    Document,
};
use thousands::Separable;

pub mod esr;
pub mod iso11649;
mod dimensions;
mod label;
pub mod render;

pub use label::Language;

const IBAN_ALLOWED_COUNTRIES: [&str; 2] = ["CH", "LI"];
const QR_IID_START: usize = 30000;
const QR_IID_END: usize = 31999;

use dimensions::MM_TO_UU;
const BILL_HEIGHT_IN_MM: f64 = 105.0;
const BILL_HEIGHT: f64 = BILL_HEIGHT_IN_MM * MM_TO_UU;
const RECEIPT_WIDTH: f64 = 62.0 * MM_TO_UU; // mm
const A4_WIDTH_IN_MM: f64 = 210.0;
const A4_WIDTH: f64 = A4_WIDTH_IN_MM * MM_TO_UU;
const A4_HEIGHT_IN_MM: f64 = 297.0;
const A4_HEIGHT: f64 = A4_HEIGHT_IN_MM * MM_TO_UU;

trait AddressExt {
    fn data_list(&self) -> Vec<String>;

    fn as_paragraph(&self, max_width: usize) -> Vec<String>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An address line should have between 1 and 70 characters.")]
    Line,
    #[error("An address name should have between 1 and 70 characters.")]
    Name,
    #[error("A street should have between 1 and 70 characters.")]
    Street,
    #[error("A postal code should have between 1 and 16 characters.")]
    PostalCode,
    #[error("A house number should have between 1 and 16 characters.")]
    HouseNumber,
    #[error("A city should have between 1 and 35 characters.")]
    City,
    #[error("The IBAN needs to start with CH or LI.")]
    InvalidIban,
    #[error("Extra infos can be no more than 140 characters.")]
    ExtraInfos,
    #[error(
        "At maximum two alternative procedure with a maximum of 100 characters can be specified."
    )]
    AlternativeProcedure,
    #[error("An error with the QR code generation occured.")]
    Qr(#[from] QrError),
    #[error("An IO error occured.")]
    Io(#[from] std::io::Error),
    #[error("An error occurred when generating PDF")]
    Pdf(#[from] svg2pdf::usvg::Error),
}

pub enum Address {
    Cobined(CombinedAddress),
    Structured(StructuredAddress),
}

impl AddressExt for Address {
    fn data_list(&self) -> Vec<String> {
        match self {
            Address::Cobined(a) => a.data_list(),
            Address::Structured(a) => a.data_list(),
        }
    }

    fn as_paragraph(&self, max_width: usize) -> Vec<String> {
        match self {
            Address::Cobined(a) => a.as_paragraph(max_width),
            Address::Structured(a) => a.as_paragraph(max_width),
        }
    }
}

pub struct CombinedAddress {
    name: String,
    line1: String,
    line2: String,
    country: CountryCode,
}

impl CombinedAddress {
    pub fn new(
        name: String,
        line1: String,
        line2: String,
        country: CountryCode,
    ) -> Result<Self, Error> {
        if line1.len() > 70 || line2.len() > 70 {
            return Err(Error::Line);
        }
        Ok(Self {
            name,
            line1,
            line2,
            country,
        })
    }
}

impl AddressExt for CombinedAddress {
    fn data_list(&self) -> Vec<String> {
        vec![
            "K".into(),
            self.name.clone(),
            self.line1.clone(),
            self.line2.clone(),
            "".into(),
            "".into(),
            self.country.alpha2().to_string(),
        ]
    }

    fn as_paragraph(&self, max_width: usize) -> Vec<String> {
        [self.name.clone(), self.line1.clone(), self.line2.clone()]
            .iter()
            .map(|line| textwrap::fill(line, max_width))
            .collect()
    }
}

pub struct StructuredAddress {
    pub name: String,
    pub street: String,
    pub house_number: String,
    pub postal_code: String,
    pub city: String,
    pub country: CountryCode,
}

impl StructuredAddress {
    pub fn new(
        name: String,
        street: String,
        house_number: String,
        postal_code: String,
        city: String,
        country: CountryCode,
    ) -> Result<Self, Error> {
        if name.len() > 70 {
            return Err(Error::Name);
        }
        if street.len() > 70 {
            return Err(Error::Street);
        }
        if house_number.len() > 16 {
            return Err(Error::HouseNumber);
        }
        if postal_code.len() > 16 {
            return Err(Error::PostalCode);
        }
        if city.len() > 35 {
            return Err(Error::City);
        }

        Ok(Self {
            name,
            street,
            house_number,
            postal_code,
            city,
            country,
        })
    }
}

impl AddressExt for StructuredAddress {
    fn data_list(&self) -> Vec<String> {
        vec![
            "S".into(),
            self.name.clone(),
            self.street.clone(),
            self.house_number.clone(),
            self.postal_code.clone(),
            self.city.clone(),
            self.country.alpha2().to_string(),
        ]
    }

    fn as_paragraph(&self, max_width: usize) -> Vec<String> {
        let maybe_prefix = if self.country == CountryCode::CHE {
            "".to_string() } else {
            format!("{}-", self.country.alpha2().to_owned())
        };
        vec![
            self.name.clone(),
            format!("{} {}", self.street, self.house_number),
            format!(
                "{maybe_prefix}{} {}",
                self.postal_code,
                self.city,
            ),
        ]
        .into_iter()
        .map(|line| textwrap::fill(&line, max_width))
        .collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Currency {
    SwissFranc,
    Euro,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Currency::SwissFranc => "CHF".to_string(),
            Currency::Euro => "EUR".to_string(),
        })
    }
}

pub struct QRBill {
    account: Iban,
    creditor: Address,
    amount: Option<f64>,
    currency: Currency,
    due_date: Option<NaiveDate>,
    debtor: Option<Address>,
    reference: Reference,
    /// Extra information aimed for the bill recipient.
    pub extra_infos: Option<String>,
    /// Two additional fields for alternative payment schemes.
    alternative_processes: Vec<String>,
    /// Language of the output.
    language: Language,
    /// Print a horizontal line at the top of the bill.
    line_top: bool,
    /// Print a vertical line between the receipt and the bill itself.
    line_mid: bool,
}

pub struct QRBillOptions {
    pub account: Iban,
    pub creditor: Address,
    pub amount: Option<f64>,
    pub currency: Currency,
    pub due_date: Option<NaiveDate>,
    pub debtor: Option<Address>,
    pub reference: Reference,
    /// Extra information aimed for the bill recipient.
    pub extra_infos: Option<String>,
    /// Two additional fields for alternative payment schemes.
    pub alternative_processes: Vec<String>,
    /// Language of the output.
    pub language: Language,
    /// Print a horizontal line at the top of the bill.
    pub top_line: bool,
    /// Print a vertical line between the receipt and the bill itself.
    pub payment_line: bool,
}

#[derive(Debug, Clone)]
pub enum Reference {
    Qrr(esr::Esr),
    Scor(iso11649::Iso11649),
    None,
}

impl Reference {
    fn data_list(&self) -> Vec<String> {
        match self {
            Reference::Qrr(esr) => vec!["QRR".to_string(), esr.to_raw()],
            Reference::Scor(scor) => vec!["SCOR".to_string(), scor.with_checksum()],
            Reference::None => vec!["NON".to_string(), "".to_string()],
        }
    }
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Reference::Qrr(esr) => esr.to_string(),
            Reference::Scor(reference) => chunked(&reference.with_checksum()),
            Reference::None => String::new(),
        })
    }
}

trait ClassExt {
    fn class(self, class: &str) -> Text;
}

impl ClassExt for Text {
    fn class(self, class: &str) -> Text {
        self.set("class", class)
    }
}

impl QRBill {
    const QR_TYPE: &'static str = "SPC";
    const VERSION: &'static str = "0200";
    const CODING: usize = 1;

    /// Creates a new QR-Bill which can be rendered onto an SVG.
    pub fn new(options: QRBillOptions) -> Result<Self, Error> {
        if !IBAN_ALLOWED_COUNTRIES.contains(&options.account.country_code()) {
            return Err(Error::InvalidIban);
        }
        let iban_iid = options.account.electronic_str()[4..9]
            .parse()
            .expect("This is a bug. Please report it.");
        let _account_is_qriban = (QR_IID_START..=QR_IID_END).contains(&iban_iid);

        // TODO validate ESR reference number

        // TODO: validate QR IBAN / QRID matches.

        if let Some(extra_infos) = options.extra_infos.as_ref() {
            if extra_infos.len() > 120 {
                return Err(Error::ExtraInfos);
            }
        }

        if options.alternative_processes.len() > 2 {
            return Err(Error::AlternativeProcedure);
        }
        if options.alternative_processes.iter().any(|v| v.len() > 100) {
            return Err(Error::AlternativeProcedure);
        }

        Ok(Self {
            account: options.account,
            creditor: options.creditor,
            amount: options.amount,
            currency: options.currency,
            due_date: options.due_date,
            debtor: options.debtor,
            reference: options.reference,
            extra_infos: options.extra_infos,
            alternative_processes: options.alternative_processes,
            language: options.language,
            line_top: options.top_line,
            line_mid: options.payment_line,
        })
    }

    /// Return data to be encoded in the QR code in the standard text representation of a list of strings.
    pub fn qr_data(&self) -> String {
        let mut data = vec![
            Self::QR_TYPE.to_string(),
            Self::VERSION.to_string(),
            Self::CODING.to_string(),
            self.account.electronic_str().to_string(),
        ];
        data.extend(self.creditor.data_list());
        data.extend(vec!["".into(); 7]);
        data.extend(vec![
            self.amount.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            self.currency.to_string(),
        ]);
        data.extend(
            self.debtor
                .as_ref()
                .map(|v| v.data_list())
                .unwrap_or_else(|| vec!["".into(); 7]),
        );
        data.extend(self.reference.data_list());
        data.extend(vec![self.extra_infos.clone().unwrap_or_default()]);
        data.push("EPD".to_string());
        data.extend(self.alternative_processes.clone());

        data.join("\n")
    }

    /// Writes the represented QR-Bill into an SVG file.
    ///
    /// * `full_page`: Makes the generated SVG the size of a full A4 page.
    pub fn write_svg_to_file(
        &self,
        path: impl AsRef<std::path::Path>,
        full_page: bool,
    ) -> Result<(), Error> {
        let svg = self.create_svg(full_page)?;

        std::fs::write(path, svg)?;

        Ok(())
    }

    /// Writes the represented QR-Bill into a PDF file.
    ///
    /// * `full_page`: Makes the generated SVG the size of a full A4 page.
    pub fn write_pdf_to_file(
        &self,
        path: impl AsRef<std::path::Path>,
        full_page: bool,
    ) -> Result<(), Error> {
        let svg = self.create_svg(full_page)?;
        let mut options = svg2pdf::usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        let tree = svg2pdf::usvg::Tree::from_str(&svg, &options)?;

        let pdf = svg2pdf::to_pdf(&tree, svg2pdf::ConversionOptions::default(), svg2pdf::PageOptions::default());
        std::fs::write(path, pdf)?;
        Ok(())
    }

    /// Returns a string containing the SVG representing the QR-Bill
    ///
    /// * `full_page`: Makes the generated SVG the size of a full A4 page.
    pub fn create_svg(&self, full_page: bool) -> Result<String, Error> {
        // Make a properly sized document with a correct viewbox.
        let (h_in_mm, h) = if full_page { (  A4_HEIGHT_IN_MM,   A4_HEIGHT) }
        else                            { (BILL_HEIGHT_IN_MM, BILL_HEIGHT) };
        let document = Document::new()
            .add(svg::node::element::Style::new(crate::dimensions::make_svg_styles()))
            .set("width", format!("{A4_WIDTH_IN_MM}mm"))
            .set("height", format!("{h_in_mm}mm"))
            .set("viewBox", format!("0 0 {A4_WIDTH} {h}"));

        // White background.
        let mut document = document.add(
            Rectangle::new()
                .set("x", 0.0)
                .set("y", 0.0)
                .set("width", "100%")
                .set("height", "100%")
                .set("fill", "white"),
        );

        let mut bill_group = self.draw_bill()?;

        if full_page {
            bill_group = self.transform_to_full_page(bill_group);
        }

        document = document.add(bill_group);

        Ok(document.to_string())
    }

    /// Renders to an A4 page, adding the bill in a group element.
    ///
    /// Also adds a note about separating the bill.
    fn transform_to_full_page(&self, group: Group) -> Group {
        let y_offset = A4_HEIGHT - BILL_HEIGHT;
        group.set("transform", format!("translate(0, {})", y_offset))
    }

    /// Draws the entire QR bill SVG image.
    fn draw_bill(&self) -> Result<Group, Error> {
        let mut group = Group::new();

        if self.line_top { group = group.add(self.line_top_scissor()?); }
        if self.line_mid { group = group.add(self.line_mid_scissor()?); }

        use render::{Render, What};
        Ok(group.add(Render::bill(self, What::ReceiptAndPayment)?))
    }

}

/// Converts a millimeter based value into a SVG screen units value.
/// This should be used to always do math in sceen units even if we have numbers in mm.
fn mm(value: f64) -> f64 {
    value * MM_TO_UU
}

/// Formats the amount according to spec.
fn format_amount(amount: f64) -> String {
    format!("{:.2}", amount).separate_with_spaces()
}

// def wrap_infos(infos) {
//     for text in infos:
//         while(text) {
//             yield text[:MAX_CHARS_PAYMENT_LINE]
//             text = text[MAX_CHARS_PAYMENT_LINE:]


pub fn chunked(unchunked: &str) -> String {
    unchunked
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join(" ")
}
