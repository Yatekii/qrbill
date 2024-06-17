use chrono::NaiveDate;
use qrbill::{esr::Esr, Error, *};

const ESR_WITH_CHECKSUM: &str = "240752772";
const ESR_NOT_VALID: &str = "240752771";
const QRIID_IBAN_NOT_VALID: &str = "CH12 3080 8001 2345 6789 0";
const QRIID_IBAN_VALID: &str = "CH44 3199 9123 0008 8901 2";
const IID_IBAN_VALID: &str = "CH79 0078 8000 C330 1425 5";

fn debt_addr_no_err() -> StructuredAddress {
    StructuredAddress {
        name: String::from("Jean-Jacques Hurluberlu"),
        street: String::from("Rue de la Marinière"),
        house_number: String::from("43"),
        postal_code: String::from("1630"),
        city: String::from("Bulle"),
        country: isocountry::CountryCode::CHE,
    }
}

fn qr_opts() -> anyhow::Result<QRBillOptions> {
    Ok(QRBillOptions {
        account: QRIID_IBAN_VALID.parse()?,
        creditor: Address::Structured(debt_addr_no_err()),
        amount: Some(876.69),
        currency: Currency::SwissFranc,
        due_date: NaiveDate::from_ymd_opt(2024, 8, 29),
        debtor: Some(Address::Structured(debt_addr_no_err())),
        reference: Reference::Qrr(Esr::try_with_checksum(ESR_WITH_CHECKSUM.to_string())?),
        extra_infos: None,
        alternative_processes: vec![],
        language: Language::French,
        top_line: true,
        payment_line: true,
    })
}

#[test]
fn new_qr() -> Result<(), Error> {
    let q = QRBill::new(qr_opts().unwrap())?;
    let q = q.create_svg(false);
    assert!(q.is_ok());
    Ok(())
}
#[test]
fn new_qr_err() -> Result<(), Error> {
    let mut q = qr_opts().unwrap();
    q.account = IID_IBAN_VALID.parse().unwrap();
    let q = QRBill::new(q);
    assert!(matches!(
        q.unwrap_err(),
        Error::Esr(esr::Error::InvalidQriid { found: _ })
    ));
    Ok(())
}
