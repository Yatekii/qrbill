mod swico;
use std::str::FromStr;
mod utils;
use utils::{make_paragraph_from_raw, Fold, RawData, RawDataKind};

use swico::Swico;

type BillingInfoParagrah = Vec<String>;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DataType {
    Unstructured,
    Structured,
}

#[derive(Debug, Clone)]
enum Emitter {
    Swico(Swico),
}

#[derive(Debug, Clone, Default)]
pub struct BillingInfos {
    emitter:            Option<Emitter>,
    unstructured_field: Option<String>,
}
impl BillingInfos {
    pub fn new() -> Self {
        Self::default()
    }
    /// Add an unstructured message to BillingInfos
    ///
    /// # Behavior
    ///
    /// Consume self and return Self
    /// Overwrite any existing unstructured message
    /// Return an error if both fields (unstructured + structured)
    /// are longer than 140 characters
    ///
    /// # Example / Tests
    ///
    /// ```
    /// fn main() -> anyhow::Result<()> {
    /// use qrbill::billing_infos::{BillingInfos};
    /// let msg = "Invoice F248956-24RI for a new gaming chair";
    /// let bi = BillingInfos::new().add_unstructured(msg)?;
    /// assert_eq!(bi.unstructured().unwrap(), msg);
    /// assert!(bi.structured().is_none());
    /// let other_msg = "Some invoice message";
    ///
    /// let bi = BillingInfos::swico();
    /// let mut bi = bi.s1_builder();
    /// let bi = bi.add_unstructured(other_msg).build();
    /// assert!(bi.is_ok());
    ///
    /// let bi = bi?.clone();
    /// assert!(&bi.structured().is_none());
    /// assert_eq!(&bi.unstructured().unwrap(), other_msg);
    ///
    /// let bi = bi.add_unstructured(msg)?;
    /// assert_ne!(bi.unstructured().unwrap(), other_msg);
    ///
    /// let long_msg = msg.repeat(10);
    /// let bi = BillingInfos::new().add_unstructured(long_msg);
    /// assert!(bi.is_err());
    /// Ok(())
    /// }
    ///
    /// ```
    pub fn add_unstructured(self, text: impl AsRef<str>) -> Result<Self, BillingInfoError> {
        let i = text.as_ref().chars().count();
        if i > 140 {
            return Err(BillingInfoError::Swico(swico::SwicoError::TooLong(i)));
        }
        if let Some(c) = &self.structured() {
            let i = c.chars().count() + i;
            if i > 140 {
                return Err(BillingInfoError::Swico(swico::SwicoError::TooLong(i)));
            }
        }
        Ok(BillingInfos {
            emitter:            self.emitter,
            unstructured_field: Some(text.as_ref().to_string()),
        })
    }
    pub fn swico() -> Swico {
        Swico::default()
    }
    /// Return an [`Option<Vec<String>>`] (A paragraph) to be displayed onto the QrBill image
    ///
    /// # Behavior
    ///
    /// Split the unstructured_infos and the structured_infos into multiple lines
    /// unstructured_infos always goes at the top (1st line)
    /// structured_infos goes under and is splited based on length
    pub fn as_paragraph(&self) -> Option<BillingInfoParagrah> {
        let mut r = RawData::new();
        if let Some(emitter) = self.emitter.as_ref() {
            match emitter {
                Emitter::Swico(x) => {
                    if let Some(y) = x.raw_data() {
                        r.extend(y);
                    }
                }
            };
        };
        if let Some(u) = &self.unstructured_field {
            r.insert(DataType::Unstructured, vec![u.clone()]);
        }
        make_paragraph_from_raw(&r)
    }
    /// Return an [`Option<String>`] of the structured for the QR_Data
    ///
    /// Data is returned as a single line
    pub fn structured(&self) -> Option<String> {
        if let Some(emitter) = self.emitter.as_ref() {
            match emitter {
                Emitter::Swico(x) => x.raw_data().and_then(|x| x.fold_from(&DataType::Structured)),
            }
        } else {
            None
        }
    }
    /// Return an [`Option<String>`] of the unstructured message for the QR_Data
    ///
    /// Data is returned as a single line
    pub fn unstructured(&self) -> Option<String> {
        if self.unstructured_field.is_some() {
            self.unstructured_field.clone()
        } else if let Some(emitter) = self.emitter.as_ref() {
            match emitter {
                Emitter::Swico(x) => x.raw_data().and_then(|x| x.fold_from(&DataType::Unstructured)),
            }
        } else {
            None
        }
    }
    pub fn len(&self) -> usize {
        let u: usize = self.unstructured().map(|f| f.chars().count()).unwrap_or(0);
        let s: usize = self.structured().map(|f| f.chars().count()).unwrap_or(0);
        u + s
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl FromStr for BillingInfos {
    type Err = BillingInfoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("//S1") {
            let emitter = Some(Emitter::Swico(
                Swico::try_from(s).map_err(BillingInfoError::Swico)?,
            ));
            Ok(Self {
                emitter,
                unstructured_field: None,
            })
        } else {
            let unstructured = Swico::new().s1_builder().add_unstructured(s).build()?;
            Ok(Self {
                emitter:            None,
                unstructured_field: unstructured.unstructured_field,
            })
            // Err(BillingInfoError::FromParser(s.into()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BillingInfoError {
    #[error("Error from Swico: {0:?}")]
    Swico(#[from] swico::SwicoError),
    #[error("Could not parse string into Swico: {0:?}")]
    FromParser(String),
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn unstructured_hierarchy() -> anyhow::Result<()> {
        let res_builder = String::from("Unstructured from builder");
        let res_struct = String::from("Unstructured from struct");
        let bi = BillingInfos::swico()
            .s1_builder()
            .add_unstructured(&res_builder)
            .build()?
            .add_unstructured(&res_struct)?;
        assert_eq!(bi.unstructured().unwrap_or_default(), res_struct);
        let bi = BillingInfos::swico()
            .s1_builder()
            .add_unstructured(&res_builder)
            .build()?;
        assert_eq!(bi.unstructured().unwrap_or_default(), res_builder);
        Ok(())
    }
    #[rstest]
    #[case("//S1/10/10201409/11/190512/20/1400.000-53/30/106017086/31/180508/32/7.7/40/2:10;0:30")]
    #[case("//S1/10/10104/11/180228/30/395856455/31/180226180227/32/3.7:400.19;7.7:553.39;0:14/40/0:30")]
    #[case("//S1/10/4031202511/11/180107/20/61257233.4/30/105493567/32/8:49.82/33/2.5:14.85/40/0:30")]
    #[case(
        r"//S1/10/X.66711\/8824/11/200712/20/MW-2020-04/30/107978798/32/2.5:117.22/40/3:5;1.5:20;1:40;0:60"
    )]
    #[case(r"//S1/10/24073428/11/240729/20/145258\/Dépôt/30/112806097/31/240630240731/40/3:10;0:30")]
    fn from_str_valid(#[case] s: &str) -> anyhow::Result<()> {
        let msg = "Message au payeur";
        let mut a = msg.to_string();
        a.push_str(s);
        let b = BillingInfos::from_str(&a);
        assert!(&b.is_ok());
        if let Ok(a) = &b {
            let u = a.structured().unwrap_or_default();
            assert_eq!(u, s);
            let uns = a.unstructured().unwrap_or_default();
            assert_eq!(uns, msg);
        }
        println!("{:#?}", &b?.as_paragraph());
        Ok(())
    }
    #[rstest]
    fn doc_example() -> anyhow::Result<()> {
        let msg = "Invoice F248956-24RI for a new gaming chair / Gaming chair for Leon-Jaden Fanum Tax";
        let bi = BillingInfos::new().add_unstructured(msg).unwrap();
        assert_eq!(bi.unstructured().unwrap(), msg);
        assert!(bi.structured().is_none());
        println!("{:#?}", bi.as_paragraph());
        let other_msg = "Some invoice message";

        let bi = BillingInfos::swico();
        let mut bi = bi.s1_builder();
        let bi = bi.add_unstructured(other_msg).build();
        assert!(bi.is_ok());
        let bi = bi.unwrap();

        assert!(&bi.structured().is_none());
        assert_eq!(&bi.unstructured().unwrap(), other_msg);

        let bi = bi.add_unstructured(msg);
        assert_ne!(bi.unwrap().unstructured().unwrap(), other_msg);

        let long_msg = msg.repeat(10);
        let bi = BillingInfos::new().add_unstructured(long_msg);
        assert!(bi.is_err());
        Ok(())
    }
    #[rstest]
    fn error_handling() {
        let s = r"//S1/10/24073428/11/240729/20/145258\/Dépôt/30/112806097/31/240630240731/40/3:10;0:30";
        let bi: Result<BillingInfos, BillingInfoError> = s.parse();
        assert!(bi.is_ok());
    }
}
