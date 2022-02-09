#![no_main]
use libfuzzer_sys::fuzz_target;
use solana_fluidity_fuzz::*;
use bumpalo::Bump;
use fluidity::instruction::FluidityInstruction;
use std::str;
use borsh::ser::BorshSerialize;


fuzz_target!(|data: &[u8]| {
    let bump = Bump::new();
    let fun = FluidityInstruction::LogTVL;
    let program_id = random_pubkey(&bump);
    process_instruction(program_id, &[], &fun.try_to_vec().unwrap());
});
