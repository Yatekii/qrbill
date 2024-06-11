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
                i64::from_str_radix(&v.to_string(), 36).expect("This is a bug. Please rport it.")
            })
            .fold(String::new(), |a, b| format!("{}{}", a, b))
            .parse::<u64>()
            .expect("This is a bug. Please report it.")
            % 97
            == 1;
        if !valid {
            return Err(Error::InvalidChecksum);
        }
        Ok(Self { number })
    }

    pub fn to_raw(&self) -> String {
        self.number.clone()
    }
}

impl ToString for Iso11649 {
    fn to_string(&self) -> String {
        self.number
            .chars()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ")
    }
}
