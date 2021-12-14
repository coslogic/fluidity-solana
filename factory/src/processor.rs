// Fluidity smart contract state processor

use crate::{
    state::{Obligation, Reserve},
};

use {
    std::str::FromStr,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        instruction::{AccountMeta, Instruction},
        log::sol_log_compute_units,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction, system_program,
        program_pack::{IsInitialized, Pack},
    },
    spl_token,
};

// enum for processes executable by fluidity smart contract
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
enum FluidityInstruction {
    // wrap fluid token
    Wrap(u64, String, u8),
    // unwrap fluid token
    Unwrap(u64, String, u8),
    // payout two accounts
    Payout (u64),
    // initialise solend obligation account
    InitSolendObligation (u64, u64, String, u8),
    LogTVL,
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

    BorrowObtaigationLiquidity,

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

    // 15
    /// Combines WithdrawObligationCollateral and RedeemReserveCollateral
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source withdraw reserve collateral supply SPL Token account.
    ///   1. `[writable]` Destination collateral token account.
    ///                     Minted by withdraw reserve collateral mint.
    ///   2. `[writable]` Withdraw reserve account - refreshed.
    ///   3. `[writable]` Obligation account - refreshed.
    ///   4. `[]` Lending market account.
    ///   5. `[]` Derived lending market authority.
    ///   6. `[writable]` User liquidity token account.
    ///   7. `[writable]` Reserve collateral SPL Token mint.
    ///   8. `[writable]` Reserve liquidity supply SPL Token account.
    ///   9. `[signer]` Obligation owner
    ///   10 `[signer]` User transfer authority ($authority).
    ///   11. `[]` Clock sysvar.
    ///   12. `[]` Token program id.
    WithdrawObligationCollateralAndRedeemReserveCollateral {
        /// liquidity_amount is the amount of collateral tokens to withdraw
        collateral_amount: u64,
    },
}

fn wrap(accounts: &[AccountInfo], amount: u64, seed: String, bump: u8) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let pda_token_account = next_account_info(accounts_iter)?;
    let fluidity_account = next_account_info(accounts_iter)?;
    let solend_program = next_account_info(accounts_iter)?;
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

    // make sure the base and fluid token match
    check_mints_and_pda(*token_mint.key, *fluidity_mint.key, *pda_account.key);

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
        &[&[&seed.as_bytes(), &[bump]]],
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
        &[&[&seed.as_bytes(), &[bump]]],
    )?;

    Ok(())
}

fn unwrap(accounts: &[AccountInfo], amount: u64, seed: String, bump: u8) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let pda_token_account = next_account_info(accounts_iter)?;
    let fluidity_account = next_account_info(accounts_iter)?;
    let solend_program = next_account_info(accounts_iter)?;
    let destination_collateral_info = next_account_info(accounts_iter)?;
    let user_collateral_info = next_account_info(accounts_iter)?;
    let withdraw_reserve_info = next_account_info(accounts_iter)?;
    let obligation_info = next_account_info(accounts_iter)?;
    let lending_market_info = next_account_info(accounts_iter)?;
    let lending_market_authority_info = next_account_info(accounts_iter)?;
    let reserve_collateral_mint_info = next_account_info(accounts_iter)?;
    let reserve_liquidity_supply_info = next_account_info(accounts_iter)?;
    let withdraw_pyth_price_info = next_account_info(accounts_iter)?;
    let withdraw_switchboard_feed_info = next_account_info(accounts_iter)?;
    let clock_info = next_account_info(accounts_iter)?;

    // make sure the base and fluid token match
    check_mints_and_pda(*token_mint.key, *fluidity_mint.key, *pda_account.key);

    invoke(
        &spl_token::instruction::burn(
            &token_program.key,
            &fluidity_account.key,
            &fluidity_mint.key,
            &sender.key,
            &[&sender.key],
            amount,
        )
        .unwrap(),
        &[
            fluidity_account.clone(),
            fluidity_mint.clone(),
            sender.clone(),
        ],
    )?;

    invoke(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::RefreshReserve,
            vec![
                AccountMeta::new(*withdraw_reserve_info.key, false),
                AccountMeta::new_readonly(*withdraw_pyth_price_info.key, false),
                AccountMeta::new_readonly(*withdraw_switchboard_feed_info.key, false),
                AccountMeta::new_readonly(*clock_info.key, false),
            ],
        ),
        &[
            withdraw_reserve_info.clone(),
            withdraw_pyth_price_info.clone(),
            withdraw_switchboard_feed_info.clone(),
            clock_info.clone(),
            solend_program.clone(),
        ],
    )?;

    invoke(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::RefreshObligation,
            vec![
                AccountMeta::new(*obligation_info.key, false),
                AccountMeta::new_readonly(*clock_info.key, false),
                AccountMeta::new(*withdraw_reserve_info.key, false),
            ],
        ),
        &[
            obligation_info.clone(),
            clock_info.clone(),
            withdraw_reserve_info.clone(),
            solend_program.clone(),
        ],
    )?;

    invoke_signed(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::WithdrawObligationCollateralAndRedeemReserveCollateral {
                collateral_amount: amount,
            },
            vec![
                AccountMeta::new(*destination_collateral_info.key, false),
                AccountMeta::new(*user_collateral_info.key, false),
                AccountMeta::new(*withdraw_reserve_info.key, false),
                AccountMeta::new(*obligation_info.key, false),
                AccountMeta::new(*lending_market_info.key, false),
                AccountMeta::new_readonly(*lending_market_authority_info.key, false),
                AccountMeta::new(*pda_token_account.key, false),
                AccountMeta::new(*reserve_collateral_mint_info.key, false),
                AccountMeta::new(*reserve_liquidity_supply_info.key, false),
                AccountMeta::new(*pda_account.key, true),
                AccountMeta::new(*pda_account.key, true),
                AccountMeta::new_readonly(*clock_info.key, false),
                AccountMeta::new_readonly(*token_program.key, false),
            ],
        ),
        &[
            destination_collateral_info.clone(),
            user_collateral_info.clone(),
            withdraw_reserve_info.clone(),
            obligation_info.clone(),
            lending_market_info.clone(),
            lending_market_authority_info.clone(),
            pda_token_account.clone(),
            reserve_collateral_mint_info.clone(),
            reserve_liquidity_supply_info.clone(),
            pda_account.clone(),
            clock_info.clone(),
            token_program.clone(),
            solend_program.clone(),
        ],
        &[&[&seed.as_bytes(), &[bump]]],
    )?;

    invoke_signed(
        &spl_token::instruction::transfer(
            &token_program.key,
            &pda_token_account.key,
            &token_account.key,
            &pda_account.key,
            &[&pda_account.key],
            amount,
        )
        .unwrap(),
        &[
            pda_token_account.clone(),
            token_account.clone(),
            pda_account.clone(),
            token_program.clone(),
        ],
        &[&[&seed.as_bytes(), &[bump]]],
    )?;

    Ok(())
}

