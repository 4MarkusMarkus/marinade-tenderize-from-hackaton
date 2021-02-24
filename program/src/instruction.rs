//! Instruction types

#![allow(clippy::too_many_arguments)]

use solana_program::instruction::AccountMeta;
use solana_program::instruction::Instruction;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar;
use std::mem::size_of;

/// Fee rate as a ratio
/// Fee is minted on deposit
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fee {
    /// denominator of the fee ratio
    pub denominator: u64,
    /// numerator of the fee ratio
    pub numerator: u64,
}

/// Inital values for the Stake Pool
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InitArgs {
    /// Fee paid to the owner in pool tokens
    pub fee: Fee,
}

/// Instructions supported by the StakePool program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum StakePoolInstruction {
    ///   0) Initializes a new StakePool.
    ///
    ///   0. `[w]` New StakePool to create.
    ///   1. `[s]` Owner
    ///   2. `[w]` Uninitialized validator stake list storage account
    ///   3. `[]` pool token Mint. Must be non zero, owned by withdraw authority.
    ///   4. `[]` Pool Account to deposit the generated fee for owner.
    ///   5. `[]` Clock sysvar
    ///   6. `[]` Rent sysvar
    ///   7. `[]` Token program id
    Initialize(InitArgs),

    ///   2) Adds validator stake account to the pool
    ///
    ///   0. `[]` Stake pool
    ///   1. `[s]` Owner
    ///   2. `[w]` Validator stake list storage account
    ///   3. `[]` Validator this stake account will vote for
    ///   4. `[]` Clock sysvar (required)
    AddValidator,

    ///   3) Removes validator stake account from the pool
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[s]` Owner
    ///   2. `[]` Stake pool withdraw authority
    ///   3. `[]` New withdraw/staker authority to set in the stake account
    ///   4. `[w]` Validator stake list storage account
    ///   5. `[w]` Stake account to remove from the pool
    ///   6. `[w]` User account with pool tokens to burn from
    ///   7. `[w]` Pool token mint account
    ///   8. '[]' Sysvar clock account (required)
    ///   9. `[]` Pool token program id
    ///  10. `[]` Stake program id,
    RemoveValidatorStakeAccount,

    ///   4) Updates balances of validator stake accounts in the pool
    ///   
    ///   0. `[w]` Validator stake list storage account
    ///   1. `[]` Sysvar clock account
    ///   2. ..2+N ` [] N validator stake accounts to update balances
    UpdateListBalance,

    ///   5) Updates total pool balance based on balances in validator stake account list storage
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[]` Validator stake list storage account
    ///   2. `[]` Sysvar clock account
    UpdatePoolBalance,

    ///   6) Deposit some stake into the pool.  The output is a "pool" token representing ownership
    ///   into the pool. Inputs are converted to the current ratio.
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[]` Stake pool withdraw authority
    ///   2. `[w]` Reserve account (PDA)
    ///   3. `[ws]` User account to take SOLs from
    ///   4. `[w]` User account to receive pool tokens
    ///   5. `[w]` Account to receive pool fee tokens
    ///   6. `[w]` Pool token mint account
    ///   7. `[]` System program
    ///   8. `[]` Pool token program id,
    Deposit(u64),

    ///   7) Withdraw the token from the pool at the current ratio.
    ///   The amount withdrawn is the MIN(u64, stake size)
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[]` Stake pool withdraw authority
    ///   3. `[w]` Validator stake account to split
    ///   4. `[w]` Unitialized stake account to receive withdrawal
    ///   5. `[]` User account to set as a new withdraw authority
    ///   6. `[w]` User account with pool tokens to burn from
    ///   7. `[w]` Pool token mint account
    ///   8. '[]' Sysvar clock account (required)
    ///   9. `[]` Pool token program id
    ///   10. `[]` Stake program id,
    ///   userdata: amount to withdraw
    Withdraw(u64),

    ///   8) Update the staking pubkey for a stake
    ///
    ///   0. `[w]` StakePool
    ///   1. `[s]` Owner
    ///   2. `[]` withdraw authority
    ///   3. `[w]` Stake to update the staking pubkey
    ///   4. '[]` Staking pubkey.
    ///   5. '[]' Sysvar clock account (reserved for future use)
    ///   6. `[]` Stake program id,
    SetStakingAuthority,

    ///   9) Update owner
    ///
    ///   0. `[w]` StakePool
    ///   1. `[s]` Owner
    ///   2. '[]` New owner pubkey
    ///   3. '[]` New owner fee account
    SetOwner,

    ///   10) Test deposit without reserve
    ///
    ///   0. `[w]` StakePool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[]` Stake pool deposit authority
    ///   3. `[ws]` User's wallet
    ///   4. `[]` System program
    ///   5. `[]` Stake program
    ///   6. `[]` Clock sysvar
    ///   7. `[]` Stake history sysvar that carries stake warmup/cooldown history
    ///   8. `[]` Address of config account that carries stake config
    ///   9. ..9+2N ` [w][] N stake + validator vote accounts to update balances
    TestDeposit(u64),

    ///   11) Test deposit without reserve
    ///
    ///   0. `[w]` StakePool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[]` Stake pool withdraw authority
    ///   3. `[w]` User's wallet
    ///   4. `[]` Stake program
    ///   5. `[]` Clock sysvar
    ///   6. `[]` Stake history sysvar that carries stake warmup/cooldown history
    ///   7..7+N `[w]` N stake accounts
    TestWithdraw(u64),

    ///   12) Delegate reserve to stake account
    ///
    ///   0.  `[w]` StakePool
    ///   1.  `[w]` Validator stake list storage account
    ///   2.  `[]` Stake pool withdraw authority
    ///   3.  `[]` Stake pool deposit authority
    ///   4.  `[ws]` SOL reserve account
    ///   5.  `[]` System program
    ///   6.  `[]` Stake program
    ///   7.  `[]` Clock sysvar
    ///   8.  `[]` Stake history sysvar that carries stake warmup/cooldown history
    ///   9.  `[]` Address of config account that carries stake config
    ///   10. `[]` Rent sysvar
    ///   11. ..11+6N ` [w]*6 N times validator + 5 stake accounts
    DelegateReserve(u64),
}

