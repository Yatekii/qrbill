use qrbill::{esr::Esr, Error, NaiveDate, *};
#[cfg(test)]
use rstest::rstest;

const ESR_WITH_CHECKSUM: &str = "240752772";
// const ESR_NOT_VALID: &str = "240752771";
// const QRIID_IBAN_NOT_VALID: &str = "CH12 3080 8001 2345 6789 0";
const QRIID_IBAN_VALID: &str = "CH44 3199 9123 0008 8901 2";
const IID_IBAN_VALID: &str = "CH79 0078 8000 C330 1425 5";

fn structured_addr() -> StructuredAddress {
    StructuredAddress::new(
        "Jean-Jacques Hurluberlu".into(),
        "Rue de la Marinière".into(),
        "43".into(),
        "1630".into(),
        "Bulle".into(),
        isocountry::CountryCode::CHE,
    )
    .unwrap()
}
fn combined_addr() -> CombinedAddress {
    CombinedAddress::new(
        String::from("Jean-Jacques Hurluberlu"),
        String::from("Rue de la Marinière 43"),
        String::from("1630 Bulle"),
        isocountry::CountryCode::CHE,
    )
    .unwrap()
}
fn qr_opts() -> anyhow::Result<QRBillOptions> {
    Ok(QRBillOptions {
        account: QRIID_IBAN_VALID.parse()?,
        creditor: Address::Structured(structured_addr()),
        amount: Some(876.69),
        currency: Currency::SwissFranc,
        due_date: NaiveDate::from_ymd_opt(2024, 8, 29),
        debtor: Some(Address::Cobined(combined_addr())),
        reference: Reference::Qrr(Esr::try_with_checksum(ESR_WITH_CHECKSUM.to_string())?),
        extra_infos: None,
        alternative_processes: vec![],
        language: Language::French,
        top_line: true,
        payment_line: true,
    })
}

#[rstest]
fn new_qr_ok() -> anyhow::Result<()> {
    let opts = qr_opts()?;
    let q = QRBill::new(opts);
    assert!(q.is_ok());
    Ok(())
}
#[rstest]
#[case("240752772", IID_IBAN_VALID, Error::InvalidQriid("".into()))]
#[case("RF18 5390 0754 7034", QRIID_IBAN_VALID, Error::InvalidIid("".into()))]
fn invalid_iban_match(
    #[case] reference: &str,
    #[case] iban: &str,
    #[case] _erro: Error,
) -> anyhow::Result<()> {
    let mut q = qr_opts().unwrap();
    q.account = iban.parse().unwrap();
    if reference.starts_with("RF") {
        q.reference = Reference::Scor(iso11649::Iso11649::try_new(reference.into())?);
    } else {
        q.reference = Reference::Qrr(Esr::try_with_checksum(reference.into())?);
    }
    let q = QRBill::new(q);
    assert!(matches!(q.unwrap_err(), _erro));
    Ok(())
}

// Errors to test ->
//
// #[error("An address line should have between 1 and 70 characters.")]
// Line,
// #[error("An address name should have between 1 and 70 characters.")]
// Name,
// #[error("A street should have between 1 and 70 characters.")]
// Street,
// #[error("A postal code should have between 1 and 16 characters.")]
// PostalCode,
// #[error("A house number should have between 1 and 16 characters.")]
// HouseNumber,
// #[error("A city should have between 1 and 35 characters.")]
// City,
// #[error("The IBAN needs to start with CH or LI.")]
// InvalidIban,
// #[error("The ESR reference is missing and is mandatory for QRIID IbanType")]
// EsrMandatory,
// #[error("IBAN provided ({0:?}) is not SCOR compatible (see IID)")]
// InvalidIid(String),
// #[error("IBAN provided ({0:?}) is not ESR compatible (see QRIID)")]
// InvalidQriid(String),
// #[error("Extra infos can be no more than 140 characters.")]
// ExtraInfos,
// #[error(
//     "At maximum two alternative procedure with a maximum of 100 characters can be specified."
// )]
// AlternativeProcedure,
// #[error("An error with the QR code generation occured.")]
