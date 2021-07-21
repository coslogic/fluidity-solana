use {
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Signature, Signer},
        instruction::{AccountMeta, Instruction},
        signer::keypair::Keypair,
        transaction::Transaction,
        system_instruction,
    },
    serde::Serialize,
    borsh::{BorshSerialize, BorshDeserialize},
    solana_transaction_status::UiTransactionEncoding,
    std::str::FromStr,
};

#[derive(BorshSerialize)]
enum FluidityInstruction {
    EnlistTxn(String, Pubkey, Pubkey),
    FlushTxns,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct PoolAccount {
    pub txns: Vec<(String, Pubkey, Pubkey)>,
}

fn test_smart_contract(client: &RpcClient) {
    // program id to send instructions to
    let prog_id = Pubkey::from_str("H1BodyrgaWK8nH6Z8wB3iP96memtTFWQv8GdCS4eQ61X").unwrap();

    // calculate fees
    let mut fees = 0;
    let (recent_blockhash, feecalc) = client.get_recent_blockhash().unwrap();
    fees += feecalc.lamports_per_signature * 100;

    // create account to pay for everything
    let payer = Keypair::from_bytes(&[127,94,209,21,1,167,119,180,188,229,9,157,68,153,36,112,68,100,81,53,204,236,73,107,125,5,87,233,241,57,233,235,122,7,17,70,84,169,115,252,108,223,133,54,56,135,195,66,46,219,239,136,167,136,15,205,210,112,31,149,65,126,76,98]).unwrap();
    /*
    let payer = Keypair::new();
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
    println!("{}", sig);
    */

    // derive address of pool account
    let pool_account_seed = "FLU: POOL ACCOUNT";
    let pool_pubkey = Pubkey::create_with_seed(&payer.pubkey(), &pool_account_seed, &prog_id).unwrap();
    println!("{}", pool_pubkey);
    // check if account exists
    match client.get_account_with_commitment(&pool_pubkey, solana_sdk::commitment_config::CommitmentConfig::confirmed()).unwrap().value {
        Some(_) => {println!("pool account exists!")},
        None => {
            match client.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
                &[system_instruction::create_account_with_seed(
                    &payer.pubkey(),
                    &pool_pubkey,
                    &payer.pubkey(),
                    &pool_account_seed,
                    feecalc.lamports_per_signature * 10_000_000,
                    10_000_000,
                    &prog_id,
                )],
                Some(&payer.pubkey()),
                &[&payer],
                recent_blockhash,
            )) {
                Ok(_) => {},
                Err(e) => panic!("Failed to create pool account: {}", e),
            };
        },
    }

    // establish instruction to send to program
    let inst = Instruction::new_with_borsh(prog_id, &[FluidityInstruction::EnlistTxn("f4ke".to_string(), payer.pubkey(), prog_id)], vec![AccountMeta::new(pool_pubkey, false)]);
    //let inst = Instruction::new_with_borsh(prog_id, &[FluidityInstruction::FlushTxns], vec![]);

    // create and send txn to program
    let mut txn = Transaction::new_with_payer(&[inst], Some(&payer.pubkey()));
    txn.sign(&[&payer], recent_blockhash);
    let _sig = match client.send_transaction(&txn) {
        Ok(sig) => sig,
        Err(e) => panic!("Failed to send txn: {}", e),
    };
}

fn get_txn_pool(client: &RpcClient) {
}

fn main() {
    // connect to local testnet
    let client = RpcClient::new("http://localhost:8899".to_string());

    // test smart contract functions
    test_smart_contract(&client);
}