impl StakePoolInstruction {
    /// Deserializes a byte buffer into an [StakePoolInstruction](enum.StakePoolInstruction.html).
    /// TODO efficient unpacking here
    pub fn deserialize(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() < size_of::<u8>() {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(match input[0] {
            0 => {
                let val: &InitArgs = unpack(input)?;
                Self::Initialize(*val)
            }
            2 => Self::AddValidator,
            3 => Self::RemoveValidatorStakeAccount,
            4 => Self::UpdateListBalance,
            5 => Self::UpdatePoolBalance,
            6 => {
                let val: &u64 = unpack(input)?;
                Self::Deposit(*val)
            }
            7 => {
                let val: &u64 = unpack(input)?;
                Self::Withdraw(*val)
            }
            8 => Self::SetStakingAuthority,
            9 => Self::SetOwner,
            10 => {
                let val: &u64 = unpack(input)?;
                Self::TestDeposit(*val)
            }
            11 => {
                let val: &u64 = unpack(input)?;
                Self::TestWithdraw(*val)
            }
            12 => {
                let val: &u64 = unpack(input)?;
                Self::DelegateReserve(*val)
            }
            _ => return Err(ProgramError::InvalidAccountData),
        })
    }

    /// Serializes an [StakePoolInstruction](enum.StakePoolInstruction.html) into a byte buffer.
    /// TODO efficient packing here
    pub fn serialize(&self) -> Result<Vec<u8>, ProgramError> {
        let mut output = vec![0u8; size_of::<StakePoolInstruction>()];
        match self {
            Self::Initialize(init) => {
                output[0] = 0;
                #[allow(clippy::cast_ptr_alignment)]
                let value = unsafe { &mut *(&mut output[1] as *mut u8 as *mut InitArgs) };
                *value = *init;
            }
            Self::AddValidator => {
                output[0] = 2;
            }
            Self::RemoveValidatorStakeAccount => {
                output[0] = 3;
            }
            Self::UpdateListBalance => {
                output[0] = 4;
            }
            Self::UpdatePoolBalance => {
                output[0] = 5;
            }
            Self::Deposit(val) => {
                output[0] = 6;
                let value = unsafe { &mut *(&mut output[1] as *mut u8 as *mut u64) };
                *value = *val;
            }
            Self::Withdraw(val) => {
                output[0] = 7;
                #[allow(clippy::cast_ptr_alignment)]
                let value = unsafe { &mut *(&mut output[1] as *mut u8 as *mut u64) };
                *value = *val;
            }
            Self::SetStakingAuthority => {
                output[0] = 8;
            }
            Self::SetOwner => {
                output[0] = 9;
            }
            Self::TestDeposit(val) => {
                output[0] = 10;
                let value = unsafe { &mut *(&mut output[1] as *mut u8 as *mut u64) };
                *value = *val;
            }
            Self::TestWithdraw(val) => {
                output[0] = 11;
                let value = unsafe { &mut *(&mut output[1] as *mut u8 as *mut u64) };
                *value = *val;
            }
            Self::DelegateReserve(val) => {
                output[0] = 11;
                let value = unsafe { &mut *(&mut output[1] as *mut u8 as *mut u64) };
                *value = *val;
            }
        }
        Ok(output)
    }
}

/// Unpacks a reference from a bytes buffer.
pub fn unpack<T>(input: &[u8]) -> Result<&T, ProgramError> {
    if input.len() < size_of::<u8>() + size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    #[allow(clippy::cast_ptr_alignment)]
    let val: &T = unsafe { &*(&input[1] as *const u8 as *const T) };
    Ok(val)
}
/*
/// Creates an 'initialize' instruction.
pub fn initialize(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    owner: &Pubkey,
    validator_stake_list: &Pubkey,
    pool_mint: &Pubkey,
    owner_pool_account: &Pubkey,
    token_program_id: &Pubkey,
    init_args: InitArgs,
) -> Result<Instruction, ProgramError> {
    let init_data = StakePoolInstruction::Initialize(init_args);
    let data = init_data.serialize()?;
    let accounts = vec![
        AccountMeta::new(*stake_pool, true),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(*validator_stake_list, false),
        AccountMeta::new_readonly(*pool_mint, false),
        AccountMeta::new_readonly(*owner_pool_account, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(*token_program_id, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates `AddValidatorStakeAccount` instruction (add new validator stake account to the pool)
pub fn add_validator(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    owner: &Pubkey,
    stake_pool_deposit: &Pubkey,
    stake_pool_withdraw: &Pubkey,
    validator_stake_list: &Pubkey,
    stake_account: &Pubkey,
    pool_tokens_to: &Pubkey,
    pool_mint: &Pubkey,
    token_program_id: &Pubkey,
    stake_program_id: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(*stake_pool_deposit, false),
        AccountMeta::new_readonly(*stake_pool_withdraw, false),
        AccountMeta::new(*validator_stake_list, false),
        AccountMeta::new(*stake_account, false),
        AccountMeta::new(*pool_tokens_to, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::stake_history::id(), false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(*stake_program_id, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: StakePoolInstruction::AddValidatorStakeAccount.serialize()?,
    })
}*/

/// Creates `RemoveValidatorStakeAccount` instruction (remove validator stake account from the pool)
pub fn remove_validator_stake_account(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    owner: &Pubkey,
    stake_pool_withdraw: &Pubkey,
    new_stake_authority: &Pubkey,
    validator_stake_list: &Pubkey,
    stake_account: &Pubkey,
    burn_from: &Pubkey,
    pool_mint: &Pubkey,
    token_program_id: &Pubkey,
    stake_program_id: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(*stake_pool_withdraw, false),
        AccountMeta::new_readonly(*new_stake_authority, false),
        AccountMeta::new(*validator_stake_list, false),
        AccountMeta::new(*stake_account, false),
        AccountMeta::new(*burn_from, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(*stake_program_id, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: StakePoolInstruction::RemoveValidatorStakeAccount.serialize()?,
    })
}

/// Creates `UpdateListBalance` instruction (update validator stake account balances)
pub fn update_list_balance(
    program_id: &Pubkey,
    validator_stake_list_storage: &Pubkey,
    validator_stake_list: &[&Pubkey],
) -> Result<Instruction, ProgramError> {
    let mut accounts: Vec<AccountMeta> = validator_stake_list
        .iter()
        .map(|pubkey| AccountMeta::new_readonly(**pubkey, false))
        .collect();
    accounts.insert(0, AccountMeta::new(*validator_stake_list_storage, false));
    accounts.insert(1, AccountMeta::new_readonly(sysvar::clock::id(), false));
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: StakePoolInstruction::UpdateListBalance.serialize()?,
    })
}

