// A program that implements transaction storing and payouts for Fluidity Money

#![cfg(all(target_arch = "bpf", not(feature = "exclude_entrypoint")))]

use {
    solana_program::{
        account_info::AccountInfo,
        entrypoint,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
};

mod processor;
mod state;
mod math;
mod error;

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process(program_id, accounts, instruction_data)?;
    Ok(())
}
