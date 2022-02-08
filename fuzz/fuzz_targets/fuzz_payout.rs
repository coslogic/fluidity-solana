#![no_main]
use libfuzzer_sys::fuzz_target;
use solana_fluidity_fuzz::{
    process_instruction, setup_payout_keys
};
use bumpalo::Bump;


fuzz_target!(|data: &[u8]| {
    let bump = Bump::new();
    let payout_accounts = setup_payout_keys(&bump);
    
    
});
