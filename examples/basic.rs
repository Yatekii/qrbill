use std::{fs, path::Path};
use qrbill::{iso11649::Iso11649, Address, Currency, Language, QRBill, QRBillOptions, Reference, StructuredAddress};

fn main() -> anyhow::Result<()> {
    let qrbill = QRBill::new(QRBillOptions {
        account: "CH5800791123000889012".parse()?,
        creditor: Address::Structured(StructuredAddress {
            name: "Noah Huesser".to_string(),
            street: "Ammerswilerstrasse".to_string(),
            house_number: "31F".to_string(),
            postal_code: "5600".to_string(),
            city: "Lenzburg".to_string(),
            country: isocountry::CountryCode::CHE,
        }),
        amount: None, //Some(42.0),
        currency: qrbill::Currency::SwissFranc,
        due_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()),
        //due_date: None,
        debtor: None,
        reference: Reference::None,
        extra_infos: Some("This that and the other".parse()?), //None,
        alternative_processes: vec![],
        language: qrbill::Language::English,
        top_line: true,
        payment_line: true,
    })?;

    let out_dir = "example-output".to_owned();
    std::fs::create_dir_all(&out_dir)?;
    let base = Path::new(&out_dir).join("test1");

    qrbill.write_svg_to_file(base.with_extension("svg"), false)?;
    qrbill.write_pdf_to_file(base.with_extension("pdf"), false)?;
    fs::write(base.with_extension("qr-data"), qrbill.qr_data())?;


    let qrbill = QRBill::new(QRBillOptions {
        account: "CH8200788000C33011582".parse()?,
        creditor: Address::Structured(StructuredAddress {
            name: "Êtat de Genève".to_string(),
            street: "Avenue des Impôts".to_string(),
            house_number: "42".to_string(),
            postal_code: "1211".to_string(),
            city: "Genève".to_string(),
            country: isocountry::CountryCode::CHE,
        }),
        amount: Some(12345.67),
        currency: Currency::SwissFranc,
        due_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()),
        debtor: Some(Address::Structured(StructuredAddress {
            name: "Jean-Philippe Contribuable".to_string(),
            street: "Prôméñądë dès Dïàçrîtiqêß".to_string(),
            house_number: "12".to_string(),
            postal_code: "3456".to_string(),
            city: "Rochemouillé-sur-Mer".to_string(),
            country: isocountry::CountryCode::FRA,
        })),
        reference: Reference::Scor(Iso11649::new("Abcd 1234 áü")),
        extra_infos: Some("Extra infos".parse()?),
        alternative_processes: vec![
            "Alternative process 1".into(),
            "Another alternative process".into()
        ],
        language: Language::French,
        top_line: true,
        payment_line: true,
    })?;

    let base = Path::new(&out_dir).join("test2");
    qrbill.write_svg_to_file(base.with_extension("svg"), false)?;
    qrbill.write_pdf_to_file(base.with_extension("pdf"), false)?;
    fs::write(base.with_extension("qr-data"), qrbill.qr_data())?;

    Ok(())
}
