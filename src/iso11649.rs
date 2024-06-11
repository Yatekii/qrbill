use deunicode::deunicode;

#[derive(Debug, Clone)]
pub struct Iso11649 {
    original: DigitsBase36,
}

impl Iso11649 {
    pub fn new(any_utf8_text: &str) -> Self {
        Self { original: any_utf8_text.into() }
    }

    pub fn original(&self) -> String {
        self.original.0.clone()
    }

    pub fn with_checksum(&self) -> String {
        let text_in_base_36 = self.without_checksum();
        let base_36_at_most_21_digits: String = text_in_base_36.chars().take(21).collect();
        let text_with_rf00 = DigitsBase36(format!("{base_36_at_most_21_digits}RF00"));
        let digits_decimal = DigitsBase10::from(&text_with_rf00);
        let check_digits = 98 - digits_decimal % 97;
        format!("RF{check_digits:02}{base_36_at_most_21_digits}")
    }

    pub fn without_checksum(&self) -> String {
        deunicode(&self.original.0).to_ascii_uppercase().replace(' ', "")
    }
}



#[derive(Debug, Clone)] struct DigitsBase10(String);
#[derive(Debug, Clone)] struct DigitsBase36(String);

impl From<&str> for DigitsBase36 {
    fn from(source: &str) -> Self {
        Self(deunicode(source)
             .to_ascii_uppercase()
             .chars()
             .filter(|c| c.is_digit(36))
             .collect())
    }
}

impl From<&DigitsBase36> for DigitsBase10 {
    fn from(digits: &DigitsBase36) -> Self {
        Self(digits.0
             .chars()
             .filter_map(|c| c.to_digit(36))
             .map(|d| d.to_string())
             .collect()
        )
    }
}

impl std::fmt::Display for DigitsBase36 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Rem<u8> for DigitsBase10 {
    type Output = u8;

    fn rem(self, rhs: u8) -> Self::Output {
        let mut res = 0;
        for digit in self.0.chars() {
            res = (res * 10 + digit.to_digit(10).unwrap()) % (rhs as u32)
        }
        res.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunked;
    use rstest::rstest;
    use pretty_assertions::assert_eq;

    #[rstest]
    #[case("a", "RF25 A")]
    #[case("b", "RF95 B")]
    #[case("C", "RF68 C")]
    #[case("fulano", "RF29 FULA NO")]
    #[case("FULANO", "RF29 FULA NO")]
    #[case("coti2223pongiste"     , "RF15 COTI 2223 PONG ISTE")]
    #[case("coti2223pongistejc"   , "RF83 COTI 2223 PONG ISTE JC")]
    #[case("coti 2223 pongistejc" , "RF83 COTI 2223 PONG ISTE JC")]
    #[case("coti 2223 pongiste-jc", "RF83 COTI 2223 PONG ISTE JC")]
    #[case("too long because it takes up more than 25 characters", "RF24 TOOL ONGB ECAU SEIT TAKE S")]
    #[case("12345678901234"    , "RF53 1234 5678 9012 34")]
    #[case("1234567890123456789"    , "RF25 1234 5678 9012 3456 789")]
    #[case("12345678901234567890"   , "RF20 1234 5678 9012 3456 7890")]
    #[case("123456789012345678901"  , "RF40 1234 5678 9012 3456 7890 1")]
    #[case("1234567890123456789012" , "RF40 1234 5678 9012 3456 7890 1")]
    #[case("12345678901234567890123", "RF40 1234 5678 9012 3456 7890 1")]
    //#[case("contains-dash"       , "RFXXCONTAINSDASH")]
    fn creditor_reference_check_digits(
        #[case] any_utf8_text: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(chunked(&Iso11649::new(any_utf8_text).with_checksum()), expected);
    }

    #[rstest]
    #[case("a"          , "A")]
    #[case("B"          , "B")]
    #[case("MixEdCaSe"  , "MIXEDCASE")]
    #[case("wIth spACes", "WITHSPACES")]
    #[case("áÀäÄ"       , "AAAA")]
    #[case("éèÉÈ"       , "EEEE")]
    fn remove_spaces_upper(
        #[case] any_utf8_text: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(Iso11649::new(any_utf8_text).without_checksum(), expected);
    }

    #[rstest]
    #[case("0", "0")]
    #[case("A", "10")]
    #[case("RF", "2715")]
    #[case("r2f", "27215")]
    fn base36_digits_to_decimal_digits(#[case] base_36_digits_as_str: &str, #[case] expected: &str) {
        //let calculated = base36_digits_to_decimal_digits(base_36_digits);
        let digits_base_36 = DigitsBase36::from(base_36_digits_as_str);
        let digits_base_10 = DigitsBase10::from(&digits_base_36);
        assert_eq!(digits_base_10.0, expected);
    }

    #[rstest]
    #[case(  "0",  0)]
    #[case(  "1",  1)]
    #[case(  "9",  9)]
    #[case( "97",  0)]
    #[case( "98",  1)]
    #[case("193", 96)]
    #[case( "97000000000000000000000000000000000000000000001", 1)]
    #[case( "97000000000000000000000000000000000000000000099", 2)]
    fn base_36_mod_97(
        #[case] digits: &str,
        #[case] expected: u8,
    ) {
        let calculated = DigitsBase10(digits.into()) % 97;
        assert_eq!(calculated, expected);
    }

    #[rstest]
    #[case("RF25A")]
    #[case("RF61AB")]
    #[case("RF45ABC")]
    #[case("RF67ABCD")]
    #[case("RF09ABCDE")]
    #[case("RF02ABCDEF")]
    #[case("RF51ABCDEFG")] // i64 starts failing at this point
    #[case("RF74ABCDEFGH")] // u64 starts failing at this point
    #[case("RF19ABCDEFGHI")]
    #[case("RF21ABCDEFGHIJ")]
    #[case("RF76ABCDEFGHIJA")]
    #[case("RF20ABCDEFGHIJAB")]
    #[case("RF19ABCDEFGHIJABC")]
    #[case("RF86ABCDEFGHIJABCD")]
    #[case("RF66ABCDEFGHIJABCDE")]
    #[case("RF76ABCDEFGHIJABCDEF")]
    #[case("RF79ABCDEFGHIJABCDEFG")] // u128 starts failing at this point
    #[case("RF61ABCDEFGHIJABCDEFGH")]
    #[case("RF77ABCDEFGHIJABCDEFGHI")]
    #[case("RF98ABCDEFGHIJABCDEFGHIJ")]
    #[case("RF16ABCDEFGHIJABCDEFGHIJA")]
    fn test_parse_overflow(#[case] input: &str) {
        println!("{input}");
        let input_without_checksum = &input[4..];
        let parsed = crate::iso11649::Iso11649::new(input_without_checksum);
        assert_eq!(parsed.without_checksum(), input_without_checksum);
        assert_eq!(parsed.with_checksum()   , input);
    }

}
