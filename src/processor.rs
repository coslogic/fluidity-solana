// Fluidity smart contract state processor

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
        msg,
    }
};

// enum for processes executable by fluidity smart contract
#[derive(BorshDeserialize, Debug, PartialEq, Clone)]
enum FluidityInstruction {
    // enlist transaction with signature (as string since BorshSerialize is not implemented for Signature), sender, and receiver
    EnlistTxn (String, Pubkey, Pubkey),
    // flush transactions from list
    FlushTxns,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct PoolAccount {
    pub txns: Vec<(String, Pubkey, Pubkey)>,
}

fn enlist(program_id: &Pubkey, accounts: &[AccountInfo], sig: String, sender: Pubkey, receiver: Pubkey) -> ProgramResult {
    // get first account from account infos, this should be the pool account.
    let accounts_iter = &mut accounts.iter();
    let pool_account = next_account_info(accounts_iter)?;

    // check that pool account is owned by program
    if pool_account.owner != program_id {
        msg!("Pool account owned by wrong program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = pool_account.try_borrow_mut_data().unwrap();
    let mut pool = PoolAccount::deserialize(&mut &data[..]).unwrap();

    msg!("{:?}", pool);
    pool.txns.push((sig, sender, receiver));
    pool.serialize(&mut &mut data[..])?;
    Ok(())
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = FluidityInstruction::try_from_slice(input)?;
    match instruction {
        FluidityInstruction::EnlistTxn (sig, sender, receiver) => {
            return enlist(&program_id, &accounts, sig, sender, receiver);
        }
        FluidityInstruction::FlushTxns => {
            msg!("Flush transactions!");
        }
    };
    Ok(())
}
