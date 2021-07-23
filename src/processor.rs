// Fluidity smart contract state processor

use {
    borsh::{BorshDeserialize},
    std::convert::TryInto,
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
    EnlistTxn ([u8; 64], Pubkey, Pubkey),
    // flush transactions from list
    FlushTxns,
}

/* this is not needed since it's more efficient to just directly edit the values of bytes,
 * but it serves as a useful reminder of the data structure.
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct PoolAccount {
    pub txns: Vec<([u8; 64], Pubkey, Pubkey)>,
}
*/

fn enlist(program_id: &Pubkey, accounts: &[AccountInfo], sig: &[u8; 64], sender: Pubkey, receiver: Pubkey) -> ProgramResult {
    // get first account from account infos, this should be the pool account.
    let accounts_iter = &mut accounts.iter();
    let pool_account = next_account_info(accounts_iter)?;

    // check that pool account is owned by program
    if pool_account.owner != program_id {
        msg!("Pool account owned by wrong program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = pool_account.try_borrow_mut_data().unwrap();
    let mut count = u32::from_le_bytes(data[0..4].try_into().expect("Bad data slice"));
    let mut start = count as usize * 128 + 4;
    for i in 0..64 {
        data[i + start] = sig[i];
    }
    start += 64;
    let pk_bytes = sender.to_bytes();
    for i in 0..32 {
        data[i + start] = pk_bytes[i];
    }
    start += 32;
    let pk_bytes = receiver.to_bytes();
    for i in 0..32 {
        data[i + start] = pk_bytes[i];
    }
    count += 1;
    let count_bytes = count.to_le_bytes();
    for i in 0..4 {
        data[i] = count_bytes[i];
    }

    Ok(())
}

fn flush(accounts: &[AccountInfo]) -> ProgramResult {
    // get pool account from account infos
    let accounts_iter = &mut accounts.iter();
    let pool_account = next_account_info(accounts_iter)?;

    // do something with the txns

    // zero the data
    let mut data = pool_account.try_borrow_mut_data().unwrap();
    for i in 0..4 {
        data[i] = 0;
    }
    Ok(())
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = FluidityInstruction::try_from_slice(input)?;
    match instruction {
        FluidityInstruction::EnlistTxn (sig, sender, receiver) => {
            return enlist(&program_id, &accounts, &sig, sender, receiver);
        }
        FluidityInstruction::FlushTxns => {
            return flush(&accounts);
        }
    };
}
