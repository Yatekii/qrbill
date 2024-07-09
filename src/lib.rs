pub mod billing_infos;
use billing_infos::BillingInfos;

pub mod esr;
pub mod iso11649;

use std::fmt::{Display, Formatter, Write};

pub use chrono::NaiveDate;
pub use iban::Iban;
pub use iban::IbanLike;
pub use isocountry::CountryCode;
use qrcode::{render, types::QrError, QrCode};
use regex::Regex;
use svg::{
    node::{
        element::{Group, Line, Path, Polygon, Rectangle, Text},
        Value,
    },
    Document,
};
use thousands::Separable;

const IBAN_ALLOWED_COUNTRIES: [&str; 2] = ["CH", "LI"];
const QR_IID_START: usize = 30000;
const QR_IID_END: usize = 31999;

const MM_TO_UU: f64 = 3.543307;
const BILL_HEIGHT_IN_MM: f64 = 105.0;
const BILL_HEIGHT: f64 = BILL_HEIGHT_IN_MM * MM_TO_UU;
const RECEIPT_WIDTH: f64 = 62.0 * MM_TO_UU; // mm
const MAX_CHARS_PAYMENT_LINE: usize = 72;
const MAX_CHARS_RECEIPT_LINE: usize = 38;
const A4_WIDTH_IN_MM: f64 = 210.0;
const A4_WIDTH: f64 = A4_WIDTH_IN_MM * MM_TO_UU;
const A4_HEIGHT_IN_MM: f64 = 297.0;
const A4_HEIGHT: f64 = A4_HEIGHT_IN_MM * MM_TO_UU;

// Annex D: Multilingual headings
const LABEL_PAYMENT_PART: Translation = Translation {
    en: "Payment part",
    de: "Zahlteil",
    fr: "Section paiement",
    it: "Sezione pagamento",
};

const LABEL_PAYABLE_TO: Translation = Translation {
    en: "Account / Payable to",
    de: "Konto / Zahlbar an",
    fr: "Compte / Payable à",
    it: "Conto / Pagabile a",
};

const LABEL_REFERENCE: Translation = Translation {
    en: "Reference",
    de: "Referenz",
    fr: "Référence",
    it: "Riferimento",
};

const LABEL_ADDITIONAL_INFORMATION: Translation = Translation {
    en: "Additional information",
    de: "Zusätzliche Informationen",
    fr: "Informations supplémentaires",
    it: "Informazioni supplementari",
};

const LABEL_CURRENCY: Translation = Translation {
    en: "Currency",
    de: "Währung",
    fr: "Monnaie",
    it: "Valuta",
};

const LABEL_AMOUNT: Translation = Translation {
    en: "Amount",
    de: "Betrag",
    fr: "Montant",
    it: "Importo",
};

const LABEL_RECEIPT: Translation = Translation {
    en: "Receipt",
    de: "Empfangsschein",
    fr: "Récépissé",
    it: "Ricevuta",
};

const LABEL_ACCEPTANCE_POINT: Translation = Translation {
    en: "Acceptance point",
    de: "Annahmestelle",
    fr: "Point de dépôt",
    it: "Punto di accettazione",
};

const LABEL_PAYABLE_BY: Translation = Translation {
    en: "Payable by",
    de: "Zahlbar durch",
    fr: "Payable par",
    it: "Pagabile da",
};

const LABEL_PAYABLE_BY_EXTENDED: Translation = Translation {
    en: "Payable by (name/address)",
    de: "Zahlbar durch (Name/Adresse)",
    fr: "Payable par (nom/adresse)",
    it: "Pagabile da (nome/indirizzo)",
};

// The extra ending space allows to differentiate from the other: "Payable by" above.
const LABEL_PAYABLE_BY_DATE: Translation = Translation {
    en: "Payable by",
    de: "Zahlbar bis",
    fr: "Payable jusqu’au",
    it: "Pagabile fino al",
};

struct Translation {
    en: &'static str,
    de: &'static str,
    fr: &'static str,
    it: &'static str,
}

