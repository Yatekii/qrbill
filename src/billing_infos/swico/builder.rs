use crate::billing_infos::BKG_INF_CHAR_LIMIT;

use super::{
    Arc, BillingInfos, Emitter, NaiveDate, StructuredSet, Swico, SwicoComponent, SwicoError, TotalLength, Version,
    DATE_FMT,
};
#[derive(Debug, Default, Clone)]
pub struct S1Builder {
    structured_set: StructuredSet,
}
impl S1Builder {
    pub fn new() -> Self {
        Self {
            structured_set: StructuredSet::new(),
        }
    }
    /// Add unstructured message into the billing informations
    pub fn add_unstructured(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.structured_set
            .insert(SwicoComponent::Unstructured, Arc::from(text.as_ref()));
        self
    }
    /// Voucher/Invoice/Bill number
    ///
    /// The voucher date is the same as the date of the invoice
    /// It is used as the reference date for the terms and conditions.
    /// Together with the field /40/0:n, a maturity date of the invoice can be calculated
    /// (payable within n days after the voucher date).
    pub fn add_invoice_ref(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.structured_set
            .insert(SwicoComponent::InvoiceRef, Arc::from(text.as_ref()));
        self
    }
    /// Voucher/Invoice/Bill date
    pub fn add_doc_date_naive(&mut self, date: NaiveDate) -> &mut Self {
        let text = format!("{}", date.format(DATE_FMT));
        self.structured_set
            .insert(SwicoComponent::DocDate, Arc::from(text.as_ref()));
        self
    }
    /// Reference from the client/customer/debtor
    ///
    /// The customer reference is a reference sent by the customer
    /// and is used to identify the bill
    pub fn add_client_ref(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.structured_set
            .insert(SwicoComponent::ClientRef, Arc::from(text.as_ref()));
        self
    }
    /// TVA/MWST/VAT/IVA CH-UID From the creditor
    ///
    /// The VAT number is the same as the numerical UID of the service provider
    /// (without the CHE prefix, separator and VAT suffix).
    /// The VAT number can be used by the bill recipient to identify the bill issuer unambiguously.
    /// All bill issuers who have a UID should enter it here,
    /// even if the other VAT fields are omitted.
    /// For a bill with more than one VAT number, the first should be entered
    pub fn add_vat_num(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.structured_set
            .insert(SwicoComponent::VatNum, Arc::from(text.as_ref()));
        self
    }
    /// VAT Date on which the service was provided
    ///
    /// Accepted date format: "YYMMDD" or "YYMMDDYYMMDD"
    /// Can be a single date or a range between to dates
    pub fn add_vat_date_naive(&mut self, start: NaiveDate, end: Option<NaiveDate>) -> &mut Self {
        let mut text = format!("{}", start.format(DATE_FMT));
        if let Some(d) = end {
            let e = format!("{}", d.format(DATE_FMT));
            text.push_str(&e);
        }
        self.structured_set
            .insert(SwicoComponent::VatDate, Arc::from(text.as_ref()));
        self
    }
    /// The VAT details refer to the invoiced amount, excluding any discount.
    ///
    /// VAT details contain either:
    /// – a single percentage that is to be applied to the whole invoiced amount or
    /// – a list of the VAT amounts, defined by a percentage rate and a net amount; the colon “:”
    /// is used as the separator.
    /// The net amount is the net price (excluding VAT) on which the VAT is calculated.
    /// If a list is given, the total of the net amounts and the VAT calculated on them must
    /// correspond to the amount in the QR Code.
    pub fn add_vat_details(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.structured_set
            .insert(SwicoComponent::VatDetails, Arc::from(text.as_ref()));
        self
    }
    /// Where goods are imported, the import tax can be entered in this field.
    ///
    /// The amount is the VAT amount.
    /// The rate serves correct recording of VAT in the accounts.
    /// This makes it easier for the bill recipient to record the VAT in the case of an import.
    pub fn add_vat_import(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.structured_set
            .insert(SwicoComponent::VatImport, Arc::from(text.as_ref()));
        self
    }
    /// The terms and conditions may refer to a discount or list of discounts.
    ///
    /// The voucher date /11/ counts as the reference date.
    /// Each discount is defined by a percentage and a deadline (in days);
    /// The colon “:” is used as the separator.
    /// The indication with a percentage rate equal to zero
    /// defines the default payment date of the invoice (e.g. “0:30” for 30 days net).
    ///
    /// # Attention:
    ///
    /// When this day is used,
    /// at least the default payment date of the invoice should be indicated.
    /// Without this indication,
    /// the payment software will not be able to suggest any date for the payment.
    pub fn add_conditions(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.structured_set
            .insert(SwicoComponent::Conditions, Arc::from(text.as_ref()));
        self
    }
    /// Return the Swico Builder [`S1Builder`] as [`Result<BillingInfos, SwicoError>`]
    ///
    /// # Behavior
    ///
    /// Validate each fields of the structured data set
    /// Max authorized length of characters combining structured and unstructured: [`140`]
    pub fn build(&mut self) -> Result<BillingInfos, SwicoError> {
        if self.structured_set.len() > 1 {
            self.structured_set.insert(SwicoComponent::Prefix, Arc::from("S1"));
        }
        let max_len = self.structured_set.tot_len();
        if max_len > BKG_INF_CHAR_LIMIT {
            return Err(SwicoError::TooLong(max_len));
        }
        let vers = Version::S1(self.structured_set.clone()).validate_syntax()?;
        Ok(BillingInfos {
            emitter:            Some(Emitter::Swico(Swico { version: Some(vers) })),
            unstructured_field: None,
        })
    }
}
