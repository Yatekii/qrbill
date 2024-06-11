#[derive(Debug, Clone)]
pub struct Esr {
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
            != checksum((number[..number.len() - 1]).to_string())
        {
            return Err(Error::InvalidChecksum);
        }
        Ok(Self { number })
    }

    pub fn to_raw(&self) -> String {
        self.number.clone()
    }
}

fn checksum(number: String) -> String {
    let digits = [0, 9, 4, 6, 8, 2, 7, 1, 3, 5];
    let mut c = 0usize;
    for n in number.chars() {
        c = digits[(n.to_digit(10).unwrap() as usize + c) % 10];
    }
    ((10 - c) % 10).to_string()
}

impl std::fmt::Display for Esr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let number = "0".repeat(27) + &self.number;
        let number = &number[number.len() - 27..];
        write!(f, "{}",
               number[..2].to_string()
               + " "
               + &number[2..]
                .chars()
                .collect::<Vec<char>>()
                .chunks(5)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
