// Fluidity smart contract state processor

use crate::{
    state::{Obligation, Reserve},
    math::*,
    instruction::*,
};

use {
    std::{str::FromStr, convert::TryFrom},
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        instruction::{AccountMeta, Instruction},
        log::sol_log_compute_units,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError, pubkey::Pubkey,
        system_instruction, system_program,
        program_pack::{IsInitialized, Pack},
    },
    spl_token,
};

// struct defining fludity data account
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Clone)]
pub struct FluidityData {
    deposited_liquidity: u64,
    token_mint: Pubkey,
    fluid_mint: Pubkey,
    pda: Pubkey,
}

fn wrap(accounts: &[AccountInfo], amount: u64, seed: String, bump: u8) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let fluidity_data_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
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

    // create seed strings following format
    let pda_seed = format!("FLU:{}_OBLIGATION", seed);
    let data_seed = format!("FLU:{}_DATA", seed);

    // check that data account is derived from pda
    if fluidity_data_account.key !=
        &Pubkey::create_with_seed(
            pda_account.key,
            &data_seed,
            fluidity_data_account.owner
        ).unwrap() {
            panic!("bad data account");
    }

    // check mints
    check_mints_and_pda(&fluidity_data_account, *token_mint.key, *fluidity_mint.key, *pda_account.key);

    // refresh reserve
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
        &[
            reserve_info.clone(), pyth_price_info.clone(), switchboard_feed_info.clone(),
            clock_info.clone(), solend_program.clone()
        ]
    )?;

    // deposit liquidity from user token account
    invoke(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::DepositReserveLiquidity{liquidity_amount: amount},
            vec![
                AccountMeta::new(*token_account.key, false),
                AccountMeta::new(*user_collateral_info.key, false),
                AccountMeta::new(*reserve_info.key, false),
                AccountMeta::new(*reserve_liquidity_supply_info.key, false),
                AccountMeta::new(*reserve_collateral_mint_info.key, false),
                AccountMeta::new(*lending_market_info.key, false),
                AccountMeta::new_readonly(*lending_market_authority_info.key, false),
                AccountMeta::new(*sender.key, true),
                AccountMeta::new_readonly(*clock_info.key, false),
                AccountMeta::new_readonly(*token_program.key, false),
            ],
        ),
        &[
            token_account.clone(), user_collateral_info.clone(), reserve_info.clone(),
            reserve_liquidity_supply_info.clone(), reserve_collateral_mint_info.clone(),
            lending_market_info.clone(), lending_market_authority_info.clone(),
            sender.clone(), clock_info.clone(), token_program.clone(),
        ],
    )?;

    // refresh reserve again
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
        &[
            reserve_info.clone(), pyth_price_info.clone(), switchboard_feed_info.clone(),
            clock_info.clone(), solend_program.clone()
        ]
    )?;

    // calculate collateral amount
    let reserve = Reserve::unpack(&reserve_info.data.borrow())?;
    let collateral_amount = reserve.collateral_exchange_rate()?.liquidity_to_collateral(amount)?;

    // deposit collateral into obligation
    invoke_signed(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::DepositObligationCollateral{collateral_amount},
            vec![
                AccountMeta::new(*user_collateral_info.key, false),
                AccountMeta::new(*destination_collateral_info.key, false),
                AccountMeta::new(*reserve_info.key, false),
                AccountMeta::new(*obligation_info.key, false),
                AccountMeta::new(*lending_market_info.key, false),
                AccountMeta::new(*pda_account.key, true),
                AccountMeta::new(*pda_account.key, true),
                AccountMeta::new_readonly(*clock_info.key, false),
                AccountMeta::new_readonly(*token_program.key, false),
            ]
        ),
        &[
            user_collateral_info.clone(), destination_collateral_info.clone(), reserve_info.clone(), obligation_info.clone(),
            lending_market_info.clone(), pda_account.clone(), clock_info.clone(), token_program.clone(),
        ],
        &[&[&pda_seed.as_bytes(), &[bump]]],
    )?;

    // mint fluid tokens to user account
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
        &[&[&pda_seed.as_bytes(), &[bump]]],
    )?;

    Ok(())
}

fn unwrap(accounts: &[AccountInfo], amount: u64, seed: String, bump: u8) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let fluidity_data_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
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

    // create seed strings from provided token
    let pda_seed = format!("FLU:{}_OBLIGATION", seed);
    let data_seed = format!("FLU:{}_DATA", seed);

    // check that data account is derived from pda
    if fluidity_data_account.key !=
        &Pubkey::create_with_seed(
            pda_account.key,
            &data_seed,
            fluidity_data_account.owner
        ).unwrap() {
            panic!("bad data account");
    }

    check_mints_and_pda(&fluidity_data_account, *token_mint.key, *fluidity_mint.key, *pda_account.key);

    // burn fluid tokens
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

    // refresh reserve
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

    // refresh obligation
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

    // calculate collateral amount from refreshed reserve
    let reserve = Reserve::unpack(&withdraw_reserve_info.data.borrow())?;
    let collateral_amount = reserve.collateral_exchange_rate()?.liquidity_to_collateral(amount)?;

    // withdraw from solend to the user's token account
    invoke_signed(
        &Instruction::new_with_borsh(
            *solend_program.key,
            &LendingInstruction::WithdrawObligationCollateralAndRedeemReserveCollateral {
                collateral_amount,
            },
            vec![
                AccountMeta::new(*destination_collateral_info.key, false),
                AccountMeta::new(*user_collateral_info.key, false),
                AccountMeta::new(*withdraw_reserve_info.key, false),
                AccountMeta::new(*obligation_info.key, false),
                AccountMeta::new(*lending_market_info.key, false),
                AccountMeta::new_readonly(*lending_market_authority_info.key, false),
                AccountMeta::new(*token_account.key, false),
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
            token_account.clone(),
            reserve_collateral_mint_info.clone(),
            reserve_liquidity_supply_info.clone(),
            pda_account.clone(),
            clock_info.clone(),
            token_program.clone(),
            solend_program.clone(),
        ],
        &[&[&pda_seed.as_bytes(), &[bump]]],
    )?;

    Ok(())
}

