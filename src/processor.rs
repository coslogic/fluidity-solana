// Fluidity smart contract state processor

use {
    borsh::{BorshDeserialize, BorshSerialize},
    std::convert::TryInto,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        program::invoke_signed,
        sysvar::{ Sysvar, clock::Clock},
        pubkey::Pubkey,
        log::sol_log_compute_units,
        instruction::{Instruction, AccountMeta},
        msg,
    },
    spl_token::instruction::mint_to_checked,
};

// enum for processes executable by fluidity smart contract
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
enum FluidityInstruction {
    // enlist transaction with signature (as string since BorshSerialize is not implemented for Signature), sender, and receiver
    EnlistTxn ([u8; 64], Pubkey, Pubkey),
    // wrap fluid token
    Wrap (u64),
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

    let mut data = pool_account.try_borrow_mut_data()?;
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
    let program_account = next_account_info(accounts_iter)?;

    // do something with the txns
    let mut data = pool_account.try_borrow_mut_data()?;

    let mut count: u32 = 0;
    // zero the data and also get the count
    for i in 0..4 {
        count += data[i] as u32 * 2_u32.pow(i as u32 * 8);
        data[i] = 0;
    }


    //let mut temp = [0 as u8; 64];
    /*
    for i in 0..count {
        //for i in 0 .. 128 {
        //    temp[i] = data[count as usize * 128 + 4 + i];
        //}
        //msg!("{:?}", temp);
    }*/
    let clock = Clock::get()?;
    msg!("{}", clock.slot);

    Ok(())
}

fn wrap(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let mut sender_lamports = sender_accountinfo.try_borrow_mut_lamports()?;
    //**pool_lamports -= amount;
    let (auth_pubkey, bump_seed) = Pubkey::find_program_address(&[b"FLU: MINT ACCOUNT"], &program_accountinfo.key);

    invoke_signed(
        &mint_to_checked (
            &token_program.key,
            &mint.key,
            &sender.key,
            &pda_account.key,
            &[&pda_account.key],
            amount * 10_u64.pow(9),
            9
        ).unwrap(),
        &[mint_accountinfo.clone(), sender_accountinfo.clone(), pda_account.clone()],
        &[&[&b"FLU: MINT ACCOUNT"[..], &[bump_seed]]],
    )?;

    Ok(())
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = FluidityInstruction::try_from_slice(input)?;
    match instruction {
        FluidityInstruction::EnlistTxn (sig, sender, receiver) => {
            return enlist(&program_id, &accounts, &sig, sender, receiver);
        }
        FluidityInstruction::Wrap (amount) => {
            return wrap(&accounts, amount);
        }
        FluidityInstruction::FlushTxns => {
            return flush(&accounts);
        }
    };
}
