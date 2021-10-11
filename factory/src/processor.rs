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
        system_program,
        system_instruction,
        instruction::{AccountMeta, Instruction},
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
    // initialise solend obligation account
    InitSolendObligation (u64, u64),
}

#[derive(BorshSerialize)]
enum LendingInstruction {
    InitLendingMarket,

    SetLendingMarketOwner,

    InitReserve,

    RefreshReserve,

    DepositReserveLiquidity,

    RedeemReserveCollateral,

    // 6
    /// Initializes a new lending market obligation.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Obligation account - uninitialized.
    ///   1. `[]` Lending market account.
    ///   2. `[signer]` Obligation owner.
    ///   3. `[]` Clock sysvar.
    ///   4. `[]` Rent sysvar.
    ///   5. `[]` Token program id.
    InitObligation,

    RefreshObligation,

    DepositObligationCollateral,

    WithdrawObligationCollateral,

    BorrowObligationLiquidity,

    RepayObligationLiquidity,

    LiquidateObligation,

    FlashLoan,

    // 14
    /// Combines DepositReserveLiquidity and DepositObligationCollateral
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account.
    ///                     $authority can transfer $liquidity_amount.
    ///   1. `[writable]` Destination collateral token account.
    ///   2. `[writable]` Reserve account.
    ///   3. `[writable]` Reserve liquidity supply SPL Token account.
    ///   4. `[writable]` Reserve collateral SPL Token mint.
    ///   5. `[]` Lending market account.
    ///   6. `[]` Derived lending market authority.
    ///   7. `[writable]` Destination deposit reserve collateral supply SPL Token account.
    ///   8. `[writable]` Obligation account.
    ///   9. `[signer]` Obligation owner.
    ///   10 `[]` Pyth price oracle account.
    ///   11 `[]` Switchboard price feed oracle account.
    ///   12 `[signer]` User transfer authority ($authority).
    ///   13 `[]` Clock sysvar.
    ///   14 `[]` Token program id.
    DepositReserveLiquidityAndObligationCollateral {
        /// Amount of liquidity to deposit in exchange
        liquidity_amount: u64,
    },
}

fn wrap(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let fluidity_account = next_account_info(accounts_iter)?;
    let solend_program = next_account_info(accounts_iter)?;
    let pda_token_account = next_account_info(accounts_iter)?;
    let user_collateral_info = next_account_info(accounts_iter)?;
    let reserve_info = next_account_info(accounts_iter)?;
    let reserve_liquidity_supply_info = next_account_info(accounts_iter)?;
    let reserve_collateral_mint_info = next_account_info(accounts_iter)?;
    let lending_market_info = next_account_info(accounts_iter)?;
    let lending_market_authority_info = next_account_info(accounts_iter)?;
    let destination_collateral_info = next_account_info(accounts_iter)?;
    let obligation_info = next_account_info(accounts_iter)?;
    let pyth_price_info = next_account_info(accounts_iter)?;
    let switchboard_feed_info = next_account_info(accounts_iter)?;
    let clock_info = next_account_info(accounts_iter)?;

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

    invoke(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::RefreshReserve,
            vec![
                AccountMeta::new(*reserve_info.key, false),
                AccountMeta::new_readonly(*pyth_price_info.key, false),
                AccountMeta::new_readonly(*switchboard_feed_info.key, false),
                AccountMeta::new_readonly(*clock_info.key, false),
            ], 
        ),
        &[reserve_info.clone(), pyth_price_info.clone(), switchboard_feed_info.clone(), clock_info.clone(), solend_program.clone()]
    )?;

    invoke_signed(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::DepositReserveLiquidityAndObligationCollateral{liquidity_amount: amount},
            vec![
                AccountMeta::new(*pda_token_account.key, false),
                AccountMeta::new(*user_collateral_info.key, false),
                AccountMeta::new(*reserve_info.key, false),
                AccountMeta::new(*reserve_liquidity_supply_info.key, false),
                AccountMeta::new(*reserve_collateral_mint_info.key, false),
                AccountMeta::new(*lending_market_info.key, false),
                AccountMeta::new_readonly(*lending_market_authority_info.key, false),
                AccountMeta::new(*destination_collateral_info.key, false),
                AccountMeta::new(*obligation_info.key, false),
                AccountMeta::new(*pda_account.key, true),
                AccountMeta::new_readonly(*pyth_price_info.key, false),
                AccountMeta::new_readonly(*switchboard_feed_info.key, false),
                AccountMeta::new(*pda_account.key, true),
                AccountMeta::new_readonly(*clock_info.key, false),
                AccountMeta::new_readonly(*token_program.key, false),
            ]
        ),
        &[
            pda_token_account.clone(), user_collateral_info.clone(), reserve_info.clone(), reserve_liquidity_supply_info.clone(),
            reserve_collateral_mint_info.clone(), lending_market_info.clone(), lending_market_authority_info.clone(),
            destination_collateral_info.clone(), obligation_info.clone(), pyth_price_info.clone(), pda_account.clone(),
            switchboard_feed_info.clone(), sender.clone(), clock_info.clone(), token_program.clone(),
            solend_program.clone()
        ],
        &[&[&b"FLU: MINT ACCOUNT"[..], &[255]]],
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

fn init_solend_obligation(accounts: &[AccountInfo], obligation_lamports: u64, obligation_size: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let solend_program_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    // init obligation infos
    let obligation_info = next_account_info(accounts_iter)?;
    let lending_market_info = next_account_info(accounts_iter)?;
    let obligation_owner_info = next_account_info(accounts_iter)?;
    let clock_info = next_account_info(accounts_iter)?;
    let rent_info = next_account_info(accounts_iter)?;
    let token_program_id = next_account_info(accounts_iter)?;

    invoke_signed(
        &system_instruction::create_account_with_seed(
            &payer.key,
            &obligation_info.key,
            &obligation_owner_info.key,
            &lending_market_info.key.to_string()[0..32],
            obligation_lamports,
            obligation_size,
            &solend_program_info.key,
        ),
        &[payer.clone(), obligation_info.clone(), obligation_owner_info.clone(), lending_market_info.clone(), solend_program_info.clone(), system_program_info.clone()],
        &[&[&b"FLU: MINT ACCOUNT"[..], &[255]]],
    )?;

    invoke_signed(
        &Instruction::new_with_borsh(
            *solend_program_info.key,
            &LendingInstruction::InitObligation,
            vec![
                AccountMeta::new(*obligation_info.key, false),
                AccountMeta::new(*lending_market_info.key, false),
                AccountMeta::new(*obligation_owner_info.key, true),
                AccountMeta::new_readonly(*clock_info.key, false),
                AccountMeta::new_readonly(*rent_info.key, false),
                AccountMeta::new_readonly(*token_program_id.key, false)
            ]
        ),
        &[obligation_info.clone(), lending_market_info.clone(), obligation_owner_info.clone(),
          clock_info.clone(), rent_info.clone(), token_program_id.clone(), solend_program_info.clone()],
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
        FluidityInstruction::InitSolendObligation (obligation_lamports, obligation_size) => {
            return init_solend_obligation(&accounts, obligation_lamports, obligation_size);
        }
    };
}
