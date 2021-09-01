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
        system_instruction,
    },
    borsh::{BorshSerialize, BorshDeserialize},
    std::str::FromStr,
    websocket,
    spl_token,
};

#[derive(BorshSerialize)]
enum FluidityInstruction {
    EnlistTxn([u8; 64], Pubkey, Pubkey),
    Wrap(u64),
    FlushTxns,
}

// [u8; 64] is used here since borshserialize isn't implemented for signature::Signature.
// could be worth impl'ing, but i can't be bothered right now.
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct PoolAccount {
    pub txns: Vec<([u8; 64], Pubkey, Pubkey)>,
}

fn test_smart_contract(client: &RpcClient) {
    // program id to send instructions to
    let prog_id = Pubkey::from_str("CTZtmgscfFZztNRrb8HnbRLpUiujcEuK1YN86aYbHajf").unwrap();
    let token_id = Pubkey::from_str("Bydsa9pQWkjhzvE9XVbrVKcKyaWYwJSokmG4ybFcaVZE").unwrap();
    let token_account = Pubkey::from_str("9jC2SWeNap4FEYq7ZSQ2ktMRn7yks8rwJp2HELdSSLR3").unwrap();

    /*// calculate fees
    let mut fees = 0;
    fees += feecalc.lamports_per_signature * 100; */
    let (recent_blockhash, feecalc) = client.get_recent_blockhash().unwrap();

    // create account to pay for everything
    // here i'm using the default account for my test validator, but that won't work on anything except my system.
    let payer = Keypair::from_bytes(&[127,94,209,21,1,167,119,180,188,229,9,157,68,153,36,112,68,100,81,53,204,236,73,107,125,5,87,233,241,57,233,235,122,7,17,70,84,169,115,252,108,223,133,54,56,135,195,66,46,219,239,136,167,136,15,205,210,112,31,149,65,126,76,98]).unwrap();
    println!("{}", payer.pubkey());

    /*let payer = Keypair::new();
    // send airdrop to our new payer acc
    let mut sig = match client.request_airdrop(&payer.pubkey(), fees) {
        Ok(sig) => sig,
        Err(e) => panic!("{}", e),
    };
    // check for confirmation
    while match client.confirm_transaction(&sig) {
        Ok(o) => !o,
        Err(e) => panic!("Failed to confirm airdrop: {}", e),
    }{};
    println!("{}", sig);*/

    // derive address of mint account
    let mint_account_seed = b"FLU: MINT ACCOUNT";
    let (mint_pubkey, bump_seed) = Pubkey::find_program_address(&[mint_account_seed], &prog_id);
    println!("{}, {}", mint_pubkey, bump_seed);

    let inst = Instruction::new_with_borsh(
        prog_id,
        &FluidityInstruction::Wrap(1),
        vec![AccountMeta::new_readonly(spl_token::ID, false), AccountMeta::new(token_id, false), AccountMeta::new(mint_pubkey, false), AccountMeta::new(payer.pubkey(), true), AccountMeta::new(token_account, false)], 
    );

    // create and send txn to program
    let mut txn = Transaction::new_with_payer(&[inst], Some(&payer.pubkey()));
    txn.sign(&[&payer], recent_blockhash);
    let _sig = match client.send_transaction(&txn) {
        Ok(sig) => sig,
        Err(e) => match e.kind {
            solana_client::client_error::ClientErrorKind::RpcError(rpc_err) =>
                match rpc_err {
                    RpcError::RpcResponseError{code, message, data} =>
                        panic!("Failed to send txn (RPC Error): {}{}", message, match data {
                            solana_client::rpc_request::RpcResponseErrorData::SendTransactionPreflightFailure(res) =>
                                match res.logs {
                                    Some(logs) => format!("\nLogs: {:#?}", logs),
                                    _ => "".to_string(),
                                }
                            _ => "".to_string(),
                        }),
                    RpcError::RpcRequestError(msg)|RpcError::ParseError(msg)|RpcError::ForUser(msg) => panic!("Failed to send txn (RPC Error): {}", msg),
                }
            _ => panic!("Failed to send txn: {}", e),
        },
    };
}

fn get_txn_pool(client: &RpcClient) {
    // program id to send instructions to
    let prog_id = Pubkey::from_str("H1BodyrgaWK8nH6Z8wB3iP96memtTFWQv8GdCS4eQ61X").unwrap();

    // create account to pay for everything
    // here i'm using the default account for my test validator, but that won't work on anything except my system.
    let payer = Keypair::from_bytes(&[127,94,209,21,1,167,119,180,188,229,9,157,68,153,36,112,68,100,81,53,204,236,73,107,125,5,87,233,241,57,233,235,122,7,17,70,84,169,115,252,108,223,133,54,56,135,195,66,46,219,239,136,167,136,15,205,210,112,31,149,65,126,76,98]).unwrap();

    // derive address of pool account
    let pool_account_seed = "FLU: POOL ACCOUNT";
    let pool_pubkey = Pubkey::create_with_seed(&payer.pubkey(), &pool_account_seed, &prog_id).unwrap();
    println!("{}", pool_pubkey);

    let acc = match client.get_account_with_commitment(&pool_pubkey, solana_sdk::commitment_config::CommitmentConfig::confirmed()).unwrap().value {
        Some(acc) => {
            println!("Pool account exists!");
            acc
        },
        None => panic!("Pool account does not exist!"),
    };
    println!("{:?}", PoolAccount::deserialize(&mut &acc.data[..]));
}

fn test_borsh_stuff() {
    let data = PoolAccount{txns: vec![([u8::MAX; 64], Pubkey::new_unique(), Pubkey::new_unique()); 256]}.try_to_vec();
    println!("{:?}", data);
}

fn main() {
    // connect to local testnet
    let client = RpcClient::new("http://thorondir.bounceme.net:8899".to_string());

    // test smart contract functions
    match env::args().nth(1).as_ref().map(|s| s.as_str()) {
        Some("check") => get_txn_pool(&client),
        Some("test") => test_borsh_stuff(),
        _ => test_smart_contract(&client),
    };
}
