use qrbill::{
    billing_infos::BillingInfos, Address, CombinedAddress, CountryCode, NaiveDate, QRBill,
    QRBillOptions, Reference, StructuredAddress,
};

fn main() -> anyhow::Result<()> {
    let bi: BillingInfos =
        "Unstructured message to the buyer//S1/11/240711/10/10239978/20/1348 Dépôt/30/109456872/40/4:5;3:10;0:30/31/240710/32/8.1".parse()?;
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
        extra_infos: Some(bi),
        alternative_processes: vec![],
        language: qrbill::Language::English,
        top_line: true,
        payment_line: true,
    })?;

    qrbill.write_svg_to_file("test.svg", false)?;
    qrbill.write_pdf_to_file("test.pdf", false)?;

    Ok(())
}