fn check_mints_and_pda(token_mint: Pubkey, fluid_mint: Pubkey, pda: Pubkey) {
    let TOKEN_TO_FLUID_MAP = [
        (
            Pubkey::from_str("zVzi5VAf4qMEwzv7NXECVx5v2pQ7xnqVVjCXZwS9XzA").unwrap(),
            Pubkey::from_str("4NYFTmvWY1EjqzEfr7t41ey9HkoYV133CWfQq18qCXAE").unwrap(),
            Pubkey::from_str("GUmgGM3MQvtHM3B7vhKxfvvTM8Rvp5aF2js4ZjcH2ZoR").unwrap(),
        ),
        (
            Pubkey::from_str("Bp2nLuamFZndE7gztA1iPsNVhdJeg9xfKdq7KmvjpGoP").unwrap(),
            Pubkey::from_str("EE6KL24UqgerwbjrWqU3Cm8V4kUbCTuvhyTJqmYJKqJj").unwrap(),
            Pubkey::from_str("CgfqRZmjUsLaUCgrdrBmotwjDQxFWWxhkBHwCdR6kPQm").unwrap()
        ),
    ];

    if let Some((t, m ,p)) = TOKEN_TO_FLUID_MAP.iter().filter(|x| x.0 == token_mint).nth(0) {
        if (t, m, p) != (&token_mint, &fluid_mint, &pda) {
            panic!("invalid token pair!");
        }
    } else {
        panic!("unkown token!");
    }

}

fn payout(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let token_program = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let payout_account_a = next_account_info(accounts_iter)?;
    let payout_account_b = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;

    if !(payer.is_signer && payer.key == &Pubkey::from_str("sohTpNitFg3WZeEcbrMunnwoZJWP4t8yisPB5o3DGD5").unwrap()) {
        panic!("bad payout authority!");
    }

    invoke_signed(
        &spl_token::instruction::mint_to(
            &token_program.key,
            &fluidity_mint.key,
            &payout_account_a.key,
            &pda_account.key,
            &[&pda_account.key],
            amount,
        ).unwrap(),
        &[fluidity_mint.clone(), payout_account_a.clone(), pda_account.clone(), token_program.clone()],
        &[&[&b"FLU: MINT ACCOUNT"[..], &[255]]],
    )?;

    invoke_signed(
        &spl_token::instruction::mint_to(
            &token_program.key,
            &fluidity_mint.key,
            &payout_account_b.key,
            &pda_account.key,
            &[&pda_account.key],
            amount,
        ).unwrap(),
        &[fluidity_mint.clone(), payout_account_b.clone(), pda_account.clone(), token_program.clone()],
        &[&[&b"FLU: MINT ACCOUNT"[..], &[255]]],
    )?;

    Ok(())
}

fn init_solend_obligation(
    accounts: &[AccountInfo],
    obligation_lamports: u64,
    obligation_size: u64,
    seed: String,
    bump: u8,
) -> ProgramResult {
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
        &[&[&seed.as_bytes(), &[bump]]],
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
        &[&[&seed.as_bytes(), &[bump]]],
    )?;

    Ok(())
}

pub fn log_tvl(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let obligation_info = next_account_info(accounts_iter)?;
    let reserve_info = next_account_info(accounts_iter)?;

    let obligation = Obligation::unpack(&obligation_info.data.borrow())?;
    let reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    msg!("{:?}", &obligation.deposits);
    msg!("{:?}", reserve.collateral_exchange_rate()?);

    Ok(())
}

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = FluidityInstruction::try_from_slice(input)?;
    match instruction {
        FluidityInstruction::Wrap(amount, seed, bump) => {
            return wrap(&accounts, amount, seed, bump);
        }
        FluidityInstruction::Unwrap(amount, seed, bump) => {
            return unwrap(&accounts, amount, seed, bump);
        }
        FluidityInstruction::Payout (amount) => {
            return payout(&accounts, amount);
        }
        FluidityInstruction::InitSolendObligation(obligation_lamports, obligation_size, seed, bump) => {
            return init_solend_obligation(&accounts, obligation_lamports, obligation_size, seed, bump);
        }
        FluidityInstruction::LogTVL => {
            return log_tvl(&accounts);
        }
    };
}