fn payout(accounts: &[AccountInfo], amount: u64, seed: String, bump: u8) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let token_program = next_account_info(accounts_iter)?;
    let fluidity_mint = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let payout_account_a = next_account_info(accounts_iter)?;
    let payout_account_b = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;

    // check payout authority
    if !(payer.is_signer && payer.key ==
         &Pubkey::from_str("sohTpNitFg3WZeEcbrMunnwoZJWP4t8yisPB5o3DGD5").unwrap()) {
        panic!("bad payout authority!");
    }

    let pda_seed =  format!("FLU:{}_OBLIGATION", seed);

    // mint fluid tokens to both receivers
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
        &[&[&pda_seed.as_bytes(), &[bump]]],
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
        &[&[&pda_seed.as_bytes(), &[bump]]],
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
        &[
            payer.clone(), obligation_info.clone(), obligation_owner_info.clone(),
            lending_market_info.clone(), solend_program_info.clone(), system_program_info.clone()
        ],
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

    let data_account = next_account_info(accounts_iter)?;
    let solend_program = next_account_info(accounts_iter)?;
    let obligation_info = next_account_info(accounts_iter)?;
    let reserve_info = next_account_info(accounts_iter)?;
    let pyth_price_info = next_account_info(accounts_iter)?;
    let switchboard_feed_info = next_account_info(accounts_iter)?;
    let clock_info = next_account_info(accounts_iter)?;

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
        &[
            reserve_info.clone(),
            pyth_price_info.clone(),
            switchboard_feed_info.clone(),
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
                AccountMeta::new(*reserve_info.key, false),
            ],
        ),
        &[
            obligation_info.clone(),
            clock_info.clone(),
            reserve_info.clone(),
            solend_program.clone(),
        ],
    )?;

    // deserialize obligation
    let obligation = Obligation::unpack(&obligation_info.data.borrow())?;

    // get data
    let mut data = data_account.try_borrow_mut_data()?;

    // serialize value into data account
    msg!("{:?}", obligation.deposits);
    //msg!("scaled {}", u64::try_from(obligation.deposited_value.to_scaled_val()?).unwrap());
    //u64::try_from(obligation.deposited_value.to_scaled_val()?).unwrap().serialize(&mut &mut data[..])?;

    Ok(())
}

fn init_data(
    accounts: &[AccountInfo],
    seed: String, lamports: u64,
    space: u64, bump: u8
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let system_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let program = next_account_info(accounts_iter)?;
    let data_account = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let fluid_mint = next_account_info(accounts_iter)?;
    let pda = next_account_info(accounts_iter)?;

    let pda_seed = format!("FLU:{}_OBLIGATION", seed);
    let data_seed = format!("FLU:{}_DATA", seed);

    invoke_signed(
        &system_instruction::create_account_with_seed(
            payer.key,
            data_account.key,
            pda.key,
            &data_seed,
            lamports,
            space,
            program.key,
        ),
        &[payer.clone(), data_account.clone(), pda.clone(), system_account.clone()],
        &[&[&pda_seed.as_bytes(), &[bump]]],
    )?;

    let mut data = data_account.try_borrow_mut_data()?;
    FluidityData{
        deposited_liquidity: 0,
        token_mint: *token_mint.key,
        fluid_mint: *fluid_mint.key,
        pda: *pda.key,
    }.serialize(&mut &mut data[..])?;

    Ok(())
}

fn check_mints_and_pda(data_account: &AccountInfo, token_mint: Pubkey, fluid_mint: Pubkey, pda: Pubkey) {
    // get fluidity data
    let data = data_account.try_borrow_data().unwrap();
    let fluidity_data = FluidityData::try_from_slice(&data).unwrap();

    // check that mints and pda are consistent
    if (fluidity_data.token_mint, fluidity_data.fluid_mint, fluidity_data.pda) !=
        (token_mint, fluid_mint, pda) {
            panic!("bad mint or pda");
    }
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
        FluidityInstruction::Payout (amount, seed, bump) => {
            return payout(&accounts, amount, seed, bump);
        }
        FluidityInstruction::InitSolendObligation(obligation_lamports, obligation_size, seed, bump) => {
            return init_solend_obligation(&accounts, obligation_lamports, obligation_size, seed, bump);
        }
        FluidityInstruction::LogTVL => {
            return log_tvl(&accounts);
        }
        FluidityInstruction::InitData(seed, lamports, space, bump) => {
            return init_data(&accounts, seed, lamports, space, bump);
        }
    };
}
