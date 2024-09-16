//! Translations of all the QRbill heading labels into the four allowed
//! languages.

#[derive(Debug, Clone, Copy)]
/// The languages allowed in QRbills
pub enum Language {
    German,
    English,
    French,
    Italian,
}

/// A collection of QRbill labels in a single language
pub struct Labels {
    pub payment_part:           &'static str,
    pub payable_to:             &'static str,
    pub reference:              &'static str,
    pub additional_information: &'static str,
    pub currency:               &'static str,
    pub amount:                 &'static str,
    pub receipt:                &'static str,
    pub acceptance_point:       &'static str,
    pub payable_by:             &'static str,
    pub payable_by_extended:    &'static str,
    pub payable_by_date:        &'static str,
}

impl Labels {
    /// Create set of all known labels in the given language
    pub fn for_language(language: Language) -> Labels {
        Labels {
            payment_part:           PAYMENT_PART           .to(language),
            payable_to:             PAYABLE_TO             .to(language),
            reference:              REFERENCE              .to(language),
            additional_information: ADDITIONAL_INFORMATION .to(language),
            currency:               CURRENCY               .to(language),
            amount:                 AMOUNT                 .to(language),
            receipt:                RECEIPT                .to(language),
            acceptance_point:       ACCEPTANCE_POINT       .to(language),
            payable_by:             PAYABLE_BY             .to(language),
            payable_by_extended:    PAYABLE_BY_EXTENDED    .to(language),
            payable_by_date:        PAYABLE_BY_DATE        .to(language),
        }
    }
}

// Annex D: Multilingual headings
pub const PAYMENT_PART: Translation = Translation {
    en: "Payment part",
    de: "Zahlteil",
    fr: "Section paiement",
    it: "Sezione pagamento",
};

pub const PAYABLE_TO: Translation = Translation {
    en: "Account / Payable to",
    de: "Konto / Zahlbar an",
    fr: "Compte / Payable à",
    it: "Conto / Pagabile a",
};

pub const REFERENCE: Translation = Translation {
    en: "Reference",
    de: "Referenz",
    fr: "Référence",
    it: "Riferimento",
};

pub const ADDITIONAL_INFORMATION: Translation = Translation {
    en: "Additional information",
    de: "Zusätzliche Informationen",
    fr: "Informations supplémentaires",
    it: "Informazioni supplementari",
};

pub const CURRENCY: Translation = Translation {
    en: "Currency",
    de: "Währung",
    fr: "Monnaie",
    it: "Valuta",
};

pub const AMOUNT: Translation = Translation {
    en: "Amount",
    de: "Betrag",
    fr: "Montant",
    it: "Importo",
};

pub const RECEIPT: Translation = Translation {
    en: "Receipt",
    de: "Empfangsschein",
    fr: "Récépissé",
    it: "Ricevuta",
};

pub const ACCEPTANCE_POINT: Translation = Translation {
    en: "Acceptance point",
    de: "Annahmestelle",
    fr: "Point de dépôt",
    it: "Punto di accettazione",
};

pub const PAYABLE_BY: Translation = Translation {
    en: "Payable by",
    de: "Zahlbar durch",
    fr: "Payable par",
    it: "Pagabile da",
};

pub const PAYABLE_BY_EXTENDED: Translation = Translation {
    en: "Payable by (name/address)",
    de: "Zahlbar durch (Name/Adresse)",
    fr: "Payable par (nom/adresse)",
    it: "Pagabile da (nome/indirizzo)",
};

// TODO WHAT IS THIS? CAN'T FIND IT ANYWHERE IN THE STANDARD!
// The extra ending space allows to differentiate from the other: "Payable by" above.
pub const PAYABLE_BY_DATE: Translation = Translation {
    en: "Payable by",
    de: "Zahlbar bis",
    fr: "Payable jusqu’au",
    it: "Pagabile fino al",
};

pub struct Translation {
    en: &'static str,
    de: &'static str,
    fr: &'static str,
    it: &'static str,
}

impl Translation {

    fn to(&self, language: Language) -> &'static str {
        use Language::*;
        match language {
            German  => self.de,
            English => self.en,
            French  => self.fr,
            Italian => self.it,
        }
    }
}
