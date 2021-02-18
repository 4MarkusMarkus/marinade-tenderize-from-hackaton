use std::convert::TryInto;
use solana_program::program_error::ProgramError;

use crate::error::StakingError::InvalidInstruction;

pub enum StakingInstruction {

    /// Accounts expected
    ///
    /// 0. `[signer]`
    ///
    Deposit {
        amount: u64
    },

    Withdraw {
        amount: u64
    }
}

impl StakingInstruction {

    /// Unpacks a byte buffer into a [StakingInstruction](enum.StakingInstruction.html)
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::Deposit {
                amount: Self::unpack_amount(rest)?
            },
            1 => Self::Withdraw {
                amount: Self::unpack_amount(input)?
            },
            _ => return Err(InvalidInstruction.into())
        })
    }

    pub fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}