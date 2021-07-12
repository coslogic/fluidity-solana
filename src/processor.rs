// Fluidity smart contract state processor

use {
    borsh::{BorshDeserialize},
    serde_derive::{Deserialize, Serialize},
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        msg,
    }
};

// enum for processes executable by fluidity smart contract
#[derive(Serialize, BorshDeserialize, Debug, PartialEq, Clone)]
enum FluidityInstruction {
    EnlistTxn,
    FlushTxns,
}

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = FluidityInstruction::try_from_slice(input)?;
        match instruction {
            FluidityInstruction::FlushTxns => {
                msg!("Flush transactions!");
            }
            FluidityInstruction::EnlistTxn => {
                msg!("Enlist transaction!");
            }
        };
        Ok(())
    }
}
