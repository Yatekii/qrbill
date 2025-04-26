//! FRENCH: https://www.swiss-qr-invoice.org/downloads/qr-bill-s1-syntax-fr.pdf
//! GERMAN: https://www.swiss-qr-invoice.org/downloads/qr-bill-s1-syntax-de.pdf
//!

use crate::billing_infos::{utils::TotalLenght, BillingInfos, Emitter, RawData, RawDataKind};
use crate::NaiveDate;
use std::{collections::BTreeMap, fmt::Display, sync::Arc};

mod parser;
use parser::s1_parser;
mod builder;
use builder::S1Builder;
mod syntax;
use syntax::Version;

const DATE_FMT: &str = "%y%m%d";

#[derive(Debug, thiserror::Error)]
pub enum SwicoError {
    #[error("Maximum 140 characters authorized for BillingInfos, found: {0}")]
    TooLong(usize),
    #[error("Could not parse Swico string: {0:?}")]
    FromSyntaxParser(#[from] parser::SyntaxParserError),
    #[error("Could not validate Swico syntax {0:?}")]
    FromSyntaxValidator(#[from] syntax::SyntaxValidatorError),
}
type Err = SwicoError;

type StructuredSet = BTreeMap<SwicoComponent, Arc<str>>;
impl TotalLenght for StructuredSet {
    fn tot_len(&self) -> usize {
        self.iter()
            .map(|(rc, l)| l.chars().count() + rc.to_string().chars().count())
            .sum()
    }
}
#[derive(Debug, Clone, Default)]
pub struct Swico {
    version: Option<Version>,
}
impl Swico {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn s1_builder(self) -> S1Builder {
        S1Builder::new()
    }
    // Future-proofing for the version 2 on the SwicoSyntax
    // pub fn s2_builder(self) -> S2Builder {
    //     unimplemented!()
    // }
}
impl RawDataKind for Swico {
    fn raw_data(&self) -> Option<RawData> {
        if let Some(r) = &self.version {
            r.raw_data()
        } else {
            None
        }
    }
}
impl TryFrom<&str> for Swico {
    type Error = Err;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let version = Some(s1_parser(value)?.validate_syntax()?);
        Ok(Self { version })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
enum SwicoComponent {
    Unstructured, // /NAN/ Free text inserted before the structured infos
    Prefix,       // //S1 Prefix used to start the parser
    InvoiceRef,   // /10/ Invoice Reference -- /10/10201409
    DocDate,      // /11/ Document date -- /11/190512
    ClientRef,    // /20/ Client Reference -- /20/140.000-53
    VatNum,       // /30/ VAT Identification Number -- /30/106017086
    VatDate,      // /31/ Invoice date for VAT -- /31/180508 -- /31/181001190131
    VatDetails,   // /32/ VAT Details -- /32/7.7 -- /32/8:1000;2.5:51.8;7.7:250
    VatImport,    // /33/ VAT Importation -- /33/7.7:48.37;2.5:12.4
    Conditions,   // /40/ Discounts -- /40/3:15;0.5:45;0:90
}
impl SwicoComponent {
    fn get_id(&self) -> u8 {
        match self {
            Self::Unstructured => 0,
            Self::Prefix => 1,
            Self::InvoiceRef => 10,
            Self::DocDate => 11,
            Self::ClientRef => 20,
            Self::VatNum => 30,
            Self::VatDate => 31,
            Self::VatDetails => 32,
            Self::VatImport => 33,
            Self::Conditions => 40,
        }
    }
    fn for_parsing<'a>() -> &'a [SwicoComponent] {
        &[
            Self::InvoiceRef,
            Self::DocDate,
            Self::ClientRef,
            Self::VatNum,
            Self::VatDate,
            Self::VatDetails,
            Self::VatImport,
            Self::Conditions,
        ]
    }
    fn invalids() -> Vec<String> {
        let mut v: Vec<u8> = Vec::new();
        let p = Self::for_parsing();
        (0..100).for_each(|i| {
            if !p.iter().any(|f| f.get_id() == i) {
                v.push(i);
            }
        });
        v.into_iter().map(|c| format!("/{:02}/", c)).collect::<Vec<String>>()
    }
}
impl Display for SwicoComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.get_id();
        let data = match self {
            Self::Unstructured => String::new(),
            Self::Prefix => String::from("//"),
            _ => format!("/{:02}/", id),
        };
        f.write_str(&data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn valid() -> anyhow::Result<()> {
        let fmt = "%Y-%m-%d";
        let doc_date = NaiveDate::parse_from_str("2024-06-30", fmt)?;
        let start_vat = NaiveDate::parse_from_str("2024-05-01", fmt)?;
        let s = Swico::new()
            .s1_builder()
            .add_vat_num("112806097")
            .add_client_ref(r"145258\/Dépôt")
            .add_conditions("3:10;0:30")
            .add_invoice_ref("24073428")
            .add_vat_date_naive(start_vat, Some(doc_date))
            .add_doc_date_naive(doc_date)
            .add_unstructured("73 années de retard d'impôts à payer sous 10 jours")
            .build()?;
        let res =
            String::from(r"//S1/10/24073428/11/240630/20/145258\/Dépôt/30/112806097/31/240501240630/40/3:10;0:30");
        assert_eq!(s.structured().unwrap(), res);
        let res = String::from("73 années de retard d'impôts à payer sous 10 jours");
        assert_eq!(s.unstructured().unwrap(), res);
        Ok(())
    }
}
