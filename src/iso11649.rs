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
    use rstest::*;
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

    struct Example { bill: crate::QRBill, expected_data: String }

    #[fixture]
    fn example1() -> Example {
        use crate::{Address, Currency, Language, QRBill, QRBillOptions, Reference, StructuredAddress};

        let iban = "CH8200788000C33011582";
        let creditor_name = "Etat de Genève";
        let creditor_street = "Avenue des Impôts";
        let creditor_house_number = 42;
        let creditor_postal_code = 1211;
        let creditor_city = "Genève";
        let creditor_country = isocountry::CountryCode::CHE;
        let amount = 12345.67;

        let creditor = Address::Structured(StructuredAddress {
            name         : creditor_name.into(),
            street       : creditor_street.into(),
            house_number : creditor_house_number.to_string(),
            postal_code  : creditor_postal_code.to_string(),
            city         : creditor_city.into(),
            country      : creditor_country,
        });

        let debtor_name = "Jean-Philippe Contribuable";
        let debtor_street = "Prôméñądë dès Dïàçrîtiqêß";
        let debtor_house_number = 12;
        let debtor_postal_code = 3456;
        let debtor_city = "Rochemouillé-sur-Lac";
        let debtor_country = isocountry::CountryCode::CHE;
        let extra_infos = "Extra infos";
        // TODO due_date seems to have no effect on the data encoded in the QR code
        let due_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 30)
            .expect("Hard-wired test date should parse");
        // let alternative1 = "Alternative process 1";
        // let alternative2 = "Another alternative process";

        let debtor = Some(Address::Structured(StructuredAddress {
            name: debtor_name.into(),
            street: debtor_street.into(),
            house_number: debtor_house_number.to_string(),
            postal_code: debtor_postal_code.to_string(),
            city: debtor_city.into(),
            country: debtor_country,
        }));

        let reference_input = "ABCD 1234 AU";
        let reference = Iso11649::new(reference_input);
        let reference_coded = reference.with_checksum();

        let bill = QRBill::new(QRBillOptions {
            account: iban.parse().expect("Hard-wired test IBAN should parse"),
            creditor,
            amount: Some(amount),
            currency: Currency::SwissFranc,
            due_date: Some(due_date),
            debtor,
            reference: Reference::Scor(reference),
            extra_infos: Some(extra_infos.into()),
            alternative_processes: vec![],// TODO reinstate when alt-procs implemented vec![alternative1.into(), alternative2.into()],
            language: Language::French,
            top_line: true,
            payment_line: true,
        }).expect("Should be able to create test example QRBill");

        // Write example out to local directory, for easier human inspection.
        // Comment out when not used.
        let path = "test-example1.pdf";
        bill.write_pdf_to_file(path, false)
            .expect("Should be able to write test example to {path}.");

        let expected_data = format!("
SPC
0200
1
{iban}
S
{creditor_name}
{creditor_street}
{creditor_house_number}
{creditor_postal_code}
{creditor_city}
CH







{amount}
CHF
S
{debtor_name}
{debtor_street}
{debtor_house_number}
{debtor_postal_code}
{debtor_city}
CH
SCOR
{reference_coded}
{extra_infos}
EPD",
// TODO reinstate when alt-procs implemented
// {alternative1}
// {alternative2}
        )[1..].to_string();

        Example { bill, expected_data  }

    }

    #[rstest]
    fn qr_data(example1: Example) {
        let Example { bill, expected_data } = example1;
        compare_original_with_derived(&bill.qr_data(), &expected_data, false);
    }

    //#[rstest] // TODO fix test qr_data_via_in_memory_image
    fn todo_qr_data_via_in_memory_image(example1: Example) {
        let Example { bill, expected_data } = example1;
        let pixmap = render_svg_data_to_png(&bill.qr_image().unwrap())
            .unwrap();

        let image = image::io::Reader::new(std::io::Cursor::new(pixmap.data()))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        let recovered_data = decode_single_qr_code_in_image(image);

        compare_original_with_derived(&recovered_data, &expected_data, false);
    }

    #[rstest]
    fn qr_data_via_saved_image(example1: Example) {
        let Example { bill, expected_data } = example1;

        // `temp_dir` will be deleted at the end of the test:
        // Append `.permanent()` to keep it around.
        let temp_dir = temp_testdir::TempDir::default();
        let png_path = temp_dir.join("test_output.png");

        render_svg_data_to_png(&bill.qr_image().unwrap())
            .unwrap()
            .save_png(&png_path)
            .expect("Failed to save pixmap to PNG");
        let recovered_data = decode_single_qr_code_in_image_at_path(png_path);

        compare_original_with_derived(&recovered_data, &expected_data, false);
    }

    fn compare_original_with_derived(original: &str, derived: &str, show_detail: bool) {
        let  derived_without_returns =  derived.replace('\r', "");
        let original_without_returns = original.replace('\r', "");
        if show_detail {
            for (i, (a,b)) in original_without_returns.chars().zip(derived_without_returns.chars()).enumerate() {
                println!("{i:03} {a:2} - {b:2}");
                assert_eq!(a,b);
            }
        }
        assert_eq!(original_without_returns, derived_without_returns);
    }

    fn render_svg_data_to_png(svg_data: &str) -> Option<resvg::tiny_skia::Pixmap> {
        use resvg::tiny_skia::{Transform, Pixmap};
        use resvg::usvg::{Options, Tree};
        let opt = Options::default();
        let rtree = Tree::from_str(svg_data, &opt).ok()?;
        let pixmap_size = rtree.size();
        let mut pixmap = Pixmap::new(pixmap_size.width() as u32, pixmap_size.height() as u32)?;
        resvg::render(&rtree, Transform::identity(), &mut pixmap.as_mut());
        Some(pixmap)
    }

    fn decode_single_qr_code_in_image_at_path(path: impl AsRef<std::path::Path>) -> String {
        decode_single_qr_code_in_image(image::open(path).unwrap())
    }

    fn decode_single_qr_code_in_image(image: image::DynamicImage) -> String {
        let decoder = bardecoder::default_decoder();
        let results = decoder.decode(&image);
        results.into_iter().next().unwrap().unwrap().replace('\r', "")
    }

}
