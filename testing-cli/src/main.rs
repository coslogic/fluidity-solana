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
    let prog_id = Pubkey::from_str("CTZtmgscfFZztNRrb8HnbRLpUiujcEuK1YN86aYbHajf").unwrap();
    let token_id = Pubkey::from_str("Bydsa9pQWkjhzvE9XVbrVKcKyaWYwJSokmG4ybFcaVZE").unwrap();
    let token_account = Pubkey::from_str("9jC2SWeNap4FEYq7ZSQ2ktMRn7yks8rwJp2HELdSSLR3").unwrap();

    // get recent blockhash
    let (recent_blockhash, _) = client.get_recent_blockhash().unwrap();

    // create account to pay for everything
    // here i'm using the default account for my test validator, but that won't work on anything except my system.
    let payer = Keypair::from_bytes(&[127,94,209,21,1,167,119,180,188,229,9,157,68,153,36,112,68,100,81,53,204,
                                      236,73,107,125,5,87,233,241,57,233,235,122,7,17,70,84,169,115,252,108,223,
                                      133,54,56,135,195,66,46,219,239,136,167,136,15,205,210,112,31,149,65,126,76,98]).unwrap();
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
