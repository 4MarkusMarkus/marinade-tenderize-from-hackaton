use num_traits::FromPrimitive;
use solana_program::{
    pubkey::Pubkey, account_info::AccountInfo, entrypoint::ProgramResult,
    program_error::PrintProgramError, decode_error::DecodeError, msg};
use crate::error::StakingError;
use crate::instruction::StakingInstruction;

pub struct Processor { }

impl Processor {

    pub fn process_deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        Ok(())
    }

    pub fn process_withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        Ok(())
    }


    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = StakingInstruction::unpack(input)?;

        match instruction {
            StakingInstruction::Deposit { amount } => {
                msg!("Deposit call in processor");
                Self::process_deposit(program_id, accounts, amount)
            },
            StakingInstruction::Withdraw { amount} => {
                msg!("Withdrwal call in processor");
                Self::process_withdraw(program_id, accounts, amount)
            }
        }
    }
}

impl PrintProgramError for StakingError {
    fn print<E>(&self)
        where
            E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            StakingError::NoStaking => msg!("Error: Withdrawal account doesn't stake any tokens"),
            StakingError::InvalidInstruction => msg!("Error: Invalid transaction instruction"),
        }
    }
}