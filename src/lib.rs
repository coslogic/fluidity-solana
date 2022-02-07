// A program that implements transaction storing and payouts for Fluidity Money

#![cfg(all(target_arch = "bpf", not(feature = "exclude_entrypoint")))]

use {
    solana_program::{
        account_info::AccountInfo,
        entrypoint,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        declare_id,
    },
};

pub mod instruction;
pub mod processor;
mod state;
mod math;
mod error;

// declare the pubkey of the program
declare_id!("GjRwsHMgCAX2QUrw64tyT9RQhqm28fmntNAjgxoaTztU");

// pass entrypoint through to processor
entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process(program_id, accounts, instruction_data)?;
    Ok(())
}
