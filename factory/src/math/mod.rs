//! Math for preserving precision
// taken from https://github.com/solendprotocol/solana-program-library/tree/master/token-lending/program/src/math

mod common;
mod decimal;
mod rate;

pub use common::*;
pub use decimal::*;
pub use rate::*;
