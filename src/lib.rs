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

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match processor::process(program_id, accounts, instruction_data) {
        Err(error) => Err(error),
        _ => Ok(()),
    }
}
