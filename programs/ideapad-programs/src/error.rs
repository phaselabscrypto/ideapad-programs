use anchor_lang::prelude::*;

#[error_code]
pub enum IdeaPadErrorCode {
    #[msg("Numerical Overflow!")]
    NumericalOverflow,
}
