use anchor_lang::prelude::*;

use crate::error::BondingCurveLaunchpadError;

/// Linear curve -> cost = slope x supply + base_price
#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
    pub base_price: u64,
    pub slope: u64,
    pub scale: u64,
    pub supply_cap: u64,
    pub supply: u64,
    pub reserve_amount: u64,
    pub bump: u8,
}
impl BondingCurve {
    /// Integral of linear curve form 0 -> amount
    ///
    /// integral_cost = (slope * amount ^ 2 + 2 * amount * base_price ) / 2
    pub fn integral_cost(&self, circulating_supply: u64) -> Result<u128> {
        let circulating_supply = circulating_supply as u128;
        let first = (self.slope as u128)
            .checked_mul(circulating_supply)
            .ok_or(BondingCurveLaunchpadError::Overflow)?
            .checked_mul(circulating_supply)
            .ok_or(BondingCurveLaunchpadError::Overflow)?;
        let second = 2u128
            .checked_mul(circulating_supply)
            .ok_or(BondingCurveLaunchpadError::Overflow)?
            .checked_mul(self.base_price as u128)
            .ok_or(BondingCurveLaunchpadError::Overflow)?;
        let sum = first
            .checked_add(second)
            .ok_or(BondingCurveLaunchpadError::Overflow)?;
        let den = 2u128
            .checked_mul(self.scale as u128)
            .ok_or(BondingCurveLaunchpadError::Overflow)?;
        Ok(sum / den)
    }

    pub fn cost_between_supplies(
        &self,
        from_circulating_supply: u64,
        to_circulating_supply: u64,
    ) -> Result<u64> {
        require!(
            to_circulating_supply >= from_circulating_supply,
            BondingCurveLaunchpadError::InvalidArguments
        );
        let cost = self
            .integral_cost(to_circulating_supply)?
            .checked_sub(self.integral_cost(from_circulating_supply)?)
            .ok_or(BondingCurveLaunchpadError::Underflow)?;
        u64::try_from(cost).map_err(|_| BondingCurveLaunchpadError::Overflow.into())
    }

    pub fn cost_to_buy_tokens(&self, num_tokens: u64) -> Result<u64> {
        require!(num_tokens > 0, BondingCurveLaunchpadError::InvalidArguments);
        let new_supply = self
            .supply
            .checked_add(num_tokens)
            .ok_or(BondingCurveLaunchpadError::Overflow)?;
        require!(
            new_supply <= self.supply_cap,
            BondingCurveLaunchpadError::SupplyCapExceeded
        );
        self.cost_between_supplies(self.supply, new_supply)
    }

    pub fn refund_for_tokens(&self, num_tokens: u64) -> Result<u64> {
        require!(num_tokens > 0, BondingCurveLaunchpadError::InvalidArguments);
        let new_supply = self
            .supply
            .checked_sub(num_tokens)
            .ok_or(BondingCurveLaunchpadError::Underflow)?;

        self.cost_between_supplies(new_supply, self.supply)
    }

    pub fn tokens_within_budget(&self, budget: u64) -> Result<(u64, u64)> {
        require!(budget > 0, BondingCurveLaunchpadError::InvalidArguments);
        let (mut start, mut end) = (0u64, self.supply_cap - self.supply);
        while start < end {
            let mid = start + (end - start + 1) / 2;
            if self.cost_between_supplies(self.supply, self.supply + mid)? <= budget {
                start = mid;
            } else {
                end = mid - 1;
            }
        }
        require!(start > 0, BondingCurveLaunchpadError::InvalidArguments);
        let cost = self.cost_between_supplies(self.supply, self.supply + start)?;
        Ok((start, cost))
    }

    pub fn apply_buy(&mut self, launchpad_token: u64, reserve_token: u64) -> Result<()> {
        self.supply = self
            .supply
            .checked_add(launchpad_token)
            .ok_or(BondingCurveLaunchpadError::Overflow)?;
        require!(
            self.supply <= self.supply_cap,
            BondingCurveLaunchpadError::SupplyCapExceeded
        );
        self.reserve_amount = self
            .reserve_amount
            .checked_add(reserve_token)
            .ok_or(BondingCurveLaunchpadError::Overflow)?;
        Ok(())
    }

    pub fn apply_sell(&mut self, launchpad_token: u64, reserve_token: u64) -> Result<()> {
        self.supply = self
            .supply
            .checked_sub(launchpad_token)
            .ok_or(BondingCurveLaunchpadError::Underflow)?;
        self.reserve_amount = self
            .reserve_amount
            .checked_sub(reserve_token)
            .ok_or(BondingCurveLaunchpadError::Underflow)?;
        Ok(())
    }
}
