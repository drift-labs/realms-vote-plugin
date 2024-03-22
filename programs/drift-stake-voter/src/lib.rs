use anchor_lang::prelude::*;

pub mod error;

mod instructions;
use instructions::*;

pub mod state;

pub mod tools;

declare_id!("H5cjESgwpXoTf7szHBch17noet6DdTFCrMgHcTro5cLb");

#[program]
pub mod drift_stake_voter {

    use super::*;

    pub fn create_registrar(ctx: Context<CreateRegistrar>, spot_market_index: u16) -> Result<()> {
        log_version();
        instructions::create_registrar(ctx, spot_market_index)
    }
    pub fn create_voter_weight_record(
        ctx: Context<CreateVoterWeightRecord>,
        governing_token_owner: Pubkey,
    ) -> Result<()> {
        log_version();
        instructions::create_voter_weight_record(ctx, governing_token_owner)
    }
    pub fn update_voter_weight_record(ctx: Context<UpdateVoterWeightRecord>) -> Result<()> {
        log_version();
        instructions::update_voter_weight_record(ctx)
    }
}

fn log_version() {
    // TODO: Check if Anchor allows to log it before instruction is deserialized
    msg!("VERSION:{:?}", env!("CARGO_PKG_VERSION"));
}
