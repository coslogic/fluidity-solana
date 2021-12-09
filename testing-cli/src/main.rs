use solana_client::client_error::reqwest::blocking::Response;

use {
    std::env,
    solana_client::{
        rpc_client::RpcClient,
        rpc_request::RpcError,
    },
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Signer},
        instruction::{AccountMeta, Instruction},
        signer::keypair::Keypair,
        transaction::Transaction,
        sysvar,
        system_program,
        system_instruction,
    },
    borsh::BorshSerialize,
    std::str::FromStr,
    spl_token,
    spl_associated_token_account,
};

#[derive(BorshSerialize)]
enum FluidityInstruction {
    Wrap (u64, String, u8),
    Unwrap (u64, String, u8),
    Payout (u64),
    InitSolendObligation (u64, u64, String, u8),
}

fn test_smart_contract(client: &RpcClient) {
    // program id to send instructions to

    let prog_id = Pubkey::from_str(&env::var("FLU_PROGRAM_ID").unwrap()).unwrap();
    let key_bytes = std::fs::read_to_string(&env::var("SOLANA_ID_PATH").unwrap()).unwrap();
    let command = env::args().nth(1).unwrap();
    let token_name = env::args().nth(2).unwrap();

    // get recent blockhash
    let (recent_blockhash, _) = client.get_recent_blockhash().unwrap();

    // create account to pay for everything
    // here i'm using the default account for my test validator, but that won't work on anything except my system.
    let payer = Keypair::from_bytes(&(serde_json::from_str::<Vec<u8>>(&key_bytes).unwrap())).unwrap();
    println!("payer pubkey: {}", payer.pubkey());

    // derive address of mint account
    let mint_account_seed = format!("FLU:{}_OBLIGATION", token_name);
    let (pda_pubkey, bump_seed) = Pubkey::find_program_address(&[mint_account_seed.as_bytes()], &prog_id);

    match command.as_str() {
        "help" => {
            println!("[wrap/unwrap] base_token_id fluidity_token_id base_token_account fluidity_token_account");
            return
        }
        "printpdakey" => {
            println!("pda pubkey: {}", pda_pubkey);
            println!("bump seed: {}", bump_seed);
            return
        }
        _ => {}
    };

    // select and create instruction
    let inst = match command.as_str() {
        "wrap" => {
            let amount =
                env::args().nth(3).unwrap().parse::<u64>().unwrap();
            let base_token_id =
                Pubkey::from_str(&env::args().nth(4).unwrap()).unwrap();
            let fluidity_token_id =
                Pubkey::from_str(&env::args().nth(5).unwrap()).unwrap();
            let base_token_account =
                Pubkey::from_str(&env::args().nth(6).unwrap()).unwrap();
            let pda_token_account =
                Pubkey::from_str(&env::args().nth(7).unwrap()).unwrap();
            let fluidity_token_account =
                Pubkey::from_str(&env::args().nth(8).unwrap()).unwrap();
            let solend_program =
                Pubkey::from_str(&env::args().nth(9).unwrap()).unwrap();
            let collateral_account =
                Pubkey::from_str(&env::args().nth(10).unwrap()).unwrap();
            let reserve =
                Pubkey::from_str(&env::args().nth(11).unwrap()).unwrap();
            let reserve_liquidity_supply =
                Pubkey::from_str(&env::args().nth(12).unwrap()).unwrap();
            let collateral_mint =
                Pubkey::from_str(&env::args().nth(13).unwrap()).unwrap();
            let lending_market =
                Pubkey::from_str(&env::args().nth(14).unwrap()).unwrap();
            let market_authority =
                Pubkey::from_str(&env::args().nth(15).unwrap()).unwrap();
            let collateral_supply =
                Pubkey::from_str(&env::args().nth(16).unwrap()).unwrap();
            let obligation_account =
                Pubkey::from_str(&env::args().nth(17).unwrap()).unwrap();
            let pyth_account =
                Pubkey::from_str(&env::args().nth(18).unwrap()).unwrap();
            let switchboard_account =
                Pubkey::from_str(&env::args().nth(19).unwrap()).unwrap();
            vec![
                Instruction::new_with_borsh(
                    prog_id,
                    &FluidityInstruction::Wrap(amount, mint_account_seed, bump_seed),
                    vec![
                        AccountMeta::new_readonly(spl_token::ID, false),
                        AccountMeta::new(base_token_id, false),
                        AccountMeta::new(fluidity_token_id, false),
                        AccountMeta::new(pda_pubkey, false),
                        AccountMeta::new(payer.pubkey(), true),
                        AccountMeta::new(base_token_account, false),
                        AccountMeta::new(pda_token_account, false),
                        AccountMeta::new(fluidity_token_account, false),
                        AccountMeta::new_readonly(solend_program, false),
                        AccountMeta::new(collateral_account, false),
                        AccountMeta::new(reserve, false),
                        AccountMeta::new(reserve_liquidity_supply, false),
                        AccountMeta::new(collateral_mint, false),
                        AccountMeta::new(lending_market, false),
                        AccountMeta::new_readonly(market_authority, false),
                        AccountMeta::new(collateral_supply, false),
                        AccountMeta::new(obligation_account, false),
                        AccountMeta::new(pyth_account, false),
                        AccountMeta::new(switchboard_account, false),
                        AccountMeta::new_readonly(sysvar::clock::ID, false),
                    ], 
                ),
            ]
        }
        "unwrap" => {
            let amount =
                env::args().nth(3).unwrap().parse::<u64>().unwrap();
            let base_token_id =
                Pubkey::from_str(&env::args().nth(4).unwrap()).unwrap();
            let fluidity_token_id =
                Pubkey::from_str(&env::args().nth(5).unwrap()).unwrap();
            let base_token_account =
                Pubkey::from_str(&env::args().nth(6).unwrap()).unwrap();
            let fluidity_token_account =
                Pubkey::from_str(&env::args().nth(7).unwrap()).unwrap();
            let solend_program =
                Pubkey::from_str(&env::args().nth(8).unwrap()).unwrap();
            let destination_collateral =
                Pubkey::from_str(&env::args().nth(9).unwrap()).unwrap();
            let user_collateral =
                Pubkey::from_str(&env::args().nth(10).unwrap()).unwrap();
            let withdraw_reserve =
                Pubkey::from_str(&env::args().nth(11).unwrap()).unwrap();
            let obligation =
                Pubkey::from_str(&env::args().nth(12).unwrap()).unwrap();
            let lending_market =
                Pubkey::from_str(&env::args().nth(13).unwrap()).unwrap();
            let lending_market_authority =
                Pubkey::from_str(&env::args().nth(14).unwrap()).unwrap();
            let reserve_collateral_mint =
                Pubkey::from_str(&env::args().nth(15).unwrap()).unwrap();
            let reserve_liquidity_supply =
                Pubkey::from_str(&env::args().nth(16).unwrap()).unwrap();
            let withdraw_pyth_price =
                Pubkey::from_str(&env::args().nth(17).unwrap()).unwrap(); 
            let withdraw_switchboard_feed =
                Pubkey::from_str(&env::args().nth(18).unwrap()).unwrap(); 
            let pda_token_pubkey =
                spl_associated_token_account::get_associated_token_address(&pda_pubkey, &base_token_id);

            vec![Instruction::new_with_borsh(
                prog_id,
                &FluidityInstruction::Unwrap(amount, mint_account_seed, bump_seed),
                vec![
                    AccountMeta::new_readonly(spl_token::ID, false),
                    AccountMeta::new(base_token_id, false),
                    AccountMeta::new(fluidity_token_id, false),
                    AccountMeta::new(pda_pubkey, false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(base_token_account, false),
                    AccountMeta::new(pda_token_pubkey, false),
                    AccountMeta::new(fluidity_token_account, false),
                    AccountMeta::new_readonly(solend_program, false),
                    AccountMeta::new(destination_collateral, false),
                    AccountMeta::new(user_collateral, false),
                    AccountMeta::new(withdraw_reserve, false),
                    AccountMeta::new(obligation, false),
                    AccountMeta::new(lending_market, false),
                    AccountMeta::new_readonly(lending_market_authority, false),
                    AccountMeta::new(reserve_collateral_mint, false),
                    AccountMeta::new(reserve_liquidity_supply, false),
                    AccountMeta::new_readonly(withdraw_pyth_price, false),
                    AccountMeta::new_readonly(withdraw_switchboard_feed, false),
                    AccountMeta::new_readonly(sysvar::clock::ID, false),
                ], 
            )]
        }
        "createacc" => {
            let mint_pk = Pubkey::from_str(&env::args().nth(3).unwrap()).unwrap();
            let acc_pk = Pubkey::from_str(&env::args().nth(4).unwrap()).unwrap();
            println!("creating account {}", spl_associated_token_account::get_associated_token_address(&acc_pk, &mint_pk));
            vec![spl_associated_token_account::create_associated_token_account(
                &payer.pubkey(),
                &acc_pk,
                &mint_pk,
            )]
        }
        "initobligation" => {
            let collateral_mint = Pubkey::from_str(&env::args().nth(3).unwrap()).unwrap();
            let market_address = &env::args().nth(4).unwrap();
            let solend_program = Pubkey::from_str(&env::args().nth(5).unwrap()).unwrap();
            println!("pda pubkey: {}", pda_pubkey);
            println!("pda seed: {}", mint_account_seed);
            println!("bump seed: {}", bump_seed);
            println!("creating {}", Pubkey::create_with_seed(&pda_pubkey, &market_address[0..32], &solend_program).unwrap());
            vec![
                spl_associated_token_account::create_associated_token_account(
                    &payer.pubkey(),
                    &pda_pubkey,
                    &collateral_mint
                ),
                Instruction::new_with_borsh(
                    prog_id,
                    &FluidityInstruction::InitSolendObligation(client.get_minimum_balance_for_rent_exemption(1300).unwrap(), 1300, mint_account_seed, bump_seed),
                    vec![
                        AccountMeta::new(payer.pubkey(), true),
                        AccountMeta::new_readonly(solend_program, false),
                        AccountMeta::new_readonly(system_program::ID, false),
                        AccountMeta::new(Pubkey::create_with_seed(&pda_pubkey, &market_address[0..32], &solend_program).unwrap(), false),
                        AccountMeta::new(Pubkey::from_str(market_address).unwrap(), false),
                        AccountMeta::new(pda_pubkey, false),
                        AccountMeta::new_readonly(sysvar::clock::ID, false),
                        AccountMeta::new_readonly(sysvar::rent::ID, false),
                        AccountMeta::new_readonly(spl_token::ID, false),
                    ]
                )
            ]
        }
        _ => panic!("please provide a valid command (help, wrap, unwrap, createacc, printpdakey)")
    };

    // create and send txn to program
    let mut txn = Transaction::new_with_payer(&inst, Some(&payer.pubkey()));
    txn.sign(&[&payer], recent_blockhash);
    if let Err(e) = client.send_transaction(&txn) {
        if let solana_client::client_error::ClientErrorKind::RpcError(rpc_err) = e.kind {
            match rpc_err {
                RpcError::RpcResponseError{code: _, message, data} =>
                    panic!("Failed to send txn (RPC Error): {}{}", message, match data {
                        solana_client::rpc_request::RpcResponseErrorData::SendTransactionPreflightFailure(res) =>
                            match res.logs {
                                Some(logs) => format!("\nLogs: {:#?}", logs),
                                _ => "".to_string(),
                            }
                        _ => "".to_string(),
                    }),
                RpcError::RpcRequestError(msg)|RpcError::ParseError(msg)|RpcError::ForUser(msg) =>
                    panic!("Failed to send txn (RPC Error): {}", msg),
            }
        } else {
            panic!("Failed to send txn: {}", e)
        }
    };
}

fn main() {
    // connect to local testnet
    let client = RpcClient::new(env::var("SOLANA_NODE_ADDRESS").unwrap());

    // test smart contract functions
    test_smart_contract(&client);
}
