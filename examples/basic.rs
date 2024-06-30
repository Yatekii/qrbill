use chrono::NaiveDate;
use qrbill::{
    Address, CombinedAddress, CountryCode, QRBill, QRBillOptions, Reference, StructuredAddress,
};

fn main() -> anyhow::Result<()> {
    let qrbill = QRBill::new(QRBillOptions {
        account: "CH5800791123000889012".parse()?,
        creditor: Address::Structured(StructuredAddress::new(
            "Noah Huesser".into(),
            "Ammerswilerstrasse".into(),
            "31F".into(),
            "5600".into(),
            "Lenzburg".into(),
            CountryCode::CHE,
        )?),
        amount: Some(419.68), // or None,
        currency: qrbill::Currency::SwissFranc,
        due_date: Some(NaiveDate::parse_from_str("2032.10.25", "%Y.%m.%d")?),
        debtor: Some(Address::Combined(CombinedAddress::new(
            "Jean-Eude".into(),
            "Rue du paiement 56B".into(),
            "1700 Fribourg".into(),
            CountryCode::CHE,
        )?)),
        reference: Reference::None,
        extra_infos: None,
        alternative_processes: vec![],
        language: qrbill::Language::English,
        top_line: true,
        payment_line: true,
    })?;

    qrbill.write_svg_to_file("test.svg", false)?;
    qrbill.write_pdf_to_file("test.pdf", false)?;

    Ok(())
}