const SCISSORS_SVG_PATH: &str = "m 0.764814,4.283977 c 0.337358,0.143009 0.862476,-0.115279 0.775145,-0.523225 -0.145918,-0.497473 
    -0.970289,-0.497475 -1.116209,-2e-6 -0.0636,0.23988 0.128719,0.447618 0.341064,0.523227 z m 3.875732,-1.917196 
    c 1.069702,0.434082 2.139405,0.868164 3.209107,1.302246 -0.295734,0.396158 -0.866482,0.368049 -1.293405,0.239509 
    -0.876475,-0.260334 -1.71099,-0.639564 -2.563602,-0.966653 -0.132426,-0.04295 -0.265139,-0.124595 
    -0.397393,-0.144327 -0.549814,0.22297 -1.09134,0.477143 -1.667719,0.62213 -0.07324,0.232838 0.150307,0.589809 
    -0.07687,0.842328 -0.311347,0.532157 -1.113542,0.624698 -1.561273,0.213165 -0.384914,-0.301216 
    -0.379442,-0.940948 7e-6,-1.245402 0.216628,-0.191603 0.506973,-0.286636 0.794095,-0.258382 0.496639,0.01219 
    1.013014,-0.04849 1.453829,-0.289388 0.437126,-0.238777 0.07006,-0.726966 -0.300853,-0.765416 
    -0.420775,-0.157424 -0.870816,-0.155853 -1.312747,-0.158623 -0.527075,-0.0016 -1.039244,-0.509731 
    -0.904342,-1.051293 0.137956,-0.620793 0.952738,-0.891064 1.47649,-0.573851 0.371484,0.188118 
    0.594679,0.675747 0.390321,1.062196 0.09829,0.262762 0.586716,0.204086 0.826177,0.378204 0.301582,0.119237 
    0.600056,0.246109 0.899816,0.36981 0.89919,-0.349142 1.785653,-0.732692 2.698347,-1.045565 0.459138,-0.152333 
    1.033472,-0.283325 1.442046,0.05643 0.217451,0.135635 -0.06954,0.160294 -0.174725,0.220936 -0.979101,0.397316 
    -1.958202,0.794633 -2.937303,1.19195 z m -3.44165,-1.917196 c -0.338434,-0.14399 -0.861225,0.116943 
    -0.775146,0.524517 0.143274,0.477916 0.915235,0.499056 1.10329,0.04328 0.09674,-0.247849 -0.09989,-0.490324 
    -0.328144,-0.567796 z";

trait AddressExt {
    fn data_list(&self) -> Vec<String>;

