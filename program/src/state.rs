//! State transition types

use crate::error::StakePoolError;
use crate::instruction::Fee;
use crate::processor::Processor;
use core::convert::TryInto;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::TryFrom;
use std::mem::size_of;

/// Initialized program details.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StakePool {
    /// Pool version
    pub version: u8,
    /// Owner authority
    /// allows for updating the staking authority
    pub owner: Pubkey,
    /// Deposit authority bump seed
    /// for `create_program_address(&[state::StakePool account, "deposit"])`
    pub deposit_bump_seed: u8,
    /// Withdrawal authority bump seed
    /// for `create_program_address(&[state::StakePool account, "withdrawal"])`
    pub withdraw_bump_seed: u8,
    /// Validator stake list storage account
    pub validator_stake_list: Pubkey,
    /// Credit list storage account
    pub credit_list: Pubkey,
    /// Pool Mint
    pub pool_mint: Pubkey,
    /// Owner fee account
    pub owner_fee_account: Pubkey,
    /// Credit reserve
    pub credit_reserve: Pubkey,
    /// Pool token program id
    pub token_program_id: Pubkey,
    /// total stake under management
    pub stake_total: u64,
    /// total pool
    pub pool_total: u64,
    /// Last epoch stake_total field was updated
    pub last_update_epoch: u64,
    /// Fee applied to deposits
    pub fee: Fee,
}

impl StakePool {
    /// Length of state data when serialized
    pub const LEN: usize = size_of::<StakePool>();
    /// calculate the pool tokens that should be minted
    pub fn calc_pool_deposit_amount(&self, stake_lamports: u64) -> Option<u64> {
        if self.stake_total == 0 {
            return Some(stake_lamports);
        }
        self.calc_pool_withdraw_amount(stake_lamports)
    }
    /// calculate the pool tokens that should be withdrawn
    pub fn calc_pool_withdraw_amount(&self, stake_lamports: u64) -> Option<u64> {
        u64::try_from(
            (stake_lamports as u128)
                .checked_mul(self.pool_total as u128)?
                .checked_div(self.stake_total as u128)?,
        )
        .ok()
    }
    /// calculate lamports amount
    pub fn calc_lamports_amount(&self, pool_tokens: u64) -> Option<u64> {
        u64::try_from(
            (pool_tokens as u128)
                .checked_mul(self.stake_total as u128)?
                .checked_div(self.pool_total as u128)?,
        )
        .ok()
    }
    /// calculate the fee in pool tokens that goes to the owner
    pub fn calc_fee_amount(&self, pool_amount: u64) -> Option<u64> {
        if self.fee.denominator == 0 {
            return Some(0);
        }
        u64::try_from(
            (pool_amount as u128)
                .checked_mul(self.fee.numerator as u128)?
                .checked_div(self.fee.denominator as u128)?,
        )
        .ok()
    }

    /// Checks withdraw authority
    pub fn check_authority_withdraw(
        &self,
        authority_to_check: &Pubkey,
        program_id: &Pubkey,
        stake_pool_key: &Pubkey,
    ) -> Result<(), ProgramError> {
        Processor::check_authority(
            authority_to_check,
            program_id,
            stake_pool_key,
            Processor::AUTHORITY_WITHDRAW,
            self.withdraw_bump_seed,
        )
    }
    /// Checks deposit authority
    pub fn check_authority_deposit(
        &self,
        authority_to_check: &Pubkey,
        program_id: &Pubkey,
        stake_pool_key: &Pubkey,
    ) -> Result<(), ProgramError> {
        Processor::check_authority(
            authority_to_check,
            program_id,
            stake_pool_key,
            Processor::AUTHORITY_DEPOSIT,
            self.deposit_bump_seed,
        )
    }

    /// Check owner validity and signature
    pub fn check_owner(&self, owner_info: &AccountInfo) -> ProgramResult {
        if *owner_info.key != self.owner {
            return Err(StakePoolError::WrongOwner.into());
        }
        if !owner_info.is_signer {
            return Err(StakePoolError::SignatureMissing.into());
        }
        Ok(())
    }

    /// Check if StakePool is initialized
    pub fn is_initialized(&self) -> bool {
        self.version > 0
    }

