use anchor_lang::prelude::*;

#[error_code]
pub enum LendingError {
    #[msg("Instruction Unpack Error")]
    InstructionUnpackError,
}