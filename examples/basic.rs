use qrbill::{Address, QRBill, QRBillOptions, Reference, StructuredAddress};

fn main() -> anyhow::Result<()> {
    let qrbill = QRBill::new(QRBillOptions {
        account: "CH51830190NOAHHUESSER".parse()?,
        creditor: Address::Structured(StructuredAddress {
            name: "Noah Hüsser".to_string(),
            street: "Ammerswilerstrasse".to_string(),
            house_number: "31F".to_string(),
            postal_code: "5600".to_string(),
            city: "Lenzburg".to_string(),
            country: isocountry::CountryCode::CHE,
        }),
        amount: Some(42.0),
        currency: qrbill::Currency::SwissFranc,
        due_date: None,
        debtor: None,
        reference: Reference::None,
        extra_infos: None,
        alternative_processes: vec![],
        language: qrbill::Language::English,
        top_line: true,
        payment_line: true,
        font_factor: 1.0,
    })?;

    qrbill.write_to_file("test.svg", false)?;

    Ok(())
}