    /// Deserializes a byte buffer into a [StakePool](struct.StakePool.html).
    pub fn deserialize(input: &[u8]) -> Result<StakePool, ProgramError> {
        if input.len() < size_of::<StakePool>() {
            return Err(ProgramError::InvalidAccountData);
        }

        let stake_pool: &StakePool = unsafe { &*(&input[0] as *const u8 as *const StakePool) };

        Ok(*stake_pool)
    }

    /// Serializes [StakePool](struct.StakePool.html) into a byte buffer.
    pub fn serialize(&self, output: &mut [u8]) -> ProgramResult {
        if output.len() < size_of::<StakePool>() {
            return Err(ProgramError::InvalidAccountData);
        }
        #[allow(clippy::cast_ptr_alignment)]
        let value = unsafe { &mut *(&mut output[0] as *mut u8 as *mut StakePool) };
        *value = *self;

        Ok(())
    }
}

/// Max validator count
pub const MAX_VALIDATORS: usize = 100;
/// Minimum stake account balance
pub const MIN_STAKE_ACCOUNT_BALANCE: u64 = 100000000; // 0.1 SOL

/// Storage list for all validator stake accounts in the pool.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidatorStakeList {
    /// Validator stake list version
    pub version: u8,
    /// List of all validator stake accounts and their info
    pub validators: Vec<ValidatorStakeInfo>,
}

/// Information about the singe validator stake account
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ValidatorStakeInfo {
    /// Validator account pubkey
    pub validator_account: Pubkey,

    /// Account balance in lamports
    pub balance: u64,

    /// Last epoch balance field was updated
    pub last_update_epoch: u64,

    /// Stake account count
    pub stake_count: u32,
}

impl ValidatorStakeList {
    /// Length of ValidatorStakeList data when serialized
    pub const LEN: usize = Self::HEADER_LEN + ValidatorStakeInfo::LEN * MAX_VALIDATORS;

    /// Header length
    pub const HEADER_LEN: usize = size_of::<u8>() + size_of::<u16>();

    /// Version of validator stake list
    pub const VALIDATOR_STAKE_LIST_VERSION: u8 = 1;

    /// Check if contains validator with particular pubkey
    pub fn contains(&self, validator: &Pubkey) -> bool {
        self.validators
            .iter()
            .any(|x| x.validator_account == *validator)
    }

    /// Check if contains validator with particular pubkey (mutable)
    pub fn find_mut(&mut self, validator: &Pubkey) -> Option<&mut ValidatorStakeInfo> {
        self.validators
            .iter_mut()
            .find(|x| x.validator_account == *validator)
    }
    /// Check if contains validator with particular pubkey (immutable)
    pub fn find(&self, validator: &Pubkey) -> Option<&ValidatorStakeInfo> {
        self.validators
            .iter()
            .find(|x| x.validator_account == *validator)
    }

    /// Check if validator stake list is initialized
    pub fn is_initialized(&self) -> bool {
        self.version > 0
    }

    /// Deserializes a byte buffer into a ValidatorStakeList.
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if input[0] == 0 {
            return Ok(ValidatorStakeList {
                version: 0,
                validators: vec![],
            });
        }

        let number_of_validators: usize = u16::from_le_bytes(
            input[1..3]
                .try_into()
                .or(Err(ProgramError::InvalidAccountData))?,
        ) as usize;
        if number_of_validators > MAX_VALIDATORS {
            return Err(ProgramError::InvalidAccountData);
        }
        let mut validators: Vec<ValidatorStakeInfo> = Vec::with_capacity(number_of_validators + 1);

        let mut from = Self::HEADER_LEN;
        let mut to = from + ValidatorStakeInfo::LEN;
        for _ in 0..number_of_validators {
            validators.push(ValidatorStakeInfo::deserialize(&input[from..to])?);
            from += ValidatorStakeInfo::LEN;
            to += ValidatorStakeInfo::LEN;
        }
        Ok(ValidatorStakeList {
            version: input[0],
            validators,
        })
    }

    /// Serializes ValidatorStakeList into a byte buffer.
    pub fn serialize(&self, output: &mut [u8]) -> ProgramResult {
        if output.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if self.validators.len() > MAX_VALIDATORS {
            return Err(ProgramError::InvalidAccountData);
        }
        output[0] = self.version;
        output[1..3].copy_from_slice(&u16::to_le_bytes(self.validators.len() as u16));
        let mut from = Self::HEADER_LEN;
        let mut to = from + ValidatorStakeInfo::LEN;
        for validator in &self.validators {
            validator.serialize(&mut output[from..to])?;
            from += ValidatorStakeInfo::LEN;
            to += ValidatorStakeInfo::LEN;
        }
        Ok(())
    }
}

