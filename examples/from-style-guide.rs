/// Examples taken from
///
/// https://www.six-group.com/dam/download/banking-services/standardization/qr-bill/style-guide-qr-bill-en.pdf
///
/// This document is included in this repository at `qr-standard-docs/style-guide-qr-bill-en.pdf`

use std::collections::HashMap;
use qrbill::{esr, iso11649::Iso11649, Address, Language, QRBill, QRBillOptions, Reference, StructuredAddress};


fn main() -> anyhow::Result<()> {
    for (name, bill) in make_all()? {
        bill.write_svg_to_file(name.to_owned() + ".svg", false)?;
        bill.write_pdf_to_file(name.to_owned() + ".pdf", false)?;
    }
    Ok(())
}

fn make_all() -> anyhow::Result<HashMap<&'static str, QRBill>> {

    //let additional_information = todo!();
    let ref_scor = Reference::Scor(Iso11649::new("5390 0754 7034"));
    let ref_qrr  = Reference::Qrr(esr::Esr::try_new("21 00000 00003 13947 14300 09017".to_string())?);
    let ref_none = Reference::None;

    let extra_1 = Some(
"Auftrag vom 15.06.2020
//S1/10/10201409/11/170309/20/14000000/
30/106017086");

    let extra_2 = Some(
"//S1/10/10201409/11/170309/20/14000000/
30/106017086/31/210122");

    let extra_3 = Some(
"Auftrag vom 15.06.2020
//S1/10/10201409/11/170309/20/14000000/
30/106017086/31/210122
");

    let map = HashMap::from_iter(
        [
            ("1a",  make(0, &ref_qrr , Some(2500.25), None   , true)?),
            ("1b",  make(0, &ref_qrr , Some(1949.75), extra_1, true)?),
            ("2a",  make(1, &ref_scor, Some(2500.25), None   , true)?),
            ("2b",  make(1, &ref_scor, Some(1949.75), extra_2, true)?),
            ("3a",  make(1, &ref_none, Some(1949.75), extra_3, true)?),
            ("3b",  make(2, &ref_none, None         , None   , false)?),
        ]
    );


    Ok(map)
}

fn make(
    creditor: usize,
    reference:   &Reference,
    amount:      Option<f64>,
    extra_infos: Option<&'static str>,
    debtor:      bool,

) -> anyhow::Result<QRBill> {
    let creditor = &[
        Creditor {
            iban:         Some("CH44 3199 9123 0008 8901 2".parse()?),
            name:         "Max Muster & Söhne",
            street:       "Musterstrasse",
            house_number: "123",
            postal_code:  "8000",
            city:         "Seldwyla",
            country:      isocountry::CountryCode::CHE,
        },
        Creditor {
            iban:         Some("CH58 0079 1123 0008 8901 2".parse()?),
            name:         "Max Muster & Söhne",
            street:       "Musterstrasse",
            house_number: "123",
            postal_code:  "8000",
            city:         "Seldwyla",
            country:      isocountry::CountryCode::CHE,
        },
        Creditor {
            iban:         Some("CH52 0483 5012 3456 7100 0".parse()?),
            name:         "Better World Trust",
            street:       "P.O. Box",
            house_number: "",
            postal_code:  "3001",
            city:         "Bern",
            country:      isocountry::CountryCode::CHE,
        }][creditor];

    let debtor = if debtor {
        Some(Address::Structured(StructuredAddress {
            name:         "Simon Muster"  .to_string(),
            street:       "Musterstrasse" .to_string(),
            house_number: "1"             .to_string(),
            postal_code:  "8000"          .to_string(),
            city:         "Seldwyla"      .to_string(),
            country:      isocountry::CountryCode::CHE,
        }))} else { None };


    let qrbill = QRBill::new(QRBillOptions {
        account: creditor.iban.unwrap(),
        creditor: Address::Structured(StructuredAddress {
            name:         creditor.name         .to_string(),
            street:       creditor.street       .to_string(),
            house_number: creditor.house_number .to_string(),
            postal_code:  creditor.postal_code  .to_string(),
            city:         creditor.city         .to_string(),
            country:      creditor.country,
        }),
        amount,
        currency: qrbill::Currency::SwissFranc,
        due_date: None,
        debtor,
        reference: reference.clone(),
        extra_infos: extra_infos.map(Into::into),
        alternative_processes: vec![],
        language: Language::English,
        top_line: true,
        payment_line: true,
    })?;

    Ok(qrbill)
}

struct Creditor {
    iban: Option<iban::Iban>,
    name:         &'static str,
    street:       &'static str,
    house_number: &'static str,
    postal_code:  &'static str,
    city:         &'static str,
    country: isocountry::CountryCode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use pretty_assertions::assert_eq;
    //use qrbill::test_helpers::*;

    #[fixture]
    fn sample_bills() -> HashMap<&'static str, QRBill> {
        make_all().unwrap()
    }

    #[rstest]
    #[case::c1a("1a")]
    #[case::c1b("1b")]
    #[case::c2a("2a")]
    #[case::c2b("2b")]
    #[case::c3a("3a")]
    #[case::c3b("3b")]
    fn test_name(
        sample_bills: HashMap<&'static str, QRBill>,
        #[case] id: &str
    ) {
        let bill = sample_bills.get(id).unwrap();
        let encoded_in_bill = bill.qr_data();
        let path = "examples/images-from-style-guide/".to_owned() + id + ".png.qr-contents";
        let encoded_in_sample = std::fs::read_to_string(path).unwrap();
        compare_sample_with_ours(&encoded_in_sample, &encoded_in_bill, false);
    }

    // Copy-pasted from iso11649 tests
    fn compare_sample_with_ours(sample: &str, ours: &str, show_detail: bool) {
        let   ours_without_returns =   ours.replace('\r', "");
        let sample_without_returns = sample.replace('\r', "");
        let   ours_trimmed =   ours_without_returns.trim();
        let sample_trimmed = sample_without_returns.trim();
        if show_detail {
            for (i, (a,b)) in sample_trimmed.chars().zip(ours_trimmed.chars()).enumerate() {
                println!("{i:03} {a:2} - {b:2}");
                assert_eq!(a,b);
            }
        }
        assert_eq!(sample_trimmed, ours_trimmed);
    }
}