    fn as_paragraph(&self, max_width: usize) -> Vec<String>;
}
#[derive(Debug)]
enum IbanType {
    Qriid,
    Iid,
}
impl IbanType {
    fn try_valid_reference(&self, reference: &Reference, iban_str: &str) -> Result<(), Error> {
        match self {
            Self::Qriid => match reference {
                Reference::Qrr(_) => Ok(()),
                _ => Err(Error::InvalidQriid(iban_str.into())),
            },
            Self::Iid => match reference {
                Reference::Qrr(_) => Err(Error::InvalidIid(iban_str.into())),
                _ => Ok(()),
            },
        }
    }
}
trait IbanKind {
    fn iban_kind<'a>(&self) -> Result<&'a IbanType, Error>;
}
impl IbanKind for Iban {
    fn iban_kind<'a>(&self) -> Result<&'a IbanType, Error> {
        let iid: usize = self.electronic_str()[4..9]
            .parse()
            .expect("This is a bug, please report it");
        if (QR_IID_START..=QR_IID_END).contains(&iid) {
            Ok(&IbanType::Qriid)
        } else {
            Ok(&IbanType::Iid)
        }
    }
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
    #[error("The ESR reference is missing and is mandatory for QRIID IbanType")]
    EsrMandatory,
    #[error("IBAN provided ({0:?}) is not SCOR compatible (see IID)")]
    InvalidIid(String),
    #[error("IBAN provided ({0:?}) is not ESR compatible (see QRIID)")]
    InvalidQriid(String),
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
    #[error("An ESR Reference error occured")]
    Esr(#[from] esr::Error),
    #[error("An QR Creditor Reference error occured")]
    Scor(#[from] iso11649::Error),
    #[error("An error occurred when generating PDF")]
    Pdf(#[from] svg2pdf::usvg::Error),
    #[error("An error occured on the (un)structured Banking Infos: {0}")]
    BillingInfos(#[from] billing_infos::BillingInfoError),
}

#[derive(Debug)]
pub enum Address {
    Combined(CombinedAddress),
    Structured(StructuredAddress),
}

impl AddressExt for Address {
    fn data_list(&self) -> Vec<String> {
        match self {
            Address::Combined(a) => a.data_list(),
            Address::Structured(a) => a.data_list(),
        }
    }

    fn as_paragraph(&self, max_width: usize) -> Vec<String> {
        match self {
            Address::Combined(a) => a.as_paragraph(max_width),
            Address::Structured(a) => a.as_paragraph(max_width),
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct StructuredAddress {
    name: String,
    street: String,
    house_number: String,
    postal_code: String,
    city: String,
    country: CountryCode,
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
        vec![
            self.name.clone(),
            format!("{} {}", self.street, self.house_number),
            format!(
                "{}-{} {}",
                self.country.alpha2(),
                self.postal_code,
                self.city
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

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Currency::Euro => "EUR".to_string(),
            Currency::SwissFranc => "CHF".to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Language {
    German,
    English,
    French,
    Italian,
}

#[derive(Debug)]
pub struct QRBill {
    account: Iban,
    creditor: Address,
    amount: Option<f64>,
    currency: Currency,
    due_date: Option<NaiveDate>,
    debtor: Option<Address>,
    reference: Reference,
    /// Extra banking information aimed for the bill recipient.
    extra_infos: Option<BillingInfos>,
    /// Two additional fields for alternative payment schemes.
    alternative_processes: Vec<String>,
    /// Language of the output.
    language: Language,
    /// Print a horizontal line at the top of the bill.
    top_line: bool,
    /// Print a vertical line between the receipt and the bill itself.
    payment_line: bool,
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
    pub extra_infos: Option<BillingInfos>,
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
            Reference::Scor(scor) => vec!["SCOR".to_string(), scor.to_raw()],
            Reference::None => vec!["NON".to_string(), "".to_string()],
        }
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Reference::Qrr(esr) => esr.to_string(),
            Reference::Scor(reference) => reference.to_string(),
            Reference::None => String::new(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
struct Style {
    font_size: Option<f64>,
    font_family: Option<&'static str>,
    font_weight: Option<&'static str>,
}

impl From<Style> for Value {
    fn from(style: Style) -> Self {
        let mut s = String::new();
        if let Some(font_size) = style.font_size {
            write!(&mut s, "font-size: {};", font_size).expect("This is a bug. Please report it.");
        }
        if let Some(font_family) = style.font_family {
            write!(&mut s, "font-family: {};", font_family)
                .expect("This is a bug. Please report it.");
        }
        if let Some(font_weight) = style.font_weight {
            write!(&mut s, "font-weight: {};", font_weight)
                .expect("This is a bug. Please report it.");
        }

        s.into()
    }
}

trait StyleExt {
    fn style(self, style: Style) -> Text;
}

impl StyleExt for Text {
    fn style(mut self, style: Style) -> Text {
        if let Some(font_size) = style.font_size {
            self = self.set("font-size", font_size);
        }
        if let Some(font_family) = style.font_family {
            self = self.set("font-family", font_family);
        }
        if let Some(font_weight) = style.font_weight {
            self = self.set("font-weight", font_weight);
        }

        self
    }
}

impl QRBill {
    const QR_TYPE: &'static str = "SPC";
    const VERSION: &'static str = "0200";
    const CODING: usize = 1;

    const TITLE_FONT: Style = Style {
        font_size: Some(12.0),
        font_family: Some("Helvetica"),
        font_weight: Some("bold"),
    };

    const FONT: Style = Style {
        font_size: Some(10.0),
        font_family: Some("Helvetica"),
        font_weight: None,
    };

    const HEAD_FONT: Style = Style {
        font_size: Some(8.0),
        font_family: Some("Helvetica"),
        font_weight: Some("bold"),
    };

    const PROCESS_FONT: Style = Style {
        font_size: Some(7.0),
        font_family: Some("Helvetica"),
        font_weight: None,
    };

    /// Creates a new QR-Bill which can be rendered onto an SVG.
    pub fn new(options: QRBillOptions) -> Result<Self, Error> {
        if !IBAN_ALLOWED_COUNTRIES.contains(&options.account.country_code()) {
            return Err(Error::InvalidIban);
        }

        let iban_kind = options.account.iban_kind()?;
        iban_kind.try_valid_reference(&options.reference, options.account.electronic_str())?;

        if let Some(extra_infos) = options.extra_infos.as_ref() {
            if extra_infos.len() > 140 {
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
            top_line: options.top_line,
            payment_line: options.payment_line,
        })
    }

    /// Return data to be encoded in the QR code in the standard text representation of a list of strings.
    fn qr_data(&self) -> String {
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
        data.extend(vec![self
            .extra_infos
            .clone()
            .and_then(|x| x.unstructured())
            .unwrap_or_default()]);
        data.push("EPD".to_string());
        data.extend(vec![self
            .extra_infos
            .clone()
            .and_then(|x| x.structured())
            .unwrap_or_default()]);
        data.extend(self.alternative_processes.clone());

        data.join("\r\n")
    }

    /// Generates the QR image in string form.
    fn qr_image(&self) -> Result<String, Error> {
        let code = QrCode::with_error_correction_level(self.qr_data(), qrcode::EcLevel::M)?;
        Ok(code
            .render()
            .dark_color(render::svg::Color("black"))
            .light_color(render::svg::Color("white"))
            .quiet_zone(false)
            .build())
    }

    /// Draws the swiss cross in the middle of the QR code.
    fn draw_swiss_cross(group: Group, x: f64, y: f64, size: f64) -> Group {
        let scale_factor = mm(7.0) / 19.0;
        let cross_group = Group::new()
            .add(
            Polygon::new()
                    .set("points", "18.3,0.7 1.6,0.7 0.7,0.7 0.7,1.6 0.7,18.3 0.7,19.1 1.6,19.1 18.3,19.1 19.1,19.1 19.1,18.3 19.1,1.6 19.1,0.7")
                    .set("fill", "black")
            )
            .add(
                Rectangle::new()
                    .set("x", 8.3)
                    .set("y", 4.0)
                    .set("width", 3.3)
                    .set("height", 11.0)
                    .set("fill", "white")
            )
            .add(
                Rectangle::new()
                    .set("x", 4.4)
                    .set("y", 7.9)
                    .set("width", 11.0)
                    .set("height", 3.3)
                    .set("fill", "white")
            )
            .add(
                Polygon::new()
                        .set("points", "0.7,1.6 0.7,18.3 0.7,19.1 1.6,19.1 18.3,19.1 19.1,19.1 19.1,18.3 19.1,1.6 19.1,0.7 18.3,0.7 1.6,0.7 0.7,0.7")
                        .set("fill", "none")
                        .set("stroke", "white")
                        .set("stroke_width", 1.4357)
                )
            .set("transform", format!("translate({}, {}) scale({})", x + size / 2.0 - 10.0 * scale_factor, y + size / 2.0 - 10.0 * scale_factor, scale_factor))
            .set("id", "swiss-cross");

        group.add(cross_group)
    }

    // Draws a single solid black line.
    fn draw_line(group: Group, x1: f64, y1: f64, x2: f64, y2: f64) -> Group {
        group.add(
            Line::new()
                .set("x1", x1)
                .set("y1", y1)
                .set("x2", x2)
                .set("y2", y2)
                .set("stroke", "black")
                .set("stroke-width", "0.26mm")
                .set("stroke-linecap", "square"),
        )
    }

    /// Draws a blank rectangle with given properties.
    fn draw_blank_rectangle(group: Group, x: f64, y: f64, width: f64, height: f64) -> Group {
        // TODO: stroke_info = {'stroke': 'black', 'stroke_width': '0.26mm', 'stroke_linecap': 'square'}
        let mut rectangle_group = Group::new();
        rectangle_group = Self::draw_line(rectangle_group, x, y, x, y + mm(2.0));
        rectangle_group = Self::draw_line(rectangle_group, x, y, x + mm(2.0), y);
        rectangle_group = Self::draw_line(rectangle_group, x, y + height, x, y + height + mm(-2.0));
        rectangle_group = Self::draw_line(rectangle_group, x, y + height, x + mm(3.0), y + height);
        rectangle_group = Self::draw_line(rectangle_group, x + width + mm(-3.0), y, x + width, y);
        rectangle_group = Self::draw_line(rectangle_group, x + width, y, x + width, y + mm(2.0));
        rectangle_group = Self::draw_line(
            rectangle_group,
            x + width + mm(-3.0),
            y + height,
            x + width,
            y + height,
        );
        rectangle_group = Self::draw_line(
            rectangle_group,
            x + width,
            y + height,
            x + width,
            y + height + mm(-2.0),
        );
        group.add(rectangle_group)
    }

    /// Gets the correct translation for a given label.
    fn label(&self, label: &Translation) -> &str {
        match self.language {
            Language::German => label.de,
            Language::English => label.en,
            Language::French => label.fr,
            Language::Italian => label.it,
        }
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

        let pdf = svg2pdf::to_pdf(
            &tree,
            svg2pdf::ConversionOptions::default(),
            svg2pdf::PageOptions::default(),
        );
        std::fs::write(path, pdf)?;
        Ok(())
    }

    /// Returns a string containing the SVG representing the QR-Bill
    ///
    /// * `full_page`: Makes the generated SVG the size of a full A4 page.
    pub fn create_svg(&self, full_page: bool) -> Result<String, Error> {
        // Make a properly sized document with a correct viewbox.
        let document = if full_page {
            Document::new()
                .set("width", format!("{}mm", A4_WIDTH_IN_MM))
                .set("height", format!("{}mm", A4_HEIGHT_IN_MM))
                .set("viewBox", format!("0 0 {} {}", A4_WIDTH, A4_HEIGHT))
        } else {
            Document::new()
                .set("width", format!("{}mm", A4_WIDTH_IN_MM))
                .set("height", format!("{}mm", BILL_HEIGHT_IN_MM))
                .set("viewBox", format!("0 0 {} {}", A4_WIDTH, BILL_HEIGHT))
        };

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
    #[allow(clippy::let_and_return)]
    fn transform_to_full_page(&self, group: Group) -> Group {
        // TODO: Work on the let and return
        let y_offset = A4_HEIGHT - BILL_HEIGHT;
        group.set("transform", format!("translate(0, {})", y_offset))
    }

    /// Draws the entire QR bill SVG image.
    fn draw_bill(&self) -> Result<Group, Error> {
        let margin = mm(5.0);
        let payment_left = RECEIPT_WIDTH + margin;
        let payment_detail_left = payment_left + mm(46.0 + 5.0);

        let mut y_pos = mm(15.0);
        let line_space = mm(3.5);
        let section_space = mm(1.5);

        let mut group = Group::new()
            .add(
                Text::new("")
                    .add(svg::node::Text::new(self.label(&LABEL_RECEIPT)))
                    .set("x", margin)
                    .set("y", mm(10.0))
                    .style(Self::TITLE_FONT),
            )
            .add(
                Text::new("")
                    .add(svg::node::Text::new(self.label(&LABEL_PAYABLE_TO)))
                    .set("x", margin)
                    .set("y", y_pos)
                    .style(Self::HEAD_FONT),
            );

        y_pos += line_space;

        group = group.add(
            Text::new("")
                .add(svg::node::Text::new(self.account.to_string()))
                .set("x", margin)
                .set("y", y_pos)
                .style(Self::FONT),
        );

        y_pos += line_space;

        for line in self.creditor.as_paragraph(MAX_CHARS_RECEIPT_LINE) {
            group = group.add(
                Text::new("")
                    .add(svg::node::Text::new(line))
                    .set("x", margin)
                    .set("y", y_pos)
                    .style(Self::FONT),
            );
            y_pos += line_space;
        }

        if !matches!(self.reference, Reference::None) {
            y_pos += section_space;
            group = group.add(
                Text::new("")
                    .add(svg::node::Text::new(self.label(&LABEL_REFERENCE)))
                    .set("x", margin)
                    .set("y", y_pos)
                    .style(Self::HEAD_FONT),
            );
            y_pos += line_space;
            group = group.add(
                Text::new("")
                    .add(svg::node::Text::new(self.reference.to_string()))
                    .set("x", margin)
                    .set("y", y_pos)
                    .style(Self::FONT),
            );
            y_pos += line_space;
        }

        y_pos += section_space;

        // Add debtor info.
        if let Some(debtor) = &self.debtor {
            group = Self::add_header(
                group,
                self.label(&LABEL_PAYABLE_BY),
                margin,
                &mut y_pos,
                line_space,
            );
            for line in debtor.as_paragraph(MAX_CHARS_PAYMENT_LINE) {
                group = group.add(
                    Text::new(line)
                        .set("x", margin)
                        .set("y", y_pos)
                        .style(Self::FONT),
                );
                y_pos += line_space;
            }
        } else {
            group = Self::add_header(
                group,
                self.label(&LABEL_PAYABLE_BY_EXTENDED),
                margin,
                &mut y_pos,
                line_space,
            );
            group = Self::draw_blank_rectangle(group, margin, y_pos, mm(52.0), mm(20.0));
        }

        group = group.add(
            Text::new(self.label(&LABEL_CURRENCY))
                .set("x", margin)
                .set("y", mm(72.0))
                .style(Self::HEAD_FONT),
        );
        group = group.add(
            Text::new(self.label(&LABEL_AMOUNT))
                .set("x", margin + mm(12.0))
                .set("y", mm(72.0))
                .style(Self::HEAD_FONT),
        );
        group = group.add(
            Text::new(self.currency.to_string())
                .set("x", margin)
                .set("y", mm(77.0))
                .style(Self::FONT),
        );

        if let Some(amount) = self.amount {
            group = group.add(
                Text::new(format_amount(amount))
                    .set("x", margin + mm(12.0))
                    .set("y", mm(77.0))
                    .style(Self::FONT),
            );
        } else {
            group =
                Self::draw_blank_rectangle(group, margin + mm(25.0), mm(70.6), mm(27.0), mm(11.0));
        }

        group = group.add(
            Text::new(self.label(&LABEL_ACCEPTANCE_POINT))
                .set("x", RECEIPT_WIDTH + margin * -1.0)
                .set("y", mm(86.0))
                .set("text-anchor", "end")
                .style(Self::HEAD_FONT),
        );

        if self.top_line {
            group = group.add(
                Line::new()
                    .set("x1", 0.0)
                    .set("y1", mm(0.141))
                    .set("x2", A4_WIDTH)
                    .set("y2", mm(0.141))
                    .set("stroke", "black")
                    .set("stroke-dasharray", "2 2")
                    .set("fill", "none"),
            );

            group = group.add(
                Path::new()
                    .set("d", SCISSORS_SVG_PATH)
                    .set(
                        "style",
                        "fill:#000000; fill-opacity:1; fill-rule:nonzero; stroke:none",
                    )
                    .set("scale", 1.9)
                    .set(
                        "transform",
                        format!(
                            "scale(1.9) translate({}, {})",
                            A4_WIDTH / 2.0 / 1.9,
                            -mm(0.6)
                        ),
                    ),
            );
        }

        if self.payment_line {
            group = group.add(
                Line::new()
                    .set("x1", RECEIPT_WIDTH)
                    .set("y1", 0.0)
                    .set("x2", RECEIPT_WIDTH)
                    .set("y2", BILL_HEIGHT)
                    .set("stroke", "black")
                    .set("stroke-dasharray", "2 2")
                    .set("fill", "none"),
            );

            group = group.add(
                Path::new()
                    .set("d", SCISSORS_SVG_PATH)
                    .set(
                        "style",
                        "fill:#000000; fill-opacity:1; fill-rule:nonzero; stroke:none",
                    )
                    .set("scale", 1.9)
                    .set("transform", "scale(1.9) translate(118, 40) rotate(90)"),
            );
        }

        group = group.add(
            Text::new(self.label(&LABEL_PAYMENT_PART))
                .set("x", payment_left)
                .set("y", mm(10.0))
                .style(Self::TITLE_FONT),
        );

        let path_re = Regex::new(r"<path [^>]*>").unwrap();
        let data_re = Regex::new(r#" d="([^"]*)""#).unwrap();
        let size_re = Regex::new(r#"<svg .* width="(\d*)" [^>]*>"#).unwrap();

        let qr_image = self.qr_image()?;

        let size = size_re
            .captures_iter(&qr_image)
            .next()
            .expect("This is a bug. Please report it.");

        let path = path_re
            .captures_iter(&qr_image)
            .next()
            .expect("This is a bug. Please report it.");

        let data = data_re
            .captures_iter(&path[0])
            .next()
            .expect("This is a bug. Please report it.");

        let qr_left = payment_left;
        let qr_top = 60.0;
        let scale_factor = mm(45.8)
            / size[1]
                .parse::<f64>()
                .expect("This is a bug. Please report it.");

        group = group.add(
            Path::new()
                .set("d", &data[1])
                .set(
                    "style",
                    "fill:black; fill-opacity:1; fill-rule:nonzero; stroke:none; margin: 0",
                )
                .set(
                    "transform",
                    format!("translate({}, {}) scale({})", qr_left, qr_top, scale_factor),
                ),
        );

        group = Self::draw_swiss_cross(group, payment_left, 60.0, mm(45.8));

        group = group.add(
            Text::new(self.label(&LABEL_CURRENCY))
                .set("x", payment_left)
                .set("y", mm(72.0))
                .style(Self::HEAD_FONT),
        );

        group = group.add(
            Text::new(self.label(&LABEL_AMOUNT))
                .set("x", payment_left + mm(12.0))
                .set("y", mm(72.0))
                .style(Self::HEAD_FONT),
        );

        group = group.add(
            Text::new(self.currency.to_string())
                .set("x", payment_left)
                .set("y", mm(77.0))
                .style(Self::FONT),
        );

        if let Some(amount) = self.amount {
            group = group.add(
                Text::new(format_amount(amount))
                    .set("x", payment_left + mm(12.0))
                    .set("y", mm(77.0))
                    .style(Self::FONT),
            );
        } else {
            group = Self::draw_blank_rectangle(
                group,
                RECEIPT_WIDTH + margin + mm(12.0),
                mm(75.0),
                mm(40.0),
                mm(15.0),
            );
        }

        // Draw the right side of the bill (The things right of the QR-Code).
        let mut y_pos = mm(10.0);
        let line_space = mm(3.5);

        group = Self::add_header(
            group,
            self.label(&LABEL_PAYABLE_TO),
            payment_detail_left,
            &mut y_pos,
            line_space,
        );

        group = group.add(
            Text::new(self.account.to_string())
                .set("x", payment_detail_left)
                .set("y", y_pos)
                .style(Self::FONT),
        );
        y_pos += line_space;

        // Draw creditor info.
        for line in self.creditor.as_paragraph(MAX_CHARS_PAYMENT_LINE) {
            group = group.add(
                Text::new(line)
                    .set("x", payment_detail_left)
                    .set("y", y_pos)
                    .style(Self::FONT),
            );
            y_pos += line_space;
        }

        // Draw reference info.
        if !matches!(self.reference, Reference::None) {
            y_pos += section_space;
            group = Self::add_header(
                group,
                self.label(&LABEL_REFERENCE),
                payment_detail_left,
                &mut y_pos,
                line_space,
            );
            group = group.add(
                Text::new(self.reference.to_string())
                    .set("x", payment_detail_left)
                    .set("y", y_pos)
                    .style(Self::FONT),
            );
            y_pos += line_space;
        }

        // Add extra info if present.
        if let Some(extra_info) = &self.extra_infos {
            y_pos += section_space;
            group = Self::add_header(
                group,
                self.label(&LABEL_ADDITIONAL_INFORMATION),
                payment_detail_left,
                &mut y_pos,
                line_space,
            );

            let extra_info = extra_info.as_paragraph().unwrap_or(vec![]);
            for line in extra_info {
                group = group.add(
                    Text::new(line)
                        .set("x", payment_detail_left)
                        .set("y", y_pos)
                        .style(Self::FONT),
                );
                y_pos += line_space;
            }
        }

        y_pos += section_space;

        // Add debtor info.
        if let Some(debtor) = &self.debtor {
            group = Self::add_header(
                group,
                self.label(&LABEL_PAYABLE_BY),
                payment_detail_left,
                &mut y_pos,
                line_space,
            );
            for line in debtor.as_paragraph(MAX_CHARS_PAYMENT_LINE) {
                group = group.add(
                    Text::new(line)
                        .set("x", payment_detail_left)
                        .set("y", y_pos)
                        .style(Self::FONT),
                );
                y_pos += line_space;
            }
        } else {
            group = Self::add_header(
                group,
                self.label(&LABEL_PAYABLE_BY_EXTENDED),
                payment_detail_left,
                &mut y_pos,
                line_space,
            );
            group =
                Self::draw_blank_rectangle(group, payment_detail_left, y_pos, mm(65.0), mm(25.0));
            y_pos += mm(28.0);
        }

        y_pos += section_space;

        // Add extra info if present.
        if let Some(_due_date) = &self.due_date {
            group = Self::add_header(
                group,
                self.label(&LABEL_PAYABLE_BY_DATE),
                payment_detail_left,
                &mut y_pos,
                line_space,
            );

            group = group.add(
                Text::new(format_date(&self.due_date))
                    .set("x", payment_detail_left)
                    .set("y", y_pos)
                    .style(Self::FONT),
            );
            y_pos += line_space;
        }

        // Draw alternative processes.
        y_pos += mm(94.0);
        for alternative_process in &self.alternative_processes {
            group = group.add(
                Text::new(alternative_process)
                    .set("x", payment_left)
                    .set("y", y_pos)
                    .style(Self::PROCESS_FONT),
            );
            y_pos += mm(2.2);
        }

        Ok(group)
    }

    fn add_header(
        group: Group,
        text: impl AsRef<str>,
        payment_detail_left: f64,
        y_pos: &mut f64,
        line_space: f64,
    ) -> Group {
        let group = group.add(
            Text::new(text.as_ref())
                .set("x", payment_detail_left)
                .set("y", *y_pos)
                .style(Self::HEAD_FONT),
        );

        *y_pos += line_space;

        group
    }
}

/// Converts a millimeter based value into a SVG screen units value.
/// This should be used to always do math in sceen units even if we have numbers in mm.
fn mm(value: f64) -> f64 {
    value * MM_TO_UU
}

/// Formats the due date according to spec.
fn format_date(date: &Option<NaiveDate>) -> String {
    date.map(|date| date.format("%d.%m.%Y").to_string())
        .unwrap_or_default()
}

/// Formats the amount according to spec.
fn format_amount(amount: f64) -> String {
    format!("{:.2}", amount).separate_with_spaces()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Way too long string", 0, Error::Name)]
    #[case("Way too long string", 1, Error::Street)]
    #[case("Way too long string", 2, Error::HouseNumber)]
    #[case("Way too long string", 3, Error::PostalCode)]
    #[case("Way too long string", 4, Error::City)]
    fn structured_addr_errs(
        #[case] new_data: &str,
        #[case] x: usize,
        #[case] _erro: Error,
    ) -> anyhow::Result<()> {
        let mut data: Vec<String> = vec![
            "name".into(),
            "street".into(),
            "house_number".into(),
            "postal".into(),
            "city".into(),
        ];
        data[x] = new_data.repeat(5);
        let address = StructuredAddress::new(
            data[0].clone(),
            data[1].clone(),
            data[2].clone(),
            data[3].clone(),
            data[4].clone(),
            CountryCode::CHE,
        );
        assert!(matches!(address.unwrap_err(), _erro));
        Ok(())
    }
}
