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
    pub fn try_new(number: String) -> Result<Self, Error> {
        let number = number.replace(' ', "").trim_start_matches('0').to_string();
        if number.len() > 27 {
            return Err(Error::InvalidLength);
        }
        if !number.chars().all(char::is_numeric) {
            return Err(Error::InvalidFormat);
        }

        if number[number.len() - 1..number.len()]
            != checksum((number[..number.len() - 1]).to_string())?
        {
            return Err(Error::InvalidChecksum);
        }
        Ok(Self { number })
    }

    pub fn to_raw(&self) -> String {
        self.number.clone()
    }
}

fn checksum(number: String) -> Result<String, Error> {
    let digits = [0, 9, 4, 6, 8, 2, 7, 1, 3, 5];
    let mut c = 0usize;
    for n in number.chars() {
        c = digits[(n.to_digit(10).ok_or(Error::InvalidFormat)? as usize + c) % 10];
    }
    Ok(((10 - c) % 10).to_string())
}

impl ToString for Esr {
    fn to_string(&self) -> String {
        let number = "0".repeat(27) + &self.number;
        let number = &number[number.len() - 27..];
        number[..2].to_string()
            + " "
            + &number[2..]
                .chars()
                .collect::<Vec<char>>()
                .chunks(5)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join(" ")
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
}
