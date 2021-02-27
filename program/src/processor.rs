//! Program state processor

use crate::{
    error::StakePoolError,
    instruction::{
        DelegateReserveInstruction, InitArgs, MergeStakesInstruction, StakePoolInstruction,
    },
    stake::{self, StakeState},
    state::{
        CreditList, CreditRecord, StakePool, ValidatorStakeInfo, ValidatorStakeList,
        MAX_CREDIT_RECORDS, MIN_STAKE_ACCOUNT_BALANCE,
    },
    PROGRAM_VERSION,
};
use bincode::deserialize;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    clock::Clock,
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::PrintProgramError,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};
use spl_token::state::Mint;

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Suffix for deposit authority seed
    pub const AUTHORITY_DEPOSIT: &'static [u8] = b"deposit";
    /// Suffix for withdraw authority seed
    pub const AUTHORITY_WITHDRAW: &'static [u8] = b"withdraw";
    /// Suffix for reserve account seed
    pub const AUTHORITY_RESERVE: &'static [u8] = b"reserve";
    /// Suffix for temp account
    pub const TEMP_ACCOUNT: &'static [u8] = b"temp";
    /// Calculates the authority id by generating a program address.
    pub fn authority_id(
        program_id: &Pubkey,
        stake_pool: &Pubkey,
        authority_type: &[u8],
        bump_seed: u8,
    ) -> Result<Pubkey, ProgramError> {
        Pubkey::create_program_address(
            &[&stake_pool.to_bytes()[..32], authority_type, &[bump_seed]],
            program_id,
        )
        .map_err(|_| StakePoolError::InvalidProgramAddress.into())
    }
    /// Generates seed bump for stake pool authorities
    pub fn find_authority_bump_seed(
        program_id: &Pubkey,
        stake_pool: &Pubkey,
        authority_type: &[u8],
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&stake_pool.to_bytes()[..32], authority_type], program_id)
    }

    /// Checks withdraw or deposit authority
    pub fn check_authority(
        authority_to_check: &Pubkey,
        program_id: &Pubkey,
        stake_pool_key: &Pubkey,
        authority_type: &[u8],
        bump_seed: u8,
    ) -> Result<(), ProgramError> {
        let id = Self::authority_id(program_id, stake_pool_key, authority_type, bump_seed)?;
        if *authority_to_check != id {
            msg!(
                "Check {} authority fails. Expected {} got {}",
                std::str::from_utf8(authority_type).unwrap(),
                id,
                authority_to_check
            );
            return Err(StakePoolError::InvalidProgramAddress.into());
        }
        Ok(())
    }

    /// Get address for reserve
    pub fn get_reserve_adderess(program_id: &Pubkey, stake_pool: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&stake_pool.to_bytes()[..32], &Self::AUTHORITY_RESERVE],
            program_id,
        )
    }

    /// Returns validator address for a particular stake account
    pub fn get_validator(stake_account_info: &AccountInfo) -> Result<Pubkey, ProgramError> {
        let stake_state: stake::StakeState = deserialize(&stake_account_info.data.borrow())
            .or_else(|_| {
                msg!("Error reading stake {} state", stake_account_info.key);
                Err(ProgramError::InvalidAccountData)
            })?;
        match stake_state {
            stake::StakeState::Stake(_, stake) => Ok(stake.delegation.voter_pubkey),
            _ => Err(StakePoolError::WrongStakeState.into()),
        }
    }

    /// Issue a spl_token `Burn` instruction.
    #[allow(clippy::too_many_arguments)]
    pub fn token_burn<'a>(
        stake_pool: &Pubkey,
        token_program: AccountInfo<'a>,
        burn_account: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        authority_type: &[u8],
        bump_seed: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let me_bytes = stake_pool.to_bytes();
        let authority_signature_seeds = [&me_bytes[..32], authority_type, &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = spl_token::instruction::burn(
            token_program.key,
            burn_account.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(
            &ix,
            &[burn_account, mint, authority, token_program],
            signers,
        )
    }

    /// Issue a spl_token `MintTo` instruction.
    #[allow(clippy::too_many_arguments)]
    pub fn token_mint_to<'a>(
        stake_pool: &Pubkey,
        token_program: AccountInfo<'a>,
        mint: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        authority_type: &[u8],
        bump_seed: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let me_bytes = stake_pool.to_bytes();
        let authority_signature_seeds = [&me_bytes[..32], authority_type, &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?;

        invoke_signed(&ix, &[mint, destination, authority, token_program], signers)
    }

    /// Processes `Initialize` instruction.
    pub fn process_initialize(
        program_id: &Pubkey,
        init: InitArgs,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_pool_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        let credit_list_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let owner_fee_info = next_account_info(account_info_iter)?;
        let credit_reserve_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;
        // Rent sysvar account
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;
        // Token program ID
        let token_program_info = next_account_info(account_info_iter)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        // Stake pool account should not be already initialized
        if stake_pool.is_initialized() {
            return Err(StakePoolError::AlreadyInUse.into());
        }

        // Check if transaction was signed by owner
        if !owner_info.is_signer {
            return Err(StakePoolError::SignatureMissing.into());
        }

        // Check if validator stake list storage is unitialized
        let mut validator_stake_list =
            ValidatorStakeList::deserialize(&validator_stake_list_info.data.borrow())?;
        if validator_stake_list.is_initialized() {
            return Err(StakePoolError::AlreadyInUse.into());
        }
        validator_stake_list.version = ValidatorStakeList::VALIDATOR_STAKE_LIST_VERSION;
        validator_stake_list.validators.clear();

        // Check if credit list storage is unitialized
        let mut credit_list = CreditList::deserialize(&credit_list_info.data.borrow())?;
        if credit_list.is_initialized() {
            return Err(StakePoolError::AlreadyInUse.into());
        }
        credit_list.version = CreditList::VERSION;
        credit_list.credits.clear();

        // Check if stake pool account is rent-exempt
        if !rent.is_exempt(stake_pool_info.lamports(), stake_pool_info.data_len()) {
            return Err(StakePoolError::AccountNotRentExempt.into());
        }

        // Check if validator stake list account is rent-exempt
        if !rent.is_exempt(
            validator_stake_list_info.lamports(),
            validator_stake_list_info.data_len(),
        ) {
            return Err(StakePoolError::AccountNotRentExempt.into());
        }

        // Check if credit list account is rent-exempt
        if !rent.is_exempt(credit_list_info.lamports(), credit_list_info.data_len()) {
            return Err(StakePoolError::AccountNotRentExempt.into());
        }

        let (_, deposit_bump_seed) = Self::find_authority_bump_seed(
            program_id,
            stake_pool_info.key,
            Self::AUTHORITY_DEPOSIT,
        );
        let (withdraw_authority_key, withdraw_bump_seed) = Self::find_authority_bump_seed(
            program_id,
            stake_pool_info.key,
            Self::AUTHORITY_WITHDRAW,
        );

        // Numerator should be smaller than or equal to denominator (fee <= 1)
        if init.fee.numerator > init.fee.denominator {
            return Err(StakePoolError::FeeTooHigh.into());
        }

        // Check if fee account's owner the same as token program id
        if owner_fee_info.owner != token_program_info.key {
            msg!(
                "Expexted owner fee's account {} to have {} owner but it has {}",
                owner_fee_info.key,
                token_program_info.key,
                owner_fee_info.owner
            );
            return Err(StakePoolError::InvalidFeeAccount.into());
        }

        // Check if credit account's owner the same as token program id
        if credit_reserve_info.owner != token_program_info.key {
            msg!(
                "Expexted credit reserve's account {} to have {} owner but it has {}",
                credit_reserve_info.key,
                token_program_info.key,
                credit_reserve_info.owner
            );
            return Err(StakePoolError::InvalidFeeAccount.into());
        }

        // Check pool mint program ID
        if pool_mint_info.owner != token_program_info.key {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Check for owner fee account to have proper mint assigned
        if *pool_mint_info.key
            != spl_token::state::Account::unpack_from_slice(&owner_fee_info.data.borrow())?.mint
        {
            return Err(StakePoolError::WrongAccountMint.into());
        }

        let credit_state =
            spl_token::state::Account::unpack_from_slice(&credit_reserve_info.data.borrow())?;
        // Check for credit account to have proper mint assigned
        if *pool_mint_info.key != credit_state.mint {
            return Err(StakePoolError::WrongAccountMint.into());
        }

        if credit_state.state != spl_token::state::AccountState::Initialized {
            return Err(StakePoolError::WrongCreditState.into());
        }

        if credit_state.delegate.is_some() {
            return Err(StakePoolError::WrongCreditState.into());
        }

        if credit_state.close_authority.is_some() {
            return Err(StakePoolError::WrongCreditState.into());
        }

        let pool_mint = Mint::unpack_from_slice(&pool_mint_info.data.borrow())?;

        if !pool_mint.mint_authority.contains(&withdraw_authority_key) {
            msg!(
                "Mint authority is {} but need to be {}",
                pool_mint.mint_authority.unwrap_or(Pubkey::new(&[0; 32])),
                withdraw_authority_key
            );
            return Err(StakePoolError::WrongMintingAuthority.into());
        }

        if pool_mint.supply > 0 {
            return Err(StakePoolError::MintHasInitialSupply.into());
        }

        // change credit reserve owner to PDA
        invoke(
            &spl_token::instruction::set_authority(
                token_program_info.key,
                credit_reserve_info.key,
                Some(&withdraw_authority_key),
                spl_token::instruction::AuthorityType::AccountOwner,
                owner_info.key,
                &[],
            )?,
            &[
                token_program_info.clone(),
                owner_info.clone(),
                credit_reserve_info.clone(),
            ],
        )?;

        validator_stake_list.serialize(&mut validator_stake_list_info.data.borrow_mut())?;

        msg!("Clock data: {:?}", clock_info.data.borrow());
        msg!("Epoch: {}", clock.epoch);

        stake_pool.version = PROGRAM_VERSION;
        stake_pool.owner = *owner_info.key;
        stake_pool.deposit_bump_seed = deposit_bump_seed;
        stake_pool.withdraw_bump_seed = withdraw_bump_seed;
        stake_pool.validator_stake_list = *validator_stake_list_info.key;
        stake_pool.credit_list = *credit_list_info.key;
        stake_pool.pool_mint = *pool_mint_info.key;
        stake_pool.owner_fee_account = *owner_fee_info.key;
        stake_pool.credit_reserve = *credit_reserve_info.key;
        stake_pool.token_program_id = *token_program_info.key;
        stake_pool.last_update_epoch = clock.epoch;
        stake_pool.fee = init.fee;

        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())
    }

    /// Processes `AddValidator` instruction.
    pub fn process_add_validator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // Stake pool account
        let stake_pool_info = next_account_info(account_info_iter)?;
        // Pool owner account
        let owner_info = next_account_info(account_info_iter)?;
        // Account storing validator stake list
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        // Validator this stake account will vote for
        let validator_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        // Get stake pool stake (and check if it is initialized)
        let stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check owner validity and signature
        stake_pool.check_owner(owner_info)?;

        // Check stake pool last update epoch
        if stake_pool.last_update_epoch < clock.epoch {
            return Err(StakePoolError::StakeListAndPoolOutOfDate.into());
        }

        // Check validator stake account list storage
        if *validator_stake_list_info.key != stake_pool.validator_stake_list {
            return Err(StakePoolError::InvalidValidatorStakeList.into());
        }

        // Read validator stake list account and check if it is valid
        let mut validator_stake_list =
            ValidatorStakeList::deserialize(&validator_stake_list_info.data.borrow())?;
        if !validator_stake_list.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        if validator_stake_list.validators.len() >= crate::state::MAX_VALIDATORS {
            return Err(StakePoolError::ValidatorListOverflow.into());
        }

        if validator_stake_list.contains(&validator_info.key) {
            return Err(StakePoolError::ValidatorAlreadyAdded.into());
        }

        // Add validator to the list and save
        validator_stake_list.validators.push(ValidatorStakeInfo {
            validator_account: validator_info.key.clone(),
            balance: 0,
            last_update_epoch: clock.epoch,
            stake_count: 0,
        });
        validator_stake_list.serialize(&mut validator_stake_list_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes `RemoveValidatorStakeAccount` instruction.
    pub fn process_remove_validator(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // Stake pool account
        let stake_pool_info = next_account_info(account_info_iter)?;
        // Pool owner account
        let owner_info = next_account_info(account_info_iter)?;
        // Account storing validator stake list
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        // Validator this stake account will vote for
        let validator_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        // Get stake pool stake (and check if it is initialized)
        let stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check owner validity and signature
        stake_pool.check_owner(owner_info)?;

        // Check stake pool last update epoch
        if stake_pool.last_update_epoch < clock.epoch {
            return Err(StakePoolError::StakeListAndPoolOutOfDate.into());
        }

        // Check validator stake account list storage
        if *validator_stake_list_info.key != stake_pool.validator_stake_list {
            return Err(StakePoolError::InvalidValidatorStakeList.into());
        }

        // Read validator stake list account and check if it is valid
        let mut validator_stake_list =
            ValidatorStakeList::deserialize(&validator_stake_list_info.data.borrow())?;
        if !validator_stake_list.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        if let Some(index) = validator_stake_list
            .validators
            .iter()
            .position(|item| item.validator_account == *validator_info.key)
        {
            if validator_stake_list.validators[index].stake_count > 0 {
                return Err(StakePoolError::ValidatorHasStakes.into());
            }

            validator_stake_list.validators.remove(index);
        } else {
            return Err(StakePoolError::ValidatorNotFound.into());
        }
        validator_stake_list.serialize(&mut validator_stake_list_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes `UpdateListBalance` instruction.
    pub fn process_update_list_balance(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // Stake pool account
        let stake_pool_info = next_account_info(account_info_iter)?;
        // Account storing validator stake list
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        let withdraw_info = next_account_info(account_info_iter)?;
        let reserve_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let stake_program_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;
        let stake_history_info = next_account_info(account_info_iter)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        // Get stake pool stake (and check if it is initialized)
        let stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check validator stake account list storage
        if *validator_stake_list_info.key != stake_pool.validator_stake_list {
            return Err(StakePoolError::InvalidValidatorStakeList.into());
        }

        // Read validator stake list account and check if it is valid
        let mut validator_stake_list =
            ValidatorStakeList::deserialize(&validator_stake_list_info.data.borrow())?;
        if !validator_stake_list.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        stake_pool.check_authority_withdraw(withdraw_info.key, program_id, stake_pool_info.key)?;

        let (reserve_address, _) = Self::get_reserve_adderess(program_id, stake_pool_info.key);
        if *reserve_account_info.key != reserve_address {
            msg!(
                "Expected reserve to be {} but got {}",
                &reserve_address,
                reserve_account_info.key
            );
            return Err(ProgramError::InvalidArgument);
        }

        let mut changes = false;
        while let Some(validator_vote_info) = account_info_iter.next() {
            if let Some(validator_stake_record) = validator_stake_list
                .validators
                .iter_mut()
                .find(|v| v.validator_account == *validator_vote_info.key)
            {
                // ? if validator_stake_record.last_update_epoch >= clock.epoch
                validator_stake_record.balance = 0;
                let mut new_stake_count = 0u32;
                for index in 0..validator_stake_record.stake_count {
                    let stake_account_info = next_account_info(account_info_iter)?;

                    let stake_bump_seed = validator_stake_record.check_validator_stake_address(
                        program_id,
                        stake_pool_info.key,
                        index,
                        stake_account_info.key,
                    )?;

                    let stake_signer_seeds = &[
                        &validator_vote_info.key.to_bytes()[..32],
                        &stake_pool_info.key.to_bytes()[..32],
                        &unsafe { std::mem::transmute::<u32, [u8; 4]>(index) },
                        &[stake_bump_seed],
                    ];

                    // ? check stake_account_owner
                    let mut balance = **stake_account_info.lamports.borrow();
                    if balance > 0 {
                        if *stake_account_info.owner == stake::id() {
                            // return money back if there are some free
                            let stake_state: stake::StakeState =
                                deserialize(&stake_account_info.data.borrow()).or_else(|_| {
                                    msg!("Error reading stake {} state", stake_account_info.key);
                                    Err(ProgramError::InvalidAccountData)
                                })?;

                            let mut available_lamports = balance;
                            match stake_state {
                                StakeState::Uninitialized => {}
                                StakeState::Initialized(meta) => {
                                    available_lamports -= meta.rent_exempt_reserve;
                                }
                                StakeState::Stake(
                                    meta,
                                    stake::Stake {
                                        delegation,
                                        credits_observed,
                                    },
                                ) => {
                                    available_lamports -= meta.rent_exempt_reserve;
                                    available_lamports -= delegation.stake;
                                    // TODO: fix available calculation
                                }
                                StakeState::RewardsPool => {
                                    msg!(
                                        "Stake account {} is rewards pool",
                                        stake_account_info.key
                                    );
                                    return Err(StakePoolError::WrongStakeState.into());
                                }
                            }

                            if available_lamports > 0 {
                                let withdraw_signer_seeds: &[&[_]] = &[
                                    &stake_pool_info.key.to_bytes()[..32],
                                    Self::AUTHORITY_WITHDRAW,
                                    &[stake_pool.withdraw_bump_seed],
                                ];

                                invoke_signed(
                                    &stake::withdraw(
                                        stake_account_info.key,
                                        withdraw_info.key,
                                        reserve_account_info.key,
                                        available_lamports,
                                        None,
                                    ),
                                    &[
                                        stake_account_info.clone(),
                                        reserve_account_info.clone(),
                                        clock_info.clone(),
                                        stake_history_info.clone(),
                                        withdraw_info.clone(),
                                        stake_program_info.clone(),
                                    ],
                                    &[withdraw_signer_seeds],
                                )?;

                                balance -= available_lamports;
                            }

                            validator_stake_record.balance += balance;
                            new_stake_count = index + 1;
                        } else {
                            if *stake_account_info.owner == system_program::id() {
                                invoke_signed(
                                    &system_instruction::transfer(
                                        stake_account_info.key,
                                        reserve_account_info.key,
                                        balance,
                                    ),
                                    &[
                                        system_program_info.clone(),
                                        stake_account_info.clone(),
                                        reserve_account_info.clone(),
                                    ],
                                    &[stake_signer_seeds],
                                )?;
                                continue;
                            }

                            msg!(
                                "Invalid stake {} owner {}",
                                stake_account_info.key,
                                stake_account_info.owner
                            );
                            return Err(StakePoolError::WrongStakeState.into());
                        }
                    }
                }

                validator_stake_record.stake_count = new_stake_count;
                validator_stake_record.last_update_epoch = clock.epoch;
                changes = true;
            } else {
                msg!("Unexpected validator account {}", validator_vote_info.key);
                return Err(StakePoolError::ValidatorNotFound.into());
            }
        }

        if changes {
            validator_stake_list.serialize(&mut validator_stake_list_info.data.borrow_mut())?;
        }

        Ok(())
    }

    /// Processes `UpdatePoolBalance` instruction.
    pub fn process_update_pool_balance(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // Stake pool account
        let stake_pool_info = next_account_info(account_info_iter)?;
        // Account storing validator stake list
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        // Reserve account PDA
        let reserve_account_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        // Get stake pool stake (and check if it is initialized)
        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check validator stake account list storage
        if *validator_stake_list_info.key != stake_pool.validator_stake_list {
            return Err(StakePoolError::InvalidValidatorStakeList.into());
        }

        // Read validator stake list account and check if it is valid
        let validator_stake_list =
            ValidatorStakeList::deserialize(&validator_stake_list_info.data.borrow())?;
        if !validator_stake_list.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        let (reserve_address, _) = Self::get_reserve_adderess(program_id, stake_pool_info.key);
        if *reserve_account_info.key != reserve_address {
            msg!(
                "Expected reserve to be {} but got {}",
                &reserve_address,
                reserve_account_info.key
            );
            return Err(ProgramError::InvalidArgument);
        }

        let mut total_balance: u64 = **reserve_account_info.lamports.borrow();
        for validator_stake_record in validator_stake_list.validators {
            if validator_stake_record.last_update_epoch < clock.epoch {
                return Err(StakePoolError::StakeListOutOfDate.into());
            }
            total_balance += validator_stake_record.balance;
        }

        stake_pool.stake_total = total_balance;
        stake_pool.last_update_epoch = clock.epoch;
        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())?;

        Ok(())
    }

    const MIN_RESERVE_BALANCE: u64 = 1000000;

    /// Processes [Deposit](enum.Instruction.html).
    pub fn process_deposit(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // Stake pool
        let stake_pool_info = next_account_info(account_info_iter)?;
        // Stake pool withdraw authority
        let withdraw_info = next_account_info(account_info_iter)?;
        // Reserve account
        let reserve_account_info = next_account_info(account_info_iter)?;
        // User account to transfer SOLs from
        let source_user_info = next_account_info(account_info_iter)?;
        // User account to receive pool tokens
        let dest_user_info = next_account_info(account_info_iter)?;
        // Account to receive pool fee tokens
        let owner_fee_info = next_account_info(account_info_iter)?;
        // Pool token mint account
        let pool_mint_info = next_account_info(account_info_iter)?;
        // Rent sysvar account
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;
        // System program id
        let system_program_info = next_account_info(account_info_iter)?;
        // Pool token program id
        let token_program_info = next_account_info(account_info_iter)?;

        let (temp_account_info, native_mint_info) =
            if *source_user_info.owner != system_program::id() {
                (
                    Some(next_account_info(account_info_iter)?),
                    Some(next_account_info(account_info_iter)?),
                )
            } else {
                (None, None)
            };

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        // Check program ids
        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        stake_pool.check_authority_withdraw(withdraw_info.key, program_id, stake_pool_info.key)?;

        let (expected_reserve, reserve_bump) =
            Self::get_reserve_adderess(program_id, stake_pool_info.key);
        if *reserve_account_info.key != expected_reserve {
            msg!(
                "Expected reserve to be {} but got {}",
                &expected_reserve,
                reserve_account_info.key
            );
            return Err(ProgramError::IncorrectProgramId);
        }

        if stake_pool.owner_fee_account != *owner_fee_info.key {
            return Err(StakePoolError::InvalidFeeAccount.into());
        }
        if stake_pool.token_program_id != *token_program_info.key {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Check stake pool last update epoch
        /*
        if stake_pool.last_update_epoch < clock.epoch {
            return Err(StakePoolError::StakeListAndPoolOutOfDate.into());
        }*/

        let target_balance = **reserve_account_info.lamports.borrow() + amount;
        if !rent.is_exempt(target_balance, 0) {
            return Err(StakePoolError::FirstDepositIsTooSmall.into());
        }

        let pool_amount = stake_pool
            .calc_pool_deposit_amount(amount)
            .ok_or(StakePoolError::CalculationFailure)?;

        let fee_amount = stake_pool
            .calc_fee_amount(pool_amount)
            .ok_or(StakePoolError::CalculationFailure)?;

        let user_amount = pool_amount
            .checked_sub(fee_amount)
            .ok_or(StakePoolError::CalculationFailure)?;

        let withdraw_signer_seeds: &[&[_]] = &[
            &stake_pool_info.key.to_bytes()[..32],
            Self::AUTHORITY_WITHDRAW,
            &[stake_pool.withdraw_bump_seed],
        ];

        // Transfer user's SOLs to reserve
        if let (Some(temp_account_info), Some(native_mint_info)) =
            (temp_account_info, native_mint_info)
        {
            let (expected_temp_address, temp_bump) = Pubkey::find_program_address(
                &[&stake_pool_info.key.to_bytes()[..32], Self::TEMP_ACCOUNT],
                program_id,
            );

            if *temp_account_info.key != expected_temp_address {
                msg!(
                    "Expected temp account {} but got {}",
                    &expected_temp_address,
                    temp_account_info.key
                );
                return Err(ProgramError::InvalidArgument);
            }

            if *native_mint_info.key != spl_token::native_mint::id() {
                msg!("Expected native mint");
                return Err(ProgramError::InvalidArgument);
            }

            let temp_seeds = &[
                &stake_pool_info.key.to_bytes()[..32],
                Self::TEMP_ACCOUNT,
                &[temp_bump],
            ];

            let reserve_signer_seeds: &[&[u8]] = &[
                &stake_pool_info.key.to_bytes()[..32],
                Self::AUTHORITY_RESERVE,
                &[reserve_bump],
            ];

            invoke_signed(
                &system_instruction::create_account(
                    reserve_account_info.key,
                    temp_account_info.key,
                    rent.minimum_balance(spl_token::state::Account::LEN),
                    spl_token::state::Account::LEN as u64,
                    &spl_token::id(),
                ),
                &[
                    reserve_account_info.clone(),
                    temp_account_info.clone(),
                    system_program_info.clone(),
                ],
                &[temp_seeds, reserve_signer_seeds],
            )?;

            invoke(
                &spl_token::instruction::initialize_account(
                    token_program_info.key,
                    temp_account_info.key,
                    &spl_token::native_mint::id(),
                    withdraw_info.key,
                )?,
                &[
                    token_program_info.clone(),
                    temp_account_info.clone(),
                    native_mint_info.clone(),
                    withdraw_info.clone(),
                    rent_info.clone(),
                ],
            )?;

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program_info.key,
                    source_user_info.key,
                    temp_account_info.key,
                    withdraw_info.key,
                    &[],
                    amount,
                )?,
                &[
                    token_program_info.clone(),
                    source_user_info.clone(),
                    temp_account_info.clone(),
                    withdraw_info.clone(),
                ],
                &[withdraw_signer_seeds],
            )?;

            invoke_signed(
                &spl_token::instruction::close_account(
                    token_program_info.key,
                    temp_account_info.key,
                    reserve_account_info.key,
                    withdraw_info.key,
                    &[],
                )?,
                &[
                    token_program_info.clone(),
                    temp_account_info.clone(),
                    reserve_account_info.clone(),
                    withdraw_info.clone(),
                ],
                &[withdraw_signer_seeds],
            )?;
        } else {
            // Initial deposit must be enough
            if target_balance < Self::MIN_RESERVE_BALANCE {
                return Err(StakePoolError::FirstDepositIsTooSmall.into());
            }
            invoke(
                &system_instruction::transfer(
                    source_user_info.key,
                    reserve_account_info.key,
                    amount,
                ),
                &[
                    source_user_info.clone(),
                    reserve_account_info.clone(),
                    system_program_info.clone(),
                ],
            )?;
        }

        Self::token_mint_to(
            stake_pool_info.key,
            token_program_info.clone(),
            pool_mint_info.clone(),
            dest_user_info.clone(),
            withdraw_info.clone(),
            Self::AUTHORITY_WITHDRAW,
            stake_pool.withdraw_bump_seed,
            user_amount,
        )?;

        Self::token_mint_to(
            stake_pool_info.key,
            token_program_info.clone(),
            pool_mint_info.clone(),
            owner_fee_info.clone(),
            withdraw_info.clone(),
            Self::AUTHORITY_WITHDRAW,
            stake_pool.withdraw_bump_seed,
            fee_amount,
        )?;
        stake_pool.pool_total += pool_amount;
        stake_pool.stake_total += amount;
        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes [Withdraw](enum.Instruction.html).
    pub fn process_withdraw(
        program_id: &Pubkey,
        pool_amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // Stake pool
        let stake_pool_info = next_account_info(account_info_iter)?;
        // Stake pool deposit authority
        let withdraw_info = next_account_info(account_info_iter)?;
        // Reserve account
        let reserve_account_info = next_account_info(account_info_iter)?;
        // User account with pool tokens to burn from
        let burn_from_info = next_account_info(account_info_iter)?;
        // Pool token mint account
        let pool_mint_info = next_account_info(account_info_iter)?;
        // Target user account with SOLs
        let target_account_info = next_account_info(account_info_iter)?;
        // Rent sysvar account
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;
        // System program id
        let system_program_info = next_account_info(account_info_iter)?;
        // Pool token program id
        let token_program_info = next_account_info(account_info_iter)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        if stake_pool.token_program_id != *token_program_info.key {
            return Err(ProgramError::IncorrectProgramId);
        }

        stake_pool.check_authority_withdraw(withdraw_info.key, program_id, stake_pool_info.key)?;

        let (expected_reserve, reserve_bump) =
            Self::get_reserve_adderess(program_id, stake_pool_info.key);
        if *reserve_account_info.key != expected_reserve {
            msg!(
                "Expected reserve to be {} but got {}",
                &expected_reserve,
                reserve_account_info.key
            );
            return Err(ProgramError::IncorrectProgramId);
        }

        /*
        // Check stake pool last update epoch
        if stake_pool.last_update_epoch < clock.epoch {
            return Err(StakePoolError::StakeListAndPoolOutOfDate.into());
        }*/

        let stake_amount = stake_pool
            .calc_lamports_amount(pool_amount)
            .ok_or(StakePoolError::CalculationFailure)?;

        let reserve_balance = **reserve_account_info.lamports.borrow();
        if stake_amount > reserve_balance
            || reserve_balance - stake_amount < Self::MIN_RESERVE_BALANCE
            || !rent.is_exempt(reserve_balance - stake_amount, 0)
        {
            msg!(
                "Requested to withdraw {} but reserve contains only {}",
                stake_amount,
                reserve_balance
            );
            return Err(ProgramError::InsufficientFunds);
        }

        Self::token_burn(
            stake_pool_info.key,
            token_program_info.clone(),
            burn_from_info.clone(),
            pool_mint_info.clone(),
            withdraw_info.clone(),
            Self::AUTHORITY_WITHDRAW,
            stake_pool.withdraw_bump_seed,
            pool_amount,
        )?;

        let reserve_signer_seeds: &[&[u8]] = &[
            &stake_pool_info.key.to_bytes()[..32],
            Self::AUTHORITY_RESERVE,
            &[reserve_bump],
        ];

        invoke_signed(
            &system_instruction::transfer(
                reserve_account_info.key,
                target_account_info.key,
                stake_amount,
            ),
            &[
                system_program_info.clone(),
                reserve_account_info.clone(),
                target_account_info.clone(),
            ],
            &[reserve_signer_seeds],
        )?;

        stake_pool.pool_total -= pool_amount;
        stake_pool.stake_total -= stake_amount;
        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes [SetOwner](enum.Instruction.html).
    pub fn process_set_owner(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_pool_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let new_owner_info = next_account_info(account_info_iter)?;
        let new_owner_fee_info = next_account_info(account_info_iter)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check owner validity and signature
        stake_pool.check_owner(owner_info)?;

        // Check for owner fee account to have proper mint assigned
        if stake_pool.pool_mint
            != spl_token::state::Account::unpack_from_slice(&new_owner_fee_info.data.borrow())?.mint
        {
            return Err(StakePoolError::WrongAccountMint.into());
        }

        stake_pool.owner = *new_owner_info.key;
        stake_pool.owner_fee_account = *new_owner_fee_info.key;
        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())?;
        Ok(())
    }

    fn init_stake<'a>(
        validator_stake_info: &mut ValidatorStakeInfo,
        validator_vote_info: &AccountInfo<'a>,
        stake_account_info: &AccountInfo<'a>,
        stake_bump_seed: u8,
        stake_pool: &Pubkey,
        index: u32,
        lamports: u64,
        reserve_info: &AccountInfo<'a>,
        deposit_info: &AccountInfo<'a>,
        withdraw_info: &AccountInfo<'a>,
        stake_program_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        stake_config_info: &AccountInfo<'a>,
        rent_info: &AccountInfo<'a>,
        deposit_signer_seeds: &[&[u8]],
        reserve_signer_seeds: &[&[u8]],
    ) -> Result<(), ProgramError> {
        // Fund the associated token account with at least the minimum balance to be rent exempt
        if lamports < MIN_STAKE_ACCOUNT_BALANCE {
            return Err(ProgramError::InsufficientFunds);
        }

        let stake_signer_seeds = &[
            &validator_vote_info.key.to_bytes()[..32],
            &stake_pool.to_bytes()[..32],
            &unsafe { std::mem::transmute::<u32, [u8; 4]>(index) },
            &[stake_bump_seed],
        ];

        msg!(
            "Init stake #{} {} with {} balance",
            index,
            stake_account_info.key,
            lamports
        );

        // Create new stake account
        invoke_signed(
            &system_instruction::create_account(
                &reserve_info.key,
                &stake_account_info.key,
                lamports,
                std::mem::size_of::<stake::StakeState>() as u64,
                &stake::id(),
            ),
            &[reserve_info.clone(), stake_account_info.clone()],
            &[stake_signer_seeds, reserve_signer_seeds],
        )?;

        invoke(
            &stake::initialize(
                &stake_account_info.key,
                &stake::Authorized {
                    staker: *deposit_info.key,
                    withdrawer: *withdraw_info.key,
                },
                &stake::Lockup::default(),
            ),
            &[
                stake_account_info.clone(),
                rent_info.clone(),
                stake_program_info.clone(),
            ],
        )?;

        invoke_signed(
            &stake::delegate_stake(
                &stake_account_info.key,
                deposit_info.key,
                validator_vote_info.key,
            ),
            &[
                stake_account_info.clone(),
                deposit_info.clone(),
                validator_vote_info.clone(),
                clock_info.clone(),
                stake_history_info.clone(),
                stake_config_info.clone(),
                stake_program_info.clone(),
            ],
            &[&deposit_signer_seeds],
        )?;

        validator_stake_info.balance += lamports;

        Ok(())
    }

    fn redelegate_stake<'a>(
        validator_stake_info: &mut ValidatorStakeInfo,
        validator_vote_info: &AccountInfo<'a>,
        stake_account_info: &AccountInfo<'a>,
        index: u32,
        lamports: u64,
        reserve_info: &AccountInfo<'a>,
        deposit_info: &AccountInfo<'a>,
        system_program_info: &AccountInfo<'a>,
        stake_program_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        stake_config_info: &AccountInfo<'a>,
        deposit_signer_seeds: &[&[u8]],
        reserve_signer_seeds: &[&[u8]],
    ) -> Result<(), ProgramError> {
        msg!(
            "Redelegate stake #{} {} with {} more lamports",
            index,
            &stake_account_info.key,
            lamports
        );

        invoke_signed(
            &system_instruction::transfer(reserve_info.key, &stake_account_info.key, lamports),
            &[
                reserve_info.clone(),
                stake_account_info.clone(),
                system_program_info.clone(),
            ],
            &[reserve_signer_seeds],
        )?;

        // Redelegate
        invoke_signed(
            &stake::delegate_stake(
                &stake_account_info.key,
                deposit_info.key,
                validator_vote_info.key,
            ),
            &[
                stake_account_info.clone(),
                deposit_info.clone(),
                validator_vote_info.clone(),
                clock_info.clone(),
                stake_history_info.clone(),
                stake_config_info.clone(),
                stake_program_info.clone(),
            ],
            &[deposit_signer_seeds],
        )?;

        validator_stake_info.balance += lamports;

        Ok(())
    }

    /// Process DelegateReserve
    pub fn process_delegate_reserve(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instructions: &[DelegateReserveInstruction],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_pool_info = next_account_info(account_info_iter)?;
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        let withdraw_info = next_account_info(account_info_iter)?;
        let deposit_info = next_account_info(account_info_iter)?;
        let reserve_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        // Staking program id
        let stake_program_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        // let clock = &Clock::from_account_info(clock_info)?;
        // Stake history sysvar account
        let stake_history_info = next_account_info(account_info_iter)?;
        // let stake_history = &StakeHistory::from_account_info(stake_history_info)?;
        // Stake config sysvar account
        let stake_config_info = next_account_info(account_info_iter)?;
        // Rent sysvar account
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check validator stake account list storage
        if *validator_stake_list_info.key != stake_pool.validator_stake_list {
            return Err(StakePoolError::InvalidValidatorStakeList.into());
        }

        // Read validator stake list account and check if it is valid
        let mut validator_stake_list =
            ValidatorStakeList::deserialize(&validator_stake_list_info.data.borrow())?;
        if !validator_stake_list.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        stake_pool.check_authority_deposit(deposit_info.key, program_id, stake_pool_info.key)?;
        stake_pool.check_authority_withdraw(withdraw_info.key, program_id, stake_pool_info.key)?;

        let (reserve_address, reserve_bump) =
            Self::get_reserve_adderess(program_id, stake_pool_info.key);
        if *reserve_account_info.key != reserve_address {
            msg!(
                "Expected reserve to be {} but got {}",
                &reserve_address,
                reserve_account_info.key
            );
            return Err(ProgramError::InvalidArgument);
        }

        let deposit_signer_seeds: &[&[_]] = &[
            &stake_pool_info.key.to_bytes()[..32],
            Self::AUTHORITY_DEPOSIT,
            &[stake_pool.deposit_bump_seed],
        ];

        let reserve_signer_seeds: &[&[u8]] = &[
            &stake_pool_info.key.to_bytes()[..32],
            Self::AUTHORITY_RESERVE,
            &[reserve_bump],
        ];

        let lamports_available = **reserve_account_info.lamports.borrow() - rent.minimum_balance(0);
        let mut total_amount = 0;
        for instruction in instructions {
            let validator_vote_info = next_account_info(account_info_iter)?;
            let stake_account_info = next_account_info(account_info_iter)?;

            if let Some(validator) = validator_stake_list
                .validators
                .iter_mut()
                .find(|validator| *validator_vote_info.key == validator.validator_account)
            {
                if total_amount + instruction.amount > lamports_available {
                    return Err(ProgramError::InsufficientFunds);
                }

                if instruction.stake_index > validator.stake_count {
                    return Err(StakePoolError::InvalidStakeIndex.into());
                }

                let stake_bump_seed = validator.check_validator_stake_address(
                    program_id,
                    stake_pool_info.key,
                    instruction.stake_index,
                    stake_account_info.key,
                )?;

                if *stake_account_info.owner == system_program::id() {
                    // non existent account
                    Self::init_stake(
                        validator,
                        validator_vote_info,
                        stake_account_info,
                        stake_bump_seed,
                        stake_pool_info.key,
                        instruction.stake_index,
                        instruction.amount,
                        reserve_account_info,
                        deposit_info,
                        withdraw_info,
                        stake_program_info,
                        clock_info,
                        stake_history_info,
                        stake_config_info,
                        rent_info,
                        deposit_signer_seeds,
                        reserve_signer_seeds,
                    )?;
                } else {
                    // must be stake account
                    Self::redelegate_stake(
                        validator,
                        validator_vote_info,
                        stake_account_info,
                        instruction.stake_index as u32,
                        instruction.amount,
                        reserve_account_info,
                        deposit_info,
                        system_program_info,
                        stake_program_info,
                        clock_info,
                        stake_history_info,
                        stake_config_info,
                        deposit_signer_seeds,
                        reserve_signer_seeds,
                    )?;
                }

                if instruction.stake_index >= validator.stake_count {
                    validator.stake_count = instruction.stake_index + 1;
                }

                total_amount += instruction.amount;
            } else {
                msg!("Unexpected validator account {}", validator_vote_info.key);
                return Err(StakePoolError::ValidatorNotFound.into());
            }
        }

        validator_stake_list.serialize(&mut validator_stake_list_info.data.borrow_mut())?;

        // ? Only update stake total if the last state update epoch is current
        stake_pool.stake_total += total_amount;
        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process MergeStakes
    pub fn process_merge_stakes(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instructions: &[MergeStakesInstruction],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_pool_info = next_account_info(account_info_iter)?;
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        let deposit_info = next_account_info(account_info_iter)?;
        // Staking program id
        let stake_program_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        // let clock = &Clock::from_account_info(clock_info)?;
        // Stake history sysvar account
        let stake_history_info = next_account_info(account_info_iter)?;
        // let stake_history = &StakeHistory::from_account_info(stake_history_info)?;

        if stake_pool_info.owner != program_id {
            msg!(
                "Wrong owner {} for the stake pool {}. Expected {}",
                stake_pool_info.owner,
                stake_pool_info.key,
                program_id
            );
            return Err(StakePoolError::WrongOwner.into());
        }
        let stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check validator stake account list storage
        if *validator_stake_list_info.key != stake_pool.validator_stake_list {
            return Err(StakePoolError::InvalidValidatorStakeList.into());
        }

        // Read validator stake list account and check if it is valid
        let mut validator_stake_list =
            ValidatorStakeList::deserialize(&validator_stake_list_info.data.borrow())?;
        if !validator_stake_list.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        stake_pool.check_authority_deposit(deposit_info.key, program_id, stake_pool_info.key)?;

        let deposit_signer_seeds: &[&[_]] = &[
            &stake_pool_info.key.to_bytes()[..32],
            Self::AUTHORITY_DEPOSIT,
            &[stake_pool.deposit_bump_seed],
        ];

        let mut changed = false;
        for instruction in instructions {
            let main_stake_account_info = next_account_info(account_info_iter)?;
            let additional_stake_account_info = next_account_info(account_info_iter)?;

            if let Some(validator) = validator_stake_list
                .validators
                .iter_mut()
                .find(|validator| validator.validator_account == instruction.validator_address)
            {
                if instruction.main_index >= validator.stake_count
                    || instruction.additional_index >= validator.stake_count
                    || instruction.main_index >= instruction.additional_index
                {
                    return Err(StakePoolError::InvalidStakeIndex.into());
                }

                invoke_signed(
                    &stake::merge(
                        main_stake_account_info.key,
                        additional_stake_account_info.key,
                        deposit_info.key,
                    ),
                    &[
                        stake_program_info.clone(),
                        main_stake_account_info.clone(),
                        additional_stake_account_info.clone(),
                        clock_info.clone(),
                        stake_history_info.clone(),
                        deposit_info.clone(),
                    ],
                    &[deposit_signer_seeds],
                )?;

                if instruction.additional_index + 1 == validator.stake_count {
                    validator.stake_count -= 1;
                    changed = true;
                }
            } else {
                msg!(
                    "Unexpected validator account {}",
                    instruction.validator_address
                );
                return Err(StakePoolError::ValidatorNotFound.into());
            }
        }

        if changed {
            validator_stake_list.serialize(&mut validator_stake_list_info.data.borrow_mut())?;
        }

        Ok(())
    }

    /// Processes [Instruction](enum.Instruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = StakePoolInstruction::deserialize(input)?;
        match instruction {
            StakePoolInstruction::Initialize(init) => {
                msg!("Instruction: Init");
                Self::process_initialize(program_id, init, accounts)
            }
            StakePoolInstruction::AddValidator => {
                msg!("Instruction: AddValidator");
                Self::process_add_validator(program_id, accounts)
            }
            StakePoolInstruction::RemoveValidator => {
                msg!("Instruction: RemoveValidator");
                Self::process_remove_validator(program_id, accounts)
            }
            StakePoolInstruction::UpdateListBalance => {
                msg!("Instruction: UpdateListBalance");
                Self::process_update_list_balance(program_id, accounts)
            }
            StakePoolInstruction::UpdatePoolBalance => {
                msg!("Instruction: UpdatePoolBalance");
                Self::process_update_pool_balance(program_id, accounts)
            }
            StakePoolInstruction::Deposit(amount) => {
                msg!("Instruction: Deposit {}", amount);
                Self::process_deposit(program_id, amount, accounts)
            }
            StakePoolInstruction::Withdraw(amount) => {
                msg!("Instruction: Withdraw {}", amount);
                Self::process_withdraw(program_id, amount, accounts)
            }
            StakePoolInstruction::SetStakingAuthority => {
                panic!("Instruction: SetStakingAuthority");
                // Self::process_set_staking_auth(program_id, accounts)
            }
            StakePoolInstruction::SetOwner => {
                msg!("Instruction: SetOwner");
                Self::process_set_owner(program_id, accounts)
            }
            StakePoolInstruction::Credit(amount) => {
                panic!("Instruction: Credit {}", amount);
                // Self::process_test_deposit(program_id, amount, accounts)
            }
            StakePoolInstruction::Uncredit(amount) => {
                panic!("Instruction: Uncredit {}", amount);
                // Self::process_test_withdraw(program_id, amount, accounts)
            }
            StakePoolInstruction::DelegateReserve(instructions) => {
                msg!(
                    "Instruction: DelegateReserve with {} instructions",
                    instructions.len()
                );
                Self::process_delegate_reserve(program_id, accounts, &instructions)
            }
            StakePoolInstruction::MergeStakes(instructions) => {
                msg!(
                    "Instruction: MergeStakes with {} instructions",
                    instructions.len()
                );
                Self::process_merge_stakes(program_id, accounts, &instructions)
            }
        }
    }
}

