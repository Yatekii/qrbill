use std::fmt::{Display, Formatter};

use crate::{QR_IID_END, QR_IID_START};
use iban::{Iban, IbanLike};

#[derive(Debug, Clone)]
pub struct Iso11649 {
    number: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Length must be between 5 and 25.")]
    InvalidLength,
    #[error("Number must start with 'RF'.")]
    InvalidFormat,
    #[error("Checksum is invalid.")]
    InvalidChecksum,
    #[error("IBAN provided ({found:?}) is not SCOR compatible (see IID)")]
    InvalidIid { found: String },
    #[error("Parsing error, this is a bug please report")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Iso11649 {
    pub fn try_new(number: String) -> Result<Self, Error> {
        let number = number.replace([' ', '-', '.', ',', '/', ':'], "");
        if number.len() < 5 || number.len() > 25 {
            return Err(Error::InvalidLength);
        }
        if !number.starts_with("RF") {
            return Err(Error::InvalidFormat);
        }

        let valid = format!("{}{}", &number[4..], &number[..4])
            .chars()
            .map(|v| {
                i64::from_str_radix(&v.to_string(), 36).expect("This is a bug. Please report it.")
            })
            .fold(String::new(), |a, b| format!("{}{}", a, b))
            .parse::<u128>()
            .expect("This is a bug. Please report it.")
            % 97
            == 1;
        if !valid {
            return Err(Error::InvalidChecksum);
        }
        Ok(Self { number })
    }

    /// Check if the provided Iban is SCOR compatible
    pub fn validate_iid(&self, iban: &Iban) -> Result<(), Error> {
        let iid: usize = iban.electronic_str()[4..9].parse()?;
        if (QR_IID_START..=QR_IID_END).contains(&iid) {
            return Err(Error::InvalidIid {
                found: iban.to_string(),
            });
        };
        Ok(())
    }

    pub fn to_raw(&self) -> String {
        self.number.clone()
    }
}
impl Display for Iso11649 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let n = self
            .number
            .chars()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{}", n)
    }
}
