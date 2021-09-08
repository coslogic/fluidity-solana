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
        system_program,
    },
    borsh::BorshSerialize,
    std::str::FromStr,
    spl_token,
    spl_associated_token_account::create_associated_token_account,
};

#[derive(BorshSerialize)]
enum FluidityInstruction {
    Wrap (u64),
    Unwrap (u64),
}

fn test_smart_contract(client: &RpcClient) {
    // program id to send instructions to
    //let prog_id = Pubkey::from_str("CTZtmgscfFZztNRrb8HnbRLpUiujcEuK1YN86aYbHajf").unwrap();
    //let token_id = Pubkey::from_str("Bydsa9pQWkjhzvE9XVbrVKcKyaWYwJSokmG4ybFcaVZE").unwrap();
    //let token_account = Pubkey::from_str("8T2jfYiUdkLReHpLtHZtp8zHocNSG58hje6T226dqXyx").unwrap();
    let prog_id = Pubkey::from_str(&env::var("FLU_PROGRAM_ID").unwrap()).unwrap();
    let token_id = Pubkey::from_str(&env::var("FLU_TOKEN_ID").unwrap()).unwrap();
    let token_account = Pubkey::from_str(&env::var("FLU_CLI_TOKEN_ACC").unwrap()).unwrap();

    // get recent blockhash
    let (recent_blockhash, _) = client.get_recent_blockhash().unwrap();

    // create account to pay for everything
    // here i'm using the default account for my test validator, but that won't work on anything except my system.
    let payer = Keypair::from_bytes(&[22,34,43,58,175,94,194,175,82,66,142,68,24,207,218,72,6,198,90,108,139,206,103,100,176,247,69,172,143,190,204,187,12,252,227,17,198,165,138,87,211,221,184,212,40,223,101,174,228,189,232,164,103,9,189,225,14,237,137,247,64,212,103,68]
                                      ).unwrap();
    println!("{}", payer.pubkey());

    // derive address of mint account
    let mint_account_seed = b"FLU: MINT ACCOUNT";
    let (mint_pubkey, bump_seed) = Pubkey::find_program_address(&[mint_account_seed], &prog_id);
    println!("{}, {}", mint_pubkey, bump_seed);

    // select and create instruction
    let inst = match env::args().nth(1).as_ref().map(|s| s.as_str()) {
        Some("wrap") => {
            let amount = env::args().nth(2).unwrap().parse::<u64>().unwrap();
            Instruction::new_with_borsh(
                prog_id,
                &FluidityInstruction::Wrap(amount),
                vec![
                    AccountMeta::new_readonly(spl_token::ID, false),
                    AccountMeta::new(token_id, false),
                    AccountMeta::new(mint_pubkey, false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(token_account, false),
                    AccountMeta::new(system_program::id(), false)
                ], 
            )
        }
        Some("unwrap") => {
            let amount = env::args().nth(2).unwrap().parse::<u64>().unwrap();
            Instruction::new_with_borsh(
                prog_id,
                &FluidityInstruction::Unwrap(amount),
                vec![AccountMeta::new_readonly(spl_token::ID, false),
                    AccountMeta::new(token_id, false),
                    AccountMeta::new(mint_pubkey, false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(token_account, false),
                    AccountMeta::new(system_program::id(), false)
                ], 
            )
        }
        Some("createacc") => {
            let mint_pk = Pubkey::from_str(&env::args().nth(2).unwrap()).unwrap();
            let acc_pk = Pubkey::from_str(&env::args().nth(3).unwrap()).unwrap();
            create_associated_token_account(
                &payer.pubkey(),
                &acc_pk,
                &mint_pk,
            )
        }
        _ => panic!("please provide a valid command (wrap, unwrap, createacc)")
    };

    // create and send txn to program
    let mut txn = Transaction::new_with_payer(&[inst], Some(&payer.pubkey()));
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
    let client = RpcClient::new("http://thorondir.bounceme.net:8899".to_string());

    // test smart contract functions
    test_smart_contract(&client);
}