impl PrintProgramError for StakePoolError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            StakePoolError::AlreadyInUse => msg!("Error: The account cannot be initialized because it is already being used"),
            StakePoolError::InvalidProgramAddress => msg!("Error: The program address provided doesn't match the value generated by the program"),
            StakePoolError::InvalidState => msg!("Error: The stake pool state is invalid"),
            StakePoolError::CalculationFailure => msg!("Error: The calculation failed"),
            StakePoolError::FeeTooHigh => msg!("Error: Stake pool fee > 1"),
            StakePoolError::WrongAccountMint => msg!("Error: Token account is associated with the wrong mint"),
            StakePoolError::NonZeroBalance => msg!("Error: Account balance should be zero"),
            StakePoolError::WrongOwner => msg!("Error: Wrong pool owner account"),
            StakePoolError::SignatureMissing => msg!("Error: Required signature is missing"),
            StakePoolError::InvalidValidatorStakeList => msg!("Error: Invalid validator stake list account"),
            StakePoolError::InvalidFeeAccount => msg!("Error: Invalid owner fee account"),
            StakePoolError::WrongPoolMint => msg!("Error: Specified pool mint account is wrong"),
            StakePoolError::WrongStakeState => msg!("Error: Stake account is not in the state expected by the program"),
            StakePoolError::UserStakeNotActive => msg!("Error: User stake is not active"),
            StakePoolError::ValidatorAlreadyAdded => msg!("Error: Stake account voting for this validator already exists in the pool"),
            StakePoolError::ValidatorNotFound => msg!("Error: Stake account for this validator not found in the pool"),
            StakePoolError::InvalidStakeAccountAddress => msg!("Error: Stake account address not properly derived from the validator address"),
            StakePoolError::StakeListOutOfDate => msg!("Error: Identify validator stake accounts with old balances and update them"),
            StakePoolError::StakeListAndPoolOutOfDate => msg!("Error: First update old validator stake account balances and then pool stake balance"),
            StakePoolError::UnknownValidatorStakeAccount => {
                msg!("Error: Validator stake account is not found in the list storage")
            }
            StakePoolError::WrongMintingAuthority => msg!("Error: Wrong minting authority set for mint pool account"),
            StakePoolError::MintHasInitialSupply => msg!("Error: Initial supply of mint is non zero"),
            StakePoolError::AccountNotRentExempt => msg!("Error: Account is not rent-exempt"),
            StakePoolError::ValidatorListOverflow => msg!("Error: Validator list is full. Can't add more validators"),
            StakePoolError::ValidatorHasStakes => msg!("Error: Withdraw all stakes before validator removal"),
            StakePoolError::FirstDepositIsTooSmall => msg!("Error: First deposit must be at least enough for rent"),
            StakePoolError::WrongCreditOwner => msg!("Error: Wrong credit owner"),
            StakePoolError::WrongCreditState => msg!("Error: Wrong credit satte"),
            StakePoolError::InvalidStakeIndex => msg!("Error: Invalid stake index"),
        }
    }
}
