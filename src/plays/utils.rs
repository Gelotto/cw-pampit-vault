use cosmwasm_std::Uint128;

use crate::{error::ContractError, math::mul_ratio_u128, state::PLATFORM_FEE_PCT};

pub struct PairAmounts {
    pub fee: Uint128,
    pub base: Uint128,
    pub quote: Uint128,
}

pub fn prepare_pair_token_amounts(
    q0: Uint128,
    b0: Uint128,
    v0: Uint128,
) -> Result<PairAmounts, ContractError> {
    let q_fee_amount = mul_ratio_u128(q0, PLATFORM_FEE_PCT, 1_000_000u128)?;
    let b_reduction_amount = mul_ratio_u128(b0, PLATFORM_FEE_PCT, 1_000_000u128)?;
    let v_reduction_amount = mul_ratio_u128(v0, PLATFORM_FEE_PCT, 1_000_000u128)?;

    let b = b0 - b_reduction_amount;
    let q = b0 - q_fee_amount;
    let v = v0 - v_reduction_amount;

    // Compute the final amounts to send into the new trading pair
    let b_pair_amount = mul_ratio_u128(b, q, q + v)?;
    let q_pair_amount = q;

    Ok(PairAmounts {
        fee: q_fee_amount,
        base: b_pair_amount,
        quote: q_pair_amount,
    })
}