impl ValidatorStakeInfo {
    /// Length of ValidatorStakeInfo data when serialized
    pub const LEN: usize = size_of::<ValidatorStakeInfo>();

    /// Deserializes a byte buffer into a ValidatorStakeInfo.
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        #[allow(clippy::cast_ptr_alignment)]
        let stake_info: &ValidatorStakeInfo =
            unsafe { &*(&input[0] as *const u8 as *const ValidatorStakeInfo) };
        Ok(*stake_info)
    }

    /// Serializes ValidatorStakeInfo into a byte buffer.
    pub fn serialize(&self, output: &mut [u8]) -> ProgramResult {
        if output.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        #[allow(clippy::cast_ptr_alignment)]
        let value = unsafe { &mut *(&mut output[0] as *mut u8 as *mut ValidatorStakeInfo) };
        *value = *self;
        Ok(())
    }

    /// Stake account address for validator
    pub fn stake_address(
        &self,
        program_id: &Pubkey,
        stake_pool: &Pubkey,
        index: u32,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &self.validator_account.to_bytes()[..32],
                &stake_pool.to_bytes()[..32],
                &unsafe { std::mem::transmute::<u32, [u8; 4]>(index) },
            ],
            program_id,
        )
    }

    /// Checks if validator stake account is a proper program address
    pub fn check_validator_stake_address(
        &self,
        program_id: &Pubkey,
        stake_pool: &Pubkey,
        index: u32,
        stake_account_pubkey: &Pubkey,
    ) -> Result<u8, ProgramError> {
        // Check stake account address validity
        let (expected_stake_address, bump_seed) =
            self.stake_address(&program_id, &stake_pool, index);
        if *stake_account_pubkey != expected_stake_address {
            msg!(
                "Invalid {} stake account {} for validator {}",
                index,
                stake_account_pubkey,
                self.validator_account
            );
            msg!("Expected {}", expected_stake_address);
            return Err(StakePoolError::InvalidStakeAccountAddress.into());
        }
        Ok(bump_seed)
    }
}

/// Maximum credit records
pub const MAX_CREDIT_RECORDS: usize = 10000;
/// Credit list
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditList {
    /// Validator stake list version
    pub version: u8,
    /// List of all credit users with amount of credit tokens
    pub credits: Vec<CreditRecord>,
}

impl CreditList {
    /// Length of ValidatorStakeList data when serialized
    pub const LEN: usize = Self::HEADER_LEN + CreditRecord::LEN * MAX_CREDIT_RECORDS;

    /// Header length
    pub const HEADER_LEN: usize = size_of::<u8>() + size_of::<u16>();

    /// Version of validator stake list
    pub const VERSION: u8 = 1;

    /// Check if contains validator with particular pubkey
    pub fn contains(&self, user: &Pubkey) -> bool {
        self.credits.iter().any(|x| x.sol_target == *user)
    }

    /// Check if contains validator with particular pubkey (mutable)
    pub fn find_mut(&mut self, user: &Pubkey) -> Option<&mut CreditRecord> {
        self.credits.iter_mut().find(|x| x.sol_target == *user)
    }
    /// Check if contains validator with particular pubkey (immutable)
    pub fn find(&self, user: &Pubkey) -> Option<&CreditRecord> {
        self.credits.iter().find(|x| x.sol_target == *user)
    }

    /// Check if validator stake list is initialized
    pub fn is_initialized(&self) -> bool {
        self.version > 0
    }

    /// Deserializes a byte buffer into a ValidatorStakeList.
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if input[0] == 0 {
            return Ok(CreditList {
                version: 0,
                credits: vec![],
            });
        }

        let number_of_records: usize = u16::from_le_bytes(
            input[1..3]
                .try_into()
                .or(Err(ProgramError::InvalidAccountData))?,
        ) as usize;
        if number_of_records > MAX_CREDIT_RECORDS {
            return Err(ProgramError::InvalidAccountData);
        }
        let mut credits: Vec<CreditRecord> = Vec::with_capacity(number_of_records + 1);

