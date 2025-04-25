#[derive(Debug, thiserror::Error)]
pub enum SwicoError {
    #[error("Maximum 140 characters authorized for BillingInfos, found: {0:?}")]
    TooLong(usize),
    #[error("Could not parse Swico string: {0}")]
    FromSyntaxParser(#[from] super::parser::SyntaxParserError),
    #[error("Could not validate Swico syntax")]
    FromSyntaxValidator(#[from] super::syntax::SyntaxValidatorError),
}
