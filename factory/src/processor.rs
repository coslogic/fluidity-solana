// Fluidity smart contract state processor

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        program::{invoke_signed, invoke},
        pubkey::Pubkey,
        log::sol_log_compute_units,
        system_instruction::transfer,
        msg,
    },
    spl_token,
};

// enum for processes executable by fluidity smart contract
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
enum FluidityInstruction {
    // wrap fluid token
    Wrap (u64),
    // unwrap fluid token
    Unwrap (u64),
}

fn wrap(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pda_token_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let fluidity_account = next_account_info(accounts_iter)?;

    invoke(
        &spl_token::instruction::transfer(
            &token_program.key,
            &token_account.key,
            &pda_token_account.key,
            &sender.key,
            &[&sender.key],
            amount,
        ).unwrap(),
        &[token_account.clone(), pda_token_account.clone(), sender.clone(), token_program.clone()]
    )?;

    invoke_signed(
        &spl_token::instruction::mint_to(
            &token_program.key,
            &fluidity_mint.key,
            &fluidity_account.key,
            &pda_account.key,
            &[&pda_account.key],
            amount,
        ).unwrap(),
        &[fluidity_mint.clone(), fluidity_account.clone(), pda_account.clone(), token_program.clone()],
        &[&[&b"FLU: MINT ACCOUNT"[..], &[255]]],
    )?;

    Ok(())
}

fn unwrap(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pda_token_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let fluidity_account = next_account_info(accounts_iter)?;

    invoke(
        &spl_token::instruction::burn(
            &token_program.key,
            &fluidity_account.key,
            &fluidity_mint.key,
            &sender.key,
            &[&sender.key],
            amount,
        ).unwrap(),
        &[fluidity_account.clone(), fluidity_mint.clone(), sender.clone()]
    )?;

    invoke_signed(
        &spl_token::instruction::transfer(
            &token_program.key,
            &pda_account.key,
            &token_account.key,
            &pda_account.key,
            &[&pda_account.key],
            amount,
        ).unwrap(),
        &[pda_account.clone(), token_account.clone(), token_program.clone()],
        &[&[&b"FLU: MINT ACCOUNT"[..], &[255]]],
    )?;

    Ok(())
}

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = FluidityInstruction::try_from_slice(input)?;
    match instruction {
        FluidityInstruction::Wrap (amount) => {
            return wrap(&accounts, amount);
        }
        FluidityInstruction::Unwrap (amount) => {
            return unwrap(&accounts, amount);
        }
    };
}
