pub mod state;
pub mod processor;
pub mod instruction;
pub mod error;
pub mod utils;
use crate::{
    processor::Processor,
    error::TokenError
};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::PrintProgramError,
};
pub const PREFIX: &str = "token";
pub const PREFIX_ASSOCIATED: &str = "pda_associated";

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process(program_id, accounts, input) {
        // catch the error so we can print it
        error.print::<TokenError>();
        return Err(error);
    }
    Ok(())
}