//! Program state processor

use crate::{
    error::StakePoolError,
    instruction::{DelegateReserveInstruction, InitArgs, StakePoolInstruction},
    stake,
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
/*
impl<'a> ValidatorStakeDelegator<'a> {
    pub(crate) const ACCOUNT_COUNT: usize = EPOCHS_FOR_DELEGATION + 2;

    pub(crate) fn from_accounts(
        validator_stake_list: &ValidatorStakeList,
        program_id: &Pubkey,
        stake_pool: &Pubkey,
        accounts: &[AccountInfo<'a>],
    ) -> Result<Self, ProgramError> {
        if accounts.len() != EPOCHS_FOR_DELEGATION + 2 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let (validator_vote_info, stake_accounts_info) = accounts.split_first().unwrap();

        let mut stake_accounts_with_seeds = Vec::with_capacity(EPOCHS_FOR_DELEGATION + 1);

        if let Some((validator_index, validator)) = validator_stake_list
            .validators
            .iter()
            .enumerate()
            .find(|(_, validator)| *validator_vote_info.key == validator.validator_account)
        {
            for stake_index in 0..(EPOCHS_FOR_DELEGATION + 1) {
                let expected_stake =
                    validator.stake_address(program_id, stake_pool, stake_index as u8);
                if *stake_accounts_info[stake_index].key == expected_stake.0 {
                    stake_accounts_with_seeds
                        .push((stake_accounts_info[stake_index].clone(), expected_stake.1));
                } else {
                    msg!(
                        "Invalid {} stake account {} for validator {}",
                        stake_index,
                        stake_accounts_info[stake_index].key,
                        validator_vote_info.key
                    );
                    msg!("Expected {}", expected_stake.0);
                    return Err(StakePoolError::InvalidStakeAccountAddress.into());
                }
            }

            Ok(Self {
                validator_vote_info: validator_vote_info.clone(),
                stake_accounts_with_seeds,
                validator_index,
            })
        } else {
            msg!("Unexpected validator account {}", validator_vote_info.key);
            Err(StakePoolError::ValidatorNotFound.into())
        }
    }

    fn current_epoch_stake_index(
        &self,
        clock: &Clock,
        stake_history: &StakeHistory,
    ) -> Result<Option<usize>, ProgramError> {
        for index in 0..self.stake_accounts_with_seeds.len() {
            let stake_account_info = &self.stake_accounts_with_seeds[index].0;
            if *stake_account_info.owner == stake::id() {
                let stake_state: stake::StakeState = deserialize(
                    &self.stake_accounts_with_seeds[index].0.data.borrow(),
                )
                .or_else(|_| {
                    msg!(
                        "Error reading stake {} state",
                        self.stake_accounts_with_seeds[index].0.key
                    );
                    Err(ProgramError::InvalidAccountData)
                })?;
                match stake_state {
                    StakeState::Stake(
                        _meta,
                        stake::Stake {
                            delegation,
                            credits_observed: _,
                        },
                    ) => {
                        if delegation
                            .stake_activating_and_deactivating(
                                clock.epoch,
                                Some(stake_history),
                                true,
                            )
                            .0
                            == 0
                        {
                            return Ok(Some(index));
                        }
                    }
                    _ => return Err(StakePoolError::WrongStakeState.into()),
                }
            }
        }
        Ok(None)
    }

    fn free_stake_index(&self) -> Option<usize> {
        self.stake_accounts_with_seeds
            .iter()
            .position(|(stake_account_info, _)| *stake_account_info.owner == system_program::id())
    }

    fn init_stake(
        &self,
        validator_stake_info: &mut ValidatorStakeInfo,
        stake_pool: &Pubkey,
        index: usize,
        lamports: u64,
        funder_info: &AccountInfo<'a>,
        deposit_info: &AccountInfo<'a>,
        withdraw_info: &AccountInfo<'a>,
        stake_program_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        stake_config_info: &AccountInfo<'a>,
        rent_info: &AccountInfo<'a>,
        rent: &Rent,
        deposit_signer_seeds: &[&[u8]],
    ) -> Result<(), ProgramError> {
        // Fund the associated token account with at least the minimum balance to be rent exempt
        if lamports < 1 + rent.minimum_balance(std::mem::size_of::<stake::StakeState>()) {
            return Err(ProgramError::InsufficientFunds);
        }

        let (stake_account_info, _stake_bump_seed) = &self.stake_accounts_with_seeds[index];

        let stake_signer_seeds = &[
            &self.validator_vote_info.key.to_bytes()[..32],
            &stake_pool.to_bytes()[..32],
            &[index as u8],
            &[self.stake_accounts_with_seeds[index as usize].1],
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
                &funder_info.key,
                &stake_account_info.key,
                lamports,
                std::mem::size_of::<stake::StakeState>() as u64,
                &stake::id(),
            ),
            &[funder_info.clone(), stake_account_info.clone()],
            &[stake_signer_seeds],
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
                self.validator_vote_info.key,
            ),
            &[
                stake_account_info.clone(),
                deposit_info.clone(),
                self.validator_vote_info.clone(),
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

    fn delegate_to_the_current_epoch_stake(
        &self,
        validator_stake_info: &mut ValidatorStakeInfo,
        index: usize,
        lamports: u64,
        funder_info: &AccountInfo<'a>,
        deposit_info: &AccountInfo<'a>,
        system_program_info: &AccountInfo<'a>,
        stake_program_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        stake_config_info: &AccountInfo<'a>,
        deposit_signer_seeds: &[&[u8]],
    ) -> Result<(), ProgramError> {
        msg!(
            "Redelegate stake #{} {} with {} more lamports",
            index,
            &self.stake_accounts_with_seeds[index].0.key,
            lamports
        );

        invoke(
            &system_instruction::transfer(
                funder_info.key,
                &self.stake_accounts_with_seeds[index].0.key,
                lamports,
            ),
            &[
                funder_info.clone(),
                self.stake_accounts_with_seeds[index].0.clone(),
                system_program_info.clone(),
            ],
        )?;

        // Redelegate
        invoke_signed(
            &stake::delegate_stake(
                &self.stake_accounts_with_seeds[index].0.key,
                deposit_info.key,
                self.validator_vote_info.key,
            ),
            &[
                self.stake_accounts_with_seeds[index].0.clone(),
                deposit_info.clone(),
                self.validator_vote_info.clone(),
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

    fn next_full_stake(
        &self,
        starting_index: usize,
        clock: &Clock,
        stake_history: &StakeHistory,
    ) -> Result<Option<(usize, u64)>, ProgramError> {
        for index in starting_index..self.stake_accounts_with_seeds.len() {
            let stake_account_info = &self.stake_accounts_with_seeds[index].0;
            if *stake_account_info.owner == stake::id() {
                let stake_state: stake::StakeState = deserialize(
                    &self.stake_accounts_with_seeds[index].0.data.borrow(),
                )
                .or_else(|_| {
                    msg!(
                        "Error reading stake {} state",
                        self.stake_accounts_with_seeds[index].0.key
                    );
                    Err(ProgramError::InvalidAccountData)
                })?;
                match stake_state {
                    StakeState::Stake(
                        _,
                        stake::Stake {
                            delegation,
                            credits_observed,
                        },
                    ) => {
                        let (_effective, activating, deactivating) = delegation
                            .stake_activating_and_deactivating(
                                clock.epoch,
                                Some(stake_history),
                                true,
                            );
                        if activating == 0 && deactivating == 0 {
                            return Ok(Some((index, credits_observed)));
                        }
                    }
                    _ => return Err(StakePoolError::WrongStakeState.into()),
                }
            }
        }
        Ok(None)
    }

    fn merge_full_stakes(
        &self,
        main_index: usize,
        additional_index: usize,
        deposit_info: &AccountInfo<'a>,
        stake_program_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        deposit_signer_seeds: &[&[u8]],
    ) -> Result<(), ProgramError> {
        msg!(
            "Merge stake #{} {}",
            main_index,
            self.stake_accounts_with_seeds[main_index].0.key
        );
        msg!(
            " with stake #{} {}",
            additional_index,
            self.stake_accounts_with_seeds[additional_index].0.key
        );
        invoke_signed(
            &stake::merge(
                self.stake_accounts_with_seeds[main_index].0.key,
                self.stake_accounts_with_seeds[additional_index].0.key,
                deposit_info.key,
            ),
            &[
                self.stake_accounts_with_seeds[main_index].0.clone(),
                self.stake_accounts_with_seeds[additional_index].0.clone(),
                clock_info.clone(),
                stake_history_info.clone(),
                deposit_info.clone(),
                stake_program_info.clone(),
            ],
            &[deposit_signer_seeds],
        )
    }

    fn merge_all_full_stakes(
        &self,
        deposit_info: &AccountInfo<'a>,
        stake_program_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        deposit_signer_seeds: &[&[u8]],
        clock: &Clock,
        stake_history: &StakeHistory,
    ) -> Result<(), ProgramError> {
        if let Some((main_index, main_credits_observed)) =
            self.next_full_stake(0, clock, stake_history)?
        {
            let mut start = main_index + 1;
            while let Some((additional_index, additional_credits_observed)) =
                self.next_full_stake(start, clock, stake_history)?
            {
                if main_credits_observed == additional_credits_observed {
                    self.merge_full_stakes(
                        main_index,
                        additional_index,
                        deposit_info,
                        stake_program_info,
                        clock_info,
                        stake_history_info,
                        deposit_signer_seeds,
                    )?;
                } else {
                    msg!(
                        "Cannot merge stakes #{} {}: {} credits observed",
                        main_index,
                        self.stake_accounts_with_seeds[main_index].0.key,
                        main_credits_observed
                    );
                    msg!(
                        " with stake #{} {}: {} credits observed",
                        additional_index,
                        self.stake_accounts_with_seeds[additional_index].0.key,
                        additional_credits_observed
                    );
                }
                start = additional_index + 1;
            }
        }

        Ok(())
    }

    pub(crate) fn delegate(
        &self,
        validator_stake_info: &mut ValidatorStakeInfo,
        stake_pool: &Pubkey,
        lamports: u64,
        funder_info: &AccountInfo<'a>,
        deposit_info: &AccountInfo<'a>,
        withdraw_info: &AccountInfo<'a>,
        system_program_info: &AccountInfo<'a>,
        stake_program_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        stake_config_info: &AccountInfo<'a>,
        rent_info: &AccountInfo<'a>,
        rent: &Rent,
        deposit_signer_seeds: &[&[u8]],
        clock: &Clock,
        stake_history: &StakeHistory,
    ) -> Result<(), ProgramError> {
        self.merge_all_full_stakes(
            deposit_info,
            stake_program_info,
            clock_info,
            stake_history_info,
            deposit_signer_seeds,
            clock,
            stake_history,
        )?;

        if let Some(index) = self.current_epoch_stake_index(clock, stake_history)? {
            self.delegate_to_the_current_epoch_stake(
                validator_stake_info,
                index,
                lamports,
                funder_info,
                deposit_info,
                system_program_info,
                stake_program_info,
                clock_info,
                stake_history_info,
                stake_config_info,
                deposit_signer_seeds,
            )?;
            return Ok(());
        }

        if let Some(index) = self.free_stake_index() {
            self.init_stake(
                validator_stake_info,
                stake_pool,
                index,
                lamports,
                funder_info,
                deposit_info,
                withdraw_info,
                stake_program_info,
                clock_info,
                stake_history_info,
                stake_config_info,
                rent_info,
                rent,
                deposit_signer_seeds,
            )
        } else {
            panic!("Stake manager error: stake line overflow. Probably delegation is too slow!");
        }
    }
}*/

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

    /// Issue a stake_split instruction.
    #[allow(clippy::too_many_arguments)]
    pub fn stake_split<'a>(
        stake_pool: &Pubkey,
        stake_account: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        authority_type: &[u8],
        bump_seed: u8,
        amount: u64,
        split_stake: AccountInfo<'a>,
        reserved: AccountInfo<'a>,
        stake_program_info: AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let me_bytes = stake_pool.to_bytes();
        let authority_signature_seeds = [&me_bytes[..32], authority_type, &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = stake::split_only(stake_account.key, authority.key, amount, split_stake.key);

        invoke_signed(
            &ix,
            &[
                stake_account,
                reserved,
                authority,
                split_stake,
                stake_program_info,
            ],
            signers,
        )
    }

    /// Issue a stake_merge instruction.
    #[allow(clippy::too_many_arguments)]
    pub fn stake_merge<'a>(
        stake_pool: &Pubkey,
        stake_account: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        authority_type: &[u8],
        bump_seed: u8,
        merge_with: AccountInfo<'a>,
        clock: AccountInfo<'a>,
        stake_history: AccountInfo<'a>,
        stake_program_info: AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let me_bytes = stake_pool.to_bytes();
        let authority_signature_seeds = [&me_bytes[..32], authority_type, &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = stake::merge(merge_with.key, stake_account.key, authority.key);

        invoke_signed(
            &ix,
            &[
                merge_with,
                stake_account,
                clock,
                stake_history,
                authority,
                stake_program_info,
            ],
            signers,
        )
    }

    /// Issue a stake_set_owner instruction.
    #[allow(clippy::too_many_arguments)]
    pub fn stake_authorize<'a>(
        stake_pool: &Pubkey,
        stake_account: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        authority_type: &[u8],
        bump_seed: u8,
        new_staker: &Pubkey,
        staker_auth: stake::StakeAuthorize,
        reserved: AccountInfo<'a>,
        stake_program_info: AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let me_bytes = stake_pool.to_bytes();
        let authority_signature_seeds = [&me_bytes[..32], authority_type, &[bump_seed]];
        let signers = &[&authority_signature_seeds[..]];

        let ix = stake::authorize(stake_account.key, authority.key, new_staker, staker_auth);

        invoke_signed(
            &ix,
            &[stake_account, reserved, authority, stake_program_info],
            signers,
        )
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

    /*
    /// Transfer SOLs from user into stake accounts
    pub fn transfer_to_validator_stakes<
        'b,
        I: ExactSizeIterator<Item = ValidatorStakeDelegator<'b>>,
    >(
        system_program_info: &AccountInfo<'b>,
        source_account_info: &AccountInfo<'b>,
        targets: I,
        total_amount: u64,
        deposit_info: &AccountInfo<'b>,
        stake_program_info: &AccountInfo<'b>,
        clock_info: &AccountInfo<'b>,
        stake_history_info: &AccountInfo<'b>,
        stake_config_info: &AccountInfo<'b>,
        deposit_signer_seeds: &[&[u8]],
    ) -> Result<(), ProgramError> {
        let count = targets.len();
        let amount_per_account = total_amount / count as u64;
        let mut first = true;
        for target in targets {
            let ix = system_instruction::transfer(
                source_account_info.key,
                &target.stake_account_info.key,
                if first {
                    total_amount - amount_per_account * (count - 1) as u64
                } else {
                    amount_per_account
                },
            );
            first = false;
            invoke(
                &ix,
                &[
                    source_account_info.clone(),
                    target.stake_account_info.clone(),
                    system_program_info.clone(),
                ],
            )?;

            invoke_signed(
                &stake::delegate_stake(
                    &target.stake_account_info.key,
                    deposit_info.key,
                    target.validator_vote_info.key,
                ),
                &[
                    target.stake_account_info.clone(),
                    deposit_info.clone(),
                    target.validator_vote_info.clone(),
                    clock_info.clone(),
                    stake_history_info.clone(),
                    stake_config_info.clone(),
                    stake_program_info.clone(),
                ],
                &[deposit_signer_seeds],
            )?;
        }
        Ok(())
    }

    /// Withdraw from stakes
    pub fn withdraw_from_validator_stakes<'b, 'a: 'b, I: Iterator<Item = &'b AccountInfo<'a>>>(
        stake_program_info: &AccountInfo<'a>,
        target_account_info: &AccountInfo<'a>,
        source_accounts: I,
        total_amount: &mut u64,
        withdraw_info: &AccountInfo<'a>,
        clock_info: &AccountInfo<'a>,
        stake_history_info: &AccountInfo<'a>,
        deposit_withdraw_seeds: &[&[u8]],
    ) -> Result<(), ProgramError> {
        for source in source_accounts {
            if *total_amount == 0 {
                break;
            }
            let mut available_lamports: u64 = **(*source.lamports).borrow();
            let stake_state: stake::StakeState =
                deserialize(&source.data.borrow()).or_else(|_| {
                    msg!("Error reading stake {} state", source.key);
                    Err(ProgramError::InvalidAccountData)
                })?;
            match stake_state {
                StakeState::Uninitialized => {
                    msg!("Stake account {} is not initialized", source.key);
                    return Err(StakePoolError::UserStakeNotActive.into());
                }
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
                    msg!("Stake account {} is rewards pool. WTF?", source.key);
                    return Err(StakePoolError::UserStakeNotActive.into());
                }
            }
            let withdraw_lamports = u64::min(available_lamports, *total_amount);
            if withdraw_lamports > 0 {
                invoke_signed(
                    &stake::withdraw(
                        source.key,
                        withdraw_info.key,
                        target_account_info.key,
                        withdraw_lamports,
                        None,
                    ),
                    &[
                        source.clone(),
                        target_account_info.clone(),
                        clock_info.clone(),
                        stake_history_info.clone(),
                        withdraw_info.clone(),
                        stake_program_info.clone(),
                    ],
                    &[deposit_withdraw_seeds],
                )?;

                *total_amount -= withdraw_lamports;
            }
        }

        Ok(())
    }*/

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

        // Check if transaction was signed by owner
        if !owner_info.is_signer {
            return Err(StakePoolError::SignatureMissing.into());
        }

        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        // Stake pool account should not be already initialized
        if stake_pool.is_initialized() {
            return Err(StakePoolError::AlreadyInUse.into());
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
    /*
        /// Processes `CreateValidatorStakeAccount` instruction.
        pub fn process_create_validator_stake_account(
            program_id: &Pubkey,
            accounts: &[AccountInfo],
        ) -> ProgramResult {
            let account_info_iter = &mut accounts.iter();
            // Stake pool account
            let stake_pool_info = next_account_info(account_info_iter)?;
            // Account creation funder account
            let funder_info = next_account_info(account_info_iter)?;
            // Stake account to be created
            let stake_account_info = next_account_info(account_info_iter)?;
            // Validator this stake account will vote for
            let validator_info = next_account_info(account_info_iter)?;
            // Stake pool deposit authority
            let deposit_info = next_account_info(account_info_iter)?;
            // Stake pool withdraw authority
            let withdraw_info = next_account_info(account_info_iter)?;
            // Rent sysvar account
            let rent_info = next_account_info(account_info_iter)?;
            let rent = &Rent::from_account_info(rent_info)?;
            // System program id
            let system_program_info = next_account_info(account_info_iter)?;
            // Staking program id
            let stake_program_info = next_account_info(account_info_iter)?;
            // Clock sysvar account
            let clock_info = next_account_info(account_info_iter)?;
            let _clock = &Clock::from_account_info(clock_info)?;
            // Stake history sysvar account
            let stake_history_info = next_account_info(account_info_iter)?;
            let _stake_history = &StakeHistory::from_account_info(stake_history_info)?;
            // Stake config sysvar account
            let stake_config_info = next_account_info(account_info_iter)?;

            let stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;

            // Check program ids
            if *system_program_info.key != solana_program::system_program::id() {
                return Err(ProgramError::IncorrectProgramId);
            }
            if *stake_program_info.key != stake::id() {
                return Err(ProgramError::IncorrectProgramId);
            }

            // Check stake account address validity
            let (stake_address, bump_seed) = Self::find_stake_address_for_validator(
                &program_id,
                &validator_info.key,
                &stake_pool_info.key,
            );
            if stake_address != *stake_account_info.key {
                msg!(
                    "Expected stake address {} but got {}",
                    stake_address,
                    stake_account_info.key
                );
                return Err(StakePoolError::InvalidStakeAccountAddress.into());
            }

            // Check authority accounts
            stake_pool.check_authority_withdraw(withdraw_info.key, program_id, stake_pool_info.key)?;
            stake_pool.check_authority_deposit(deposit_info.key, program_id, stake_pool_info.key)?;

            if *stake_config_info.key != Pubkey::from_str(stake::STAKE_CONFIG).unwrap() {
                return Err(ProgramError::InvalidArgument);
            }

            let stake_account_signer_seeds: &[&[_]] = &[
                &validator_info.key.to_bytes()[..32],
                &stake_pool_info.key.to_bytes()[..32],
                &[bump_seed],
            ];

            let deposit_signer_seeds: &[&[_]] = &[
                &stake_pool_info.key.to_bytes()[..32],
                Self::AUTHORITY_DEPOSIT,
                &[stake_pool.deposit_bump_seed],
            ];

            // Fund the associated token account with the minimum balance to be rent exempt
            let required_lamports = 1 + rent.minimum_balance(std::mem::size_of::<stake::StakeState>());

            // Create new stake account
            invoke_signed(
                &system_instruction::create_account(
                    &funder_info.key,
                    &stake_account_info.key,
                    required_lamports,
                    std::mem::size_of::<stake::StakeState>() as u64,
                    &stake::id(),
                ),
                &[funder_info.clone(), stake_account_info.clone()],
                &[&stake_account_signer_seeds],
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
                    validator_info.key,
                ),
                &[
                    stake_account_info.clone(),
                    deposit_info.clone(),
                    validator_info.clone(),
                    clock_info.clone(),
                    stake_history_info.clone(),
                    stake_config_info.clone(),
                    stake_program_info.clone(),
                ],
                &[&deposit_signer_seeds],
            )
        }
    */
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

        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())?;

        Ok(())
    }

    /*
    /// Processes `RemoveValidatorStakeAccount` instruction.
    pub fn process_remove_validator_stake_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        // Stake pool account
        let stake_pool_info = next_account_info(account_info_iter)?;
        // Pool owner account
        let owner_info = next_account_info(account_info_iter)?;
        // Stake pool withdraw authority
        let withdraw_info = next_account_info(account_info_iter)?;
        // New stake authority
        let new_stake_authority_info = next_account_info(account_info_iter)?;
        // Account storing validator stake list
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        // Stake account to remove from the pool
        let stake_account_info = next_account_info(account_info_iter)?;
        // User account with pool tokens to burn from
        let burn_from_info = next_account_info(account_info_iter)?;
        // Pool token mint account
        let pool_mint_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;
        // Pool token program id
        let token_program_info = next_account_info(account_info_iter)?;
        // Staking program id
        let stake_program_info = next_account_info(account_info_iter)?;

        // Check program ids
        if *stake_program_info.key != stake::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get stake pool stake (and check if it is initialized)
        let mut stake_pool = StakePool::deserialize(&stake_pool_info.data.borrow())?;
        if !stake_pool.is_initialized() {
            return Err(StakePoolError::InvalidState.into());
        }

        // Check authority account
        stake_pool.check_authority_withdraw(withdraw_info.key, program_id, stake_pool_info.key)?;

        // Check owner validity and signature
        stake_pool.check_owner(owner_info)?;

        // Check stake pool last update epoch
        if stake_pool.last_update_epoch < clock.epoch {
            return Err(StakePoolError::StakeListAndPoolOutOfDate.into());
        }

        if stake_pool.token_program_id != *token_program_info.key {
            return Err(ProgramError::IncorrectProgramId);
        }
        if stake_pool.pool_mint != *pool_mint_info.key {
            return Err(StakePoolError::WrongPoolMint.into());
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

        let validator_account =
            Self::get_validator_checked(program_id, stake_pool_info, stake_account_info)?;

        if !validator_stake_list.contains(&validator_account) {
            return Err(StakePoolError::ValidatorNotFound.into());
        }

        // Update Withdrawer and Staker authority to the provided authority
        for authority in &[
            stake::StakeAuthorize::Withdrawer,
            stake::StakeAuthorize::Staker,
        ] {
            Self::stake_authorize(
                stake_pool_info.key,
                stake_account_info.clone(),
                withdraw_info.clone(),
                Self::AUTHORITY_WITHDRAW,
                stake_pool.withdraw_bump_seed,
                new_stake_authority_info.key,
                *authority,
                clock_info.clone(),
                stake_program_info.clone(),
            )?;
        }

        // Calculate and burn tokens
        let stake_lamports = **stake_account_info.lamports.borrow();
        let token_amount = stake_pool
            .calc_pool_withdraw_amount(stake_lamports)
            .ok_or(StakePoolError::CalculationFailure)?;
        Self::token_burn(
            stake_pool_info.key,
            token_program_info.clone(),
            burn_from_info.clone(),
            pool_mint_info.clone(),
            withdraw_info.clone(),
            Self::AUTHORITY_WITHDRAW,
            stake_pool.withdraw_bump_seed,
            token_amount,
        )?;

        // Remove validator from the list and save
        validator_stake_list
            .validators
            .retain(|item| item.validator_account != validator_account);
        validator_stake_list.serialize(&mut validator_stake_list_info.data.borrow_mut())?;

        // Save amounts to the stake pool state
        stake_pool.pool_total -= token_amount;
        // Only update stake total if the last state update epoch is current
        stake_pool.stake_total -= stake_lamports;
        stake_pool.serialize(&mut stake_pool_info.data.borrow_mut())?;

        Ok(())
    }*/

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
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;

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
                    validator_stake_record.check_validator_stake_address(
                        program_id,
                        stake_pool_info.key,
                        index,
                        stake_account_info.key,
                    )?;
                    // ? check stake_account_owner
                    let balance = **stake_account_info.lamports.borrow();
                    if balance > 0 {
                        validator_stake_record.balance += balance;
                        new_stake_count = index + 1;
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

        if !rent.is_exempt(**reserve_account_info.lamports.borrow() + amount, 0) {
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
        if stake_amount > reserve_balance || !rent.is_exempt(reserve_balance - stake_amount, 0) {
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
    pub fn process_set_owner(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_pool_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let new_owner_info = next_account_info(account_info_iter)?;
        let new_owner_fee_info = next_account_info(account_info_iter)?;

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

    /*
    fn process_test_deposit(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_pool_info = next_account_info(account_info_iter)?;
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        let deposit_info = next_account_info(account_info_iter)?;
        let user_wallet_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        // Staking program id
        let stake_program_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        // let _clock = &Clock::from_account_info(clock_info)?;
        // Stake history sysvar account
        let stake_history_info = next_account_info(account_info_iter)?;
        // let _stake_history = &StakeHistory::from_account_info(stake_history_info)?;
        // Stake config sysvar account
        let stake_config_info = next_account_info(account_info_iter)?;

        let validator_stake_accounts = account_info_iter.as_slice().chunks(2);

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

        let validator_stake_accounts = validator_stake_accounts
            .map(|pair| {
                let stake_delegator = ValidatorStakeDelegator {
                    stake_account_info: pair[0].clone(),
                    validator_vote_info: pair[1].clone(),
                };

                if validator_stake_list
                    .validators
                    .iter()
                    .find(|validator| {
                        *stake_delegator.validator_vote_info.key == validator.validator_account
                    })
                    .is_some()
                {
                    if Self::find_stake_address_for_validator(
                        program_id,
                        &stake_delegator.validator_vote_info.key,
                        stake_pool_info.key,
                    )
                    .0 != *stake_delegator.stake_account_info.key
                    {
                        msg!(
                            "Invalid stake account {} for validator {}",
                            stake_delegator.stake_account_info.key,
                            stake_delegator.validator_vote_info.key
                        );
                        return Err(StakePoolError::InvalidStakeAccountAddress.into());
                    }
                } else {
                    msg!(
                        "Unexpected validator account {}",
                        stake_delegator.validator_vote_info.key
                    );
                    return Err(StakePoolError::ValidatorNotFound.into());
                }

                Ok(stake_delegator)
            })
            .collect::<Result<Vec<_>, ProgramError>>()?;

        if *system_program_info.key != solana_program::system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let deposit_signer_seeds: &[&[_]] = &[
            &stake_pool_info.key.to_bytes()[..32],
            Self::AUTHORITY_DEPOSIT,
            &[stake_pool.deposit_bump_seed],
        ];

        Self::transfer_to_validator_stakes(
            system_program_info,
            user_wallet_info,
            validator_stake_accounts.into_iter(),
            amount,
            deposit_info,
            stake_program_info,
            clock_info,
            stake_history_info,
            stake_config_info,
            deposit_signer_seeds,
        )?;

        // TODO: update stats

        Ok(())
    }

    /// Process TestWithdraw
    fn process_test_withdraw(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_pool_info = next_account_info(account_info_iter)?;
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        let withdraw_info = next_account_info(account_info_iter)?;
        let user_wallet_info = next_account_info(account_info_iter)?;
        let stake_program_info = next_account_info(account_info_iter)?;
        // Clock sysvar account
        let clock_info = next_account_info(account_info_iter)?;
        // let _clock = &Clock::from_account_info(clock_info)?;
        // Stake history sysvar account
        let stake_history_info = next_account_info(account_info_iter)?;
        // let _stake_history = &StakeHistory::from_account_info(stake_history_info)?;
        let stake_accounts = account_info_iter.as_slice();

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

        for stake in stake_accounts {
            if !validator_stake_list.validators.iter().any(|validator| {
                Self::find_stake_address_for_validator(
                    program_id,
                    &validator.validator_account,
                    stake_pool_info.key,
                )
                .0 == *stake.key
            }) {
                msg!("Unexpected stake account {}", stake.key);
                return Err(StakePoolError::InvalidStakeAccountAddress.into());
            }
        }

        let withdraw_signer_seeds: &[&[_]] = &[
            &stake_pool_info.key.to_bytes()[..32],
            Self::AUTHORITY_WITHDRAW,
            &[stake_pool.withdraw_bump_seed],
        ];

        let mut amount_left = amount;
        Self::withdraw_from_validator_stakes(
            stake_program_info,
            user_wallet_info,
            stake_accounts.clone().into_iter(),
            &mut amount_left,
            withdraw_info,
            clock_info,
            stake_history_info,
            withdraw_signer_seeds,
        )?;

        if amount_left > 0 {
            msg!(
                "Can be withdraw only {} of {} requested",
                amount - amount_left,
                amount
            );
            return Err(ProgramError::InsufficientFunds);
        }

        // TODO: update balance

        Ok(())
    }*/

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

                let stake_bump_seed = validator.check_validator_stake_address(
                    program_id,
                    stake_pool_info.key,
                    instruction.stake_index as u32,
                    stake_account_info.key,
                )?;

                if *stake_account_info.owner == system_program::id() {
                    Self::init_stake(
                        validator,
                        validator_vote_info,
                        stake_account_info,
                        stake_bump_seed,
                        stake_pool_info.key,
                        instruction.stake_index as u32,
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

                if instruction.stake_index as u32 >= validator.stake_count {
                    validator.stake_count = instruction.stake_index as u32 + 1;
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
            StakePoolInstruction::RemoveValidatorStakeAccount => {
                panic!("Instruction: RemoveValidatorStakeAccount");
                // Self::process_remove_validator_stake_account(program_id, accounts)
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
            StakePoolError::FirstDepositIsTooSmall => msg!("Error: First deposit must be at least enough for rent"),
            StakePoolError::WrongCreditOwner => msg!("Error: Wrong credit owner"),
            StakePoolError::WrongCreditState => msg!("Error: Wrong credit satte"),
        }
    }
}
