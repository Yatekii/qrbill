use crate::billing_infos::{
    swico::{StructuredSet, SwicoComponent, DATE_FMT},
    DataType, RawData, RawDataKind,
};
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub enum Version {
    S1(StructuredSet),
    //S2(StructuredSet)
}
impl Version {
    pub fn validate_syntax(self) -> Result<Self, Err> {
        match &self {
            Self::S1(v) => {
                let date = v.get(&SwicoComponent::DocDate);
                if let Some(d) = date {
                    is_date_ok(d)?
                };
                let date = v.get(&SwicoComponent::VatDate);
                if let Some(d) = date {
                    is_date_ok(d)?
                }
                if let Some(vatnum) = v.get(&SwicoComponent::VatNum) {
                    if vatnum.chars().count() != 9 || vatnum.chars().any(|c| !c.is_ascii_digit()) {
                        return Err(Err::InvalidVatNum(vatnum.to_string()));
                    }
                }
                let to_check = [SwicoComponent::InvoiceRef, SwicoComponent::ClientRef];
                to_check.iter().try_for_each(|key| -> Result<(), Err> {
                    if let Some(s) = v.get(key) {
                        invalid_chars(s)?
                    };
                    Ok(())
                })?;
                let to_check = [
                    SwicoComponent::VatDetails,
                    SwicoComponent::VatImport,
                    SwicoComponent::Conditions,
                ];
                to_check.iter().try_for_each(|key| {
                    if let Some(s) = v.get(key) {
                        if s.find(',').is_some() {
                            return Err(Err::InvalidDecimalPoint(s.as_ref().into()));
                        }
                        if key == &SwicoComponent::Conditions {
                            group_control(s, true)?
                        } else {
                            group_control(s, false)?
                        };
                    };
                    Ok(())
                })?;
            }
        }
        Ok(self)
    }
}

impl RawDataKind for Version {
    fn raw_data(&self) -> Option<RawData> {
        let mut rd = RawData::new();
        match self {
            Self::S1(x) => {
                if let Some(u) = x.get(&SwicoComponent::Unstructured) {
                    rd.insert(DataType::Unstructured, vec![u.to_string()]);
                };
                let v = x
                    .iter()
                    .filter(|(c, _)| !matches!(c, &SwicoComponent::Unstructured))
                    .map(|(comp, t)| comp.to_string() + t.as_ref())
                    .collect::<Vec<String>>();
                if !v.is_empty() {
                    rd.insert(DataType::Structured, v);
                }
                if !rd.is_empty() {
                    return Some(rd);
                }
            }
        }
        None
    }
}

fn is_date_ok(d: impl AsRef<str>) -> Result<(), Err> {
    let d = d.as_ref();
    let lenght = d.chars().count();
    match lenght {
        6 => {
            let _ = NaiveDate::parse_from_str(d, DATE_FMT).map_err(Err::FromNaiveDateParse)?;
        }
        12 => {
            let d1 = &d[..6];
            let d2 = &d[6..];
            let _ = NaiveDate::parse_from_str(d1, DATE_FMT).map_err(Err::FromNaiveDateParse)?;
            let _ = NaiveDate::parse_from_str(d2, DATE_FMT).map_err(Err::FromNaiveDateParse)?;
        }
        _ => return Err(Err::InvalidDateFormat(d.to_string())),
    }
    Ok(())
}

fn invalid_chars(s: &str) -> Result<(), Err> {
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' => {
                return Err(Err::InvalidEscapeChar(s.into()));
            }
            '\\' => {
                if let Some(&next_char) = chars.peek() {
                    if next_char != '\\' && next_char != '/' {
                        return Err(Err::InvalidEscapeChar(s.into()));
                    }
                    chars.next();
                } else {
                    return Err(Err::InvalidEscapeChar(s.into()));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn group_control(s: &str, is_condition: bool) -> Result<(), Err> {
    let group = s.split(';').collect::<Vec<&str>>();
    group.iter().try_for_each(|g| -> Result<(), Err> {
        let sub_groups = g.split(':').collect::<Vec<&str>>();
        sub_groups.iter().try_for_each(|sg| -> Result<(), Err> {
            sg.parse::<f32>().map_err(Err::FromParseFloat)?;
            Ok(())
        })?;
        if is_condition {
            if sub_groups.len() != 2 {
                return Err(Err::InvalidConditions(s.into()));
            } else {
                sub_groups
                    .last()
                    .ok_or(Err::InvalidConditions(s.into()))
                    .and_then(|f| f.parse::<u8>().map_err(Err::FromParseInt))?;
            }
        }
        Ok(())
    })?;
    Ok(())
}

type Err = SyntaxValidatorError;

#[derive(Debug, thiserror::Error)]
pub enum SyntaxValidatorError {
    #[error("{0}")]
    FromParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    FromParseFloat(#[from] std::num::ParseFloatError),
    #[error("Invalid Date Format, parsing error: {0}")]
    FromNaiveDateParse(#[from] chrono::ParseError),
    #[error(r"Invalid escape char on: '{0}' -> '\' and '/' must be escaped, replace them by '\\' or '\/'")]
    InvalidEscapeChar(String),
    #[error("VAT ID/NUM must be 9 digits, found: {0}")]
    InvalidVatNum(String),
    #[error(r"An amount or a percentage with decimal places must use the character '.' (full stop) as the separator. Found: {0}")]
    InvalidDecimalPoint(String),
    #[error("Conditions consists of 2 elements, \"Skonto(float):Days(int)\", found: {0}")]
    InvalidConditions(String),
    #[error("Invalid date format, expected YYMMDD, found {0:?}")]
    InvalidDateFormat(String),
}
