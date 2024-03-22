use crate::error::DriftVoterError;
use crate::state::*;
use crate::tools::drift_tools::get_user_token_stake;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use drift::program::Drift;
use drift::state::insurance_fund_stake::InsuranceFundStake;
use drift::state::spot_market::SpotMarket;
use spl_governance::state::token_owner_record::get_token_owner_record_data_for_realm_and_governing_mint;
use std::ops::Deref;

/// Updates VoterWeightRecord based on Realm DAO membership
/// The membership is evaluated via a valid TokenOwnerRecord which must belong to one of the configured spl-governance instances
///
/// This instruction sets VoterWeightRecord.voter_weight which is valid for the current slot only
/// and must be executed inside the same transaction as the corresponding spl-gov instruction
#[derive(Accounts)]
pub struct UpdateVoterWeightRecord<'info> {
    #[account(
        constraint = registrar.drift_program_id == drift_program.key(),
        seeds = [
            b"registrar".as_ref(),
            voter_weight_record.realm.key().as_ref(),
            voter_weight_record.governing_token_mint.key().as_ref()
        ],
        bump
    )]
    pub registrar: Account<'info, Registrar>,

    #[account(
        mut,
        constraint = voter_weight_record.realm == registrar.realm
        @ DriftVoterError::InvalidVoterWeightRecordRealm,

        constraint = voter_weight_record.governing_token_mint == registrar.governing_token_mint
        @ DriftVoterError::InvalidVoterWeightRecordMint,
        // can't do a seeds constraint here because we don't yet know the token_owner_record governing_token_owner
    )]
    pub voter_weight_record: Account<'info, VoterWeightRecord>,

    /// TokenOwnerRecord for any of the configured spl-governance instances
    /// CHECK: Owned by any of the spl-governance instances specified in registrar.governance_program_configs
    #[account()]
    pub token_owner_record: UncheckedAccount<'info>,

    #[account(
        constraint = spot_market.load()?.market_index == registrar.spot_market_index,
        constraint = spot_market.load()?.mint == registrar.governing_token_mint,
    )]
    pub spot_market: AccountLoader<'info, SpotMarket>,
    #[account(
        constraint = spot_market.load()?.insurance_fund.vault == insurance_fund_vault.key(),
    )]
    pub insurance_fund_vault: Account<'info, TokenAccount>,
    #[account(
        constraint = insurance_fund_stake.load()?.authority == voter_weight_record.governing_token_owner.key(),
        constraint = insurance_fund_stake.load()?.market_index == registrar.spot_market_index,
        // check that this is owned by the drift program specified by the registrar
    )]
    pub insurance_fund_stake: AccountLoader<'info, InsuranceFundStake>,
    pub drift_program: Program<'info, Drift>,
}

pub fn update_voter_weight_record(ctx: Context<UpdateVoterWeightRecord>) -> Result<()> {
    let voter_weight_record: &mut Account<'_, VoterWeightRecord> =
        &mut ctx.accounts.voter_weight_record;

    // Get base spl-gov weight. One could use chaining for this but... sounds annoying!!
    let token_owner_record = ctx.accounts.token_owner_record.to_account_info();
    let registrar = &ctx.accounts.registrar;

    // this also performs checks that the token owner record uses the right mint, realm, and program
    let record = get_token_owner_record_data_for_realm_and_governing_mint(
        &registrar.governance_program_id,
        &token_owner_record.clone(),
        &registrar.realm,
        &registrar.governing_token_mint,
    )
    .unwrap();

    // Ensure that the token owner record belongs to the same governing token owner
    require_eq!(
        voter_weight_record.governing_token_owner,
        record.governing_token_owner,
        DriftVoterError::GoverningTokenOwnerMustMatch
    );

    let spl_gov_deposit_weight = record.governing_token_deposit_amount;
    msg!("SPL-GOV weight: {}", spl_gov_deposit_weight);

    // Get drift insurance pool deposit weight
    let drift_stake_weight = get_user_token_stake(
        ctx.accounts.insurance_fund_stake.load()?.deref(),
        ctx.accounts.spot_market.load()?.deref(),
        ctx.accounts.insurance_fund_vault.amount,
        Clock::get()?.unix_timestamp,
    )?;

    msg!("Drift stake weight: {}", drift_stake_weight);

    let total_weight = spl_gov_deposit_weight.saturating_add(drift_stake_weight);
    msg!("Total weight: {}", total_weight);

    // Setup voter_weight
    voter_weight_record.voter_weight = total_weight;

    // Record is only valid as of the current slot
    voter_weight_record.voter_weight_expiry = Some(Clock::get()?.slot);

    // Set action and target to None to indicate the weight is valid for any action and target
    voter_weight_record.weight_action = None;
    voter_weight_record.weight_action_target = None;

    Ok(())
}