/// Creates `UpdatePoolBalance` instruction (pool balance from the stake account list balances)
pub fn update_pool_balance(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    validator_stake_list_storage: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*validator_stake_list_storage, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: StakePoolInstruction::UpdatePoolBalance.serialize()?,
    })
}
/*
/// Creates a 'Deposit' instruction.
pub fn deposit(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    validator_stake_list_storage: &Pubkey,
    stake_pool_deposit: &Pubkey,
    stake_pool_withdraw: &Pubkey,
    stake_to_join: &Pubkey,
    validator_stake_accont: &Pubkey,
    pool_tokens_to: &Pubkey,
    pool_fee_to: &Pubkey,
    pool_mint: &Pubkey,
    token_program_id: &Pubkey,
    stake_program_id: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let args = StakePoolInstruction::Deposit;
    let data = args.serialize()?;
    let accounts = vec![
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*validator_stake_list_storage, false),
        AccountMeta::new_readonly(*stake_pool_deposit, false),
        AccountMeta::new_readonly(*stake_pool_withdraw, false),
        AccountMeta::new(*stake_to_join, false),
        AccountMeta::new(*validator_stake_accont, false),
        AccountMeta::new(*pool_tokens_to, false),
        AccountMeta::new(*pool_fee_to, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::stake_history::id(), false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(*stake_program_id, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
*/
/// Creates a 'withdraw' instruction.
pub fn withdraw(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    validator_stake_list_storage: &Pubkey,
    stake_pool_withdraw: &Pubkey,
    stake_to_split: &Pubkey,
    stake_to_receive: &Pubkey,
    user_withdrawer: &Pubkey,
    burn_from: &Pubkey,
    pool_mint: &Pubkey,
    token_program_id: &Pubkey,
    stake_program_id: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let args = StakePoolInstruction::Withdraw(amount);
    let data = args.serialize()?;
    let accounts = vec![
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*validator_stake_list_storage, false),
        AccountMeta::new_readonly(*stake_pool_withdraw, false),
        AccountMeta::new(*stake_to_split, false),
        AccountMeta::new(*stake_to_receive, false),
        AccountMeta::new_readonly(*user_withdrawer, false),
        AccountMeta::new(*burn_from, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(*stake_program_id, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates a 'set staking authority' instruction.
pub fn set_staking_authority(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    stake_pool_owner: &Pubkey,
    stake_pool_withdraw: &Pubkey,
    stake_account_to_update: &Pubkey,
    stake_account_new_authority: &Pubkey,
    stake_program_id: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let args = StakePoolInstruction::SetStakingAuthority;
    let data = args.serialize()?;
    let accounts = vec![
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new_readonly(*stake_pool_owner, true),
        AccountMeta::new_readonly(*stake_pool_withdraw, false),
        AccountMeta::new(*stake_account_to_update, false),
        AccountMeta::new_readonly(*stake_account_new_authority, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(*stake_program_id, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates a 'set owner' instruction.
pub fn set_owner(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    stake_pool_owner: &Pubkey,
    stake_pool_new_owner: &Pubkey,
    stake_pool_new_fee_receiver: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let args = StakePoolInstruction::SetOwner;
    let data = args.serialize()?;
    let accounts = vec![
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new_readonly(*stake_pool_owner, true),
        AccountMeta::new_readonly(*stake_pool_new_owner, false),
        AccountMeta::new_readonly(*stake_pool_new_fee_receiver, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
