const ESR_MAX_LENGTH: usize = 27;
const ESR_MAX_NO_CHECKSUM: usize = 25;
#[derive(Debug, Clone)]
pub struct Esr {
    number: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Length must be between 5 and 25.")]
    InvalidLength,
    #[error("ESR requires only digits")]
    InvalidFormat,
    #[error("Checksum is invalid.")]
    InvalidChecksum,
}

impl Esr {
    #[deprecated(
        since = "0.2.4",
        note = "use `try_without_checksum() or try_with_checksum()` instead"
    )]
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
    /// The checksum should already be present at the end of the string!
    /// If your reference doesn't have the checksum calculated use this method instead
    /// ```
    /// qrbill::esr::Esr::try_without_checksum("00000024072049".to_string());
    /// ```
    pub fn try_with_checksum(number: String) -> Result<Self, Error> {
        let number = number.replace(' ', "").trim_start_matches('0').to_string();
        if number.len() > ESR_MAX_LENGTH {
            return Err(Error::InvalidLength);
        }
        is_checksum_valid(&number)?;

        Ok(Self { number })
    }

    /// Instantiate a new [`Esr`] struct and calculate the checksum digit
    /// The checksum should not be provided at the end of the string!
    /// Provide the checksum digit at the end of the String
    pub fn try_without_checksum(value: String) -> Result<Self, Error> {
        let value = value.replace(' ', "").trim_start_matches('0').to_string();
        if value.len() > ESR_MAX_NO_CHECKSUM {
            return Err(Error::InvalidLength);
        };
        let new_checksum = checksum(value.clone())?;
        let number = format!("{}{}", value, new_checksum);
        is_checksum_valid(&number)?;

        Ok(Self { number })
    }

    pub fn to_raw(&self) -> String {
        self.number.clone()
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
        let number = "0".repeat(27) + &self.number;
        let number = &number[number.len() - 27..];
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
    use super::*;
    #[test]
    fn checksum_ok() {
        let sample = String::from("24075237");
        let res = String::from("1");
        assert_eq!(checksum(sample).unwrap(), res)
    }
    #[test]
    fn checksum_not_ok() {
        let sample = String::from("24075277");
        let res = String::from("1"); // Correct checksum = "2"
        assert_ne!(checksum(sample).unwrap(), res)
    }
    #[test]
    fn checksum_error() {
        let sample = String::from("24075A37");
        assert!(matches!(
            checksum(sample).unwrap_err(),
            Error::InvalidFormat
        ))
    }
    #[test]
    fn try_new_error() {
        let sample = String::from("24075277");
        let esr = Esr::try_with_checksum(sample);
        assert!(matches!(esr.unwrap_err(), Error::InvalidChecksum))
    }
    #[test]
    fn try_from_ok() {
        let sample = String::from("0000000024075277");
        let esr = Esr::try_without_checksum(sample).unwrap();
        assert_eq!(esr.to_raw(), "240752772".to_string())
    }
    #[test]
    fn invalid_format() {
        let sample = String::from("24075A37");
        let esr = Esr::try_without_checksum(sample);
        assert!(matches!(esr.unwrap_err(), Error::InvalidFormat))
    }
}
