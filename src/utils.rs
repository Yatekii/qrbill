use crate::Error;
use crate::Reference;
use crate::{Iban, IbanLike};
use crate::{QR_IID_END, QR_IID_START};

#[derive(Debug)]
pub enum IbanType {
    Qriid,
    Iid,
}
impl IbanType {
    pub fn try_matching_reference(&self, reference: &Reference, iban_str: &str) -> Result<(), Error> {
        //For QRIID, only ESR reference is allowed
        match self {
            Self::Qriid => match reference {
                Reference::Qrr(_) => Ok(()),
                _ => Err(Error::InvalidQriid(iban_str.into())),
            },
            //For IID, SCOR or NON are allowed, ESR is prohibited
            Self::Iid => match reference {
                Reference::Qrr(_) => Err(Error::InvalidIid(iban_str.into())),
                _ => Ok(()),
            },
        }
    }
}
pub trait IbanKind {
    fn kind<'a>(&self) -> Result<&'a IbanType, Error>;
}
impl IbanKind for Iban {
    fn kind<'a>(&self) -> Result<&'a IbanType, Error> {
        let iid: usize = self.electronic_str()[4..9]
            .parse()
            .expect("This is a bug, please report it");
        if (QR_IID_START..=QR_IID_END).contains(&iid) {
            Ok(&IbanType::Qriid)
        } else {
            Ok(&IbanType::Iid)
        }
    }
}
