use anchor_lang::prelude::*;
use drift::{
    math::insurance::if_shares_to_vault_amount, state::insurance_fund_stake::InsuranceFundStake,
    state::spot_market::SpotMarket,
};

pub fn get_user_token_stake(
    insurance_fund_stake: &InsuranceFundStake,
    spot_market: &SpotMarket,
    insurance_fund_vault_balance: u64,
    now: i64,
) -> Result<u64> {
    // small warm up period: insurance_fund_stake must be more than 5 seconds old
    if insurance_fund_stake.last_valid_ts > now - 5 {
        msg!("insurance_fund_stake.last_valid_ts > now - 5");
        return Ok(0);
    }

    // insurance stake must fully staked
    if insurance_fund_stake.last_withdraw_request_shares != 0 {
        msg!("insurance_fund_stake.last_withdraw_request_shares != 0");
        return Ok(0);
    }
    /*
    // insurance fund must be configured with sufficiently unstaking_period
    if spot_market.insurance_fund.unstaking_period < 100 {
        return Ok(0);
    } */

    let user_stake_in_tokens = if_shares_to_vault_amount(
        insurance_fund_stake.checked_if_shares(spot_market)?,
        spot_market.insurance_fund.total_shares,
        insurance_fund_vault_balance,
    )?;

    Ok(user_stake_in_tokens)
}
