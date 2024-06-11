#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iso11649 {
    number: String,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Length must be between 5 and 25.")]
    InvalidLength,
    #[error("Number must start with 'RF'.")]
    MissingLeadingRF,
    #[error("Checksum is invalid.")]
    InvalidChecksum,
    #[error("Contains invalid character: {0}")]
    InvalidCharacter(char),
}

impl Iso11649 {
    pub fn try_new(number: &str) -> Result<Self, Error> {
        let number = number.replace([' ', '-', '.', ',', '/', ':'], "");
        if number.len() < 5 || number.len() > 25 {
            return Err(Error::InvalidLength);
        }
        if !number.starts_with("RF") {
            return Err(Error::MissingLeadingRF);
        }

        let valid = format!("{}{}", &number[4..], &number[..4])
            .chars()
            .map(|v| i64::from_str_radix(&v.to_string(), 36)
                 .map_err(|_| Error::InvalidCharacter(v)))
            .collect::<Result<Vec<_>, _>>()?;
        let valid = valid.into_iter()
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

    pub fn to_raw(&self) -> String {
        self.number.clone()
    }
}

impl std::fmt::Display for Iso11649 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.number
            .chars()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use Error::*;

    #[rstest]
    #[case("1234"                       , InvalidLength)]
    #[case("12345"                      , MissingLeadingRF)]
    #[case("1234567890123456789012345"  , MissingLeadingRF)]
    #[case("12345678901234567890123456" , InvalidLength)]
    #[case("RF12345"                    , InvalidChecksum)]
    #[case("RF12345"                    , InvalidChecksum)]
    #[case("RF29fulaño"                 , InvalidCharacter('ñ'))]
    fn test_failures(
        #[case] input: &str,
        #[case] expected: Error,
    ) {
        assert_eq!(Iso11649::try_new(input), Err(expected))
    }

    #[rstest]
    #[case("RF25a"      , "RF25a")]
    #[case("RF95B"      , "RF95B")]
    #[case("RF68C"      , "RF68C")]
    #[case("RF29FULANO" , "RF29FULANO")]
    #[case("RF29fulano" , "RF29fulano")]
    fn test_successes(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let expected = Iso11649 { number: expected.to_string() };
        assert_eq!(Iso11649::try_new(input), Ok(expected))
    }

}
