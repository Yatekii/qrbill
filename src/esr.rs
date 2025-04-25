use std::fmt::{Display, Formatter};

const ESR_MAX_LENGTH: usize = 27;
const ESR_MAX_NO_CHECKSUM: usize = 25;

#[derive(Debug, Clone)]
pub struct Esr {
    number: String,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Length must be between 5 and 25.")]
    InvalidLength,
    #[error("ESR requires only digits")]
    InvalidFormat,
    #[error("Checksum is invalid.")]
    InvalidChecksum,
    #[error("Parsing error, this is a bug please report")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Esr {
    #[deprecated(note = "use `try_without_checksum() or try_with_checksum()` instead")]
    pub fn try_new(number: String) -> Result<Self, Error> {
        let number = number.replace(' ', "").trim_start_matches('0').to_string();
        if number.len() > ESR_MAX_LENGTH {
            return Err(Error::InvalidLength);
        }
        if !number.chars().all(char::is_numeric) {
            return Err(Error::InvalidFormat);
        }
        is_checksum_valid(&number)?;

        Ok(Self { number })
    }

    /// Instantiate a new [`Esr`] struct
    ///
    /// The checksum should already be present at the end of the string!
    /// If your reference doesn't have the checksum calculated use [`Esr::try_without_checksum`] instead.
    pub fn try_with_checksum(number: String) -> Result<Self, Error> {
        let number = number.replace(' ', "").trim_start_matches('0').to_string();
        if number.len() > ESR_MAX_LENGTH || number.len() < 5 {
            return Err(Error::InvalidLength);
        }
        is_checksum_valid(&number)?;

        Ok(Self { number })
    }

    /// Instantiate a new [`Esr`] struct and calculate the checksum digit
    ///
    /// The checksum should not be provided at the end of the string!
    pub fn try_without_checksum(value: String) -> Result<Self, Error> {
        let value = value.replace(' ', "").trim_start_matches('0').to_string();
        if value.len() > ESR_MAX_NO_CHECKSUM || value.len() < 5 {
            return Err(Error::InvalidLength);
        };
        let new_checksum = checksum(value.clone())?;
        let number = format!("{}{}", value, new_checksum);
        is_checksum_valid(&number)?;

        Ok(Self { number })
    }

    pub fn to_raw(&self) -> String {
        //leading 0's are mandatory in the spec
        //not doing so results of certain automated paiement systems to fail on processing said paiements
        let esr_repr = "0".repeat(ESR_MAX_LENGTH) + &self.number;
        esr_repr[esr_repr.len() - ESR_MAX_LENGTH..].into()
    }
}

fn is_checksum_valid(number: &str) -> Result<(), Error> {
    let check_digit = number[number.len() - 1..number.len()].to_string();
    let sample = number[..number.len() - 1].to_string();
    let res = checksum(sample)?;
    if check_digit != res {
        return Err(Error::InvalidChecksum);
    };
    Ok(())
}

/// Return the checksum digit as a `Result<String, Error>`
fn checksum(number: String) -> Result<String, Error> {
    let digits = [0, 9, 4, 6, 8, 2, 7, 1, 3, 5];
    let mut c = 0usize;
    for n in number.chars() {
        c = digits[(n.to_digit(10).ok_or(Error::InvalidFormat)? as usize + c) % 10];
    }
    Ok(((10 - c) % 10).to_string())
}

/// Format the reference number as a String to "00 00000 00000 00000 00000 00000"
impl Display for Esr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let number = self.to_raw();
        let number = number[..2].to_string()
            + " "
            + &number[2..]
                .chars()
                .collect::<Vec<char>>()
                .chunks(5)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join(" ");
        write!(f, "{}", number)
    }
}

#[cfg(test)]
mod test {
    use super::{Error, *};
    use rstest::rstest;
    #[rstest]
    #[case("24075237", "1")]
    #[case("24075277", "2")]
    #[case("240", "4")]
    fn checksum_correct(#[case] sample: &str, #[case] res: &str) -> () {
        let s = String::from(sample);
        let r = String::from(res);
        assert_eq!(checksum(s).unwrap(), r)
    }
    #[rstest]
    #[case("24075237", Error::InvalidChecksum)]
    #[case("24075A37", Error::InvalidFormat)]
    #[case("2404", Error::InvalidLength)]
    #[case("2100000000000458423122404546", Error::InvalidLength)]
    fn try_new_errors(#[case] sample: &str, #[case] erro: Error) -> () {
        let s = String::from(sample);
        let esr = Esr::try_with_checksum(s);
        assert_eq!(esr.unwrap_err(), erro)
    }
    #[rstest]
    #[case("0000000024075277", false)]
    #[case("240752371", true)]
    #[case("240752772", true)]
    #[case("210000000000045842312240456", true)]
    fn try_from_ok(#[case] sample: &str, #[case] with_checksum: bool) -> () {
        let sample = String::from(sample);
        if with_checksum {
            let esr = Esr::try_with_checksum(sample);
            assert!(esr.is_ok())
        } else {
            let esr = Esr::try_without_checksum(sample);
            assert!(esr.is_ok())
        }
    }
}