        let mut from = Self::HEADER_LEN;
        let mut to = from + CreditRecord::LEN;
        for _ in 0..number_of_records {
            credits.push(CreditRecord::deserialize(&input[from..to])?);
            from += CreditRecord::LEN;
            to += CreditRecord::LEN;
        }
        Ok(Self {
            version: input[0],
            credits,
        })
    }

    /// Serializes ValidatorStakeList into a byte buffer.
    pub fn serialize(&self, output: &mut [u8]) -> ProgramResult {
        if output.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if self.credits.len() > MAX_CREDIT_RECORDS {
            return Err(ProgramError::InvalidAccountData);
        }
        output[0] = self.version;
        output[1..3].copy_from_slice(&u16::to_le_bytes(self.credits.len() as u16));
        let mut from = Self::HEADER_LEN;
        let mut to = from + CreditRecord::LEN;
        for record in &self.credits {
            record.serialize(&mut output[from..to])?;
            from += CreditRecord::LEN;
            to += CreditRecord::LEN;
        }
        Ok(())
    }
}
/// Credit record
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CreditRecord {
    /// User account for receiving sols
    pub sol_target: Pubkey,
    /// Signature to cancel request
    pub cancel_authority: Pubkey,
    /// Amount from reserve
    pub token_amount: u64,
}

impl CreditRecord {
    /// Length of CreditRecord
    pub const LEN: usize = size_of::<CreditRecord>();

    /// Deserializes a byte buffer into a CreditRecord.
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        #[allow(clippy::cast_ptr_alignment)]
        let credit_record: &CreditRecord =
            unsafe { &*(&input[0] as *const u8 as *const CreditRecord) };
        Ok(*credit_record)
    }

    /// Serializes CreditRecord into a byte buffer.
    pub fn serialize(&self, output: &mut [u8]) -> ProgramResult {
        if output.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        #[allow(clippy::cast_ptr_alignment)]
        let value = unsafe { &mut *(&mut output[0] as *mut u8 as *mut CreditRecord) };
        *value = *self;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_state_packing() {
        // Not initialized
        let stake_list = ValidatorStakeList {
            version: 0,
            validators: vec![],
        };
        let mut bytes: [u8; ValidatorStakeList::LEN] = [0; ValidatorStakeList::LEN];
        stake_list.serialize(&mut bytes).unwrap();
        let stake_list_unpacked = ValidatorStakeList::deserialize(&bytes).unwrap();
        assert_eq!(stake_list_unpacked, stake_list);

        // Empty
        let stake_list = ValidatorStakeList {
            version: ValidatorStakeList::VALIDATOR_STAKE_LIST_VERSION,
            validators: vec![],
        };
        let mut bytes: [u8; ValidatorStakeList::LEN] = [0; ValidatorStakeList::LEN];
        stake_list.serialize(&mut bytes).unwrap();
        let stake_list_unpacked = ValidatorStakeList::deserialize(&bytes).unwrap();
        assert_eq!(stake_list_unpacked, stake_list);

        // With several accounts
        let stake_list = ValidatorStakeList {
            version: ValidatorStakeList::VALIDATOR_STAKE_LIST_VERSION,
            validators: vec![
                ValidatorStakeInfo {
                    validator_account: Pubkey::new_from_array([1; 32]),
                    balance: 123456789,
                    last_update_epoch: 987654321,
                    stake_count: 0,
                },
                ValidatorStakeInfo {
                    validator_account: Pubkey::new_from_array([2; 32]),
                    balance: 998877665544,
                    last_update_epoch: 11223445566,
                    stake_count: 0,
                },
                ValidatorStakeInfo {
                    validator_account: Pubkey::new_from_array([3; 32]),
                    balance: 0,
                    last_update_epoch: 999999999999999,
                    stake_count: 0,
                },
            ],
        };
        let mut bytes: [u8; ValidatorStakeList::LEN] = [0; ValidatorStakeList::LEN];
        stake_list.serialize(&mut bytes).unwrap();
        let stake_list_unpacked = ValidatorStakeList::deserialize(&bytes).unwrap();
        assert_eq!(stake_list_unpacked, stake_list);
    }
}
