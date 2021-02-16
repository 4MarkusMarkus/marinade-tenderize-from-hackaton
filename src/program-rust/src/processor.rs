use num_traits::FromPrimitive;
use solana_program::{
    pubkey::Pubkey, account_info::AccountInfo, entrypoint::ProgramResult,
    program_error::PrintProgramError, decode_error::DecodeError, msg};
use crate::error::StakingError;

pub struct Processor { }

impl Processor {

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        Ok(())
    }
}

impl PrintProgramError for StakingError {
    fn print<E>(&self)
        where
            E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            StakingError::NoStaking => msg!("Error: Withdrawal account doesn't stake any tokens"),
        }
    }
}