#![no_std]

use week_timekeeping::Week;

dharitri_sc::imports!();

pub mod price_query;
pub mod project;
pub mod rewards;
pub mod validation;

pub type Timestamp = u64;

pub const MAX_PERCENTAGE: u32 = 100_000;
pub const DAY_IN_SECONDS: Timestamp = 24 * 60 * 60;
pub const WEEK_IN_SECONDS: Timestamp = 7 * DAY_IN_SECONDS;
pub const PRECISION: u64 = 1_000_000_000_000_000_000;

pub const DEFAULT_MIN_REWARDS_PERIOD: Week = 26;
pub const DEFAULT_MIN_WEEKLY_REWARDS_DOLLARS_VALUE: u64 = 1_000;
pub const USDC_DECIMALS: u32 = 6;

#[dharitri_sc::contract]
pub trait GrowthProgram:
    project::ProjectsModule
    + rewards::deposit::DepositRewardsModule
    + rewards::withdraw::WithdrawRewardsModule
    + rewards::energy::EnergyModule
    + rewards::claim::ClaimRewardsModule
    + rewards::common_rewards::CommonRewardsModule
    + price_query::PriceQueryModule
    + validation::ValidationModule
    + week_timekeeping::WeekTimekeepingModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + dharitri_sc_modules::pause::PauseModule
{
    /// Arguments:
    /// min_energy_per_reward_dollar: Scaled to PRECISION const.
    /// alpha: Percentage, scaled to MAX_PERCENTAGE const.
    /// beta: Percentage, scaled to MAX_PERCENTAGE const.
    /// signer: Public key of the signer, used to verify user claims
    #[init]
    fn init(
        &self,
        min_energy_per_reward_dollar: BigUint,
        alpha: BigUint,
        beta: BigUint,
        signer: ManagedAddress,
        router_address: ManagedAddress,
        safe_price_pair: ManagedAddress,
        energy_factory_address: ManagedAddress,
        simple_lock_address: ManagedAddress,
        usdc_token_id: TokenIdentifier,
        wmoax_token_id: TokenIdentifier,
    ) {
        self.require_sc_address(&router_address);
        self.require_sc_address(&safe_price_pair);
        self.require_sc_address(&simple_lock_address);
        self.require_valid_token_id(&usdc_token_id);
        self.require_valid_token_id(&wmoax_token_id);

        self.router_address().set(router_address);
        self.safe_price_pair().set(safe_price_pair);
        self.simple_lock_address().set(simple_lock_address);
        self.usdc_token_id().set(usdc_token_id);
        self.wmoax_token_id().set(wmoax_token_id);

        self.set_energy_factory_address(energy_factory_address);
        self.set_min_energy_per_reward_dollar(min_energy_per_reward_dollar);
        self.set_alpha(alpha);
        self.set_beta(beta);
        self.change_signer(signer);

        self.min_rewards_period().set(DEFAULT_MIN_REWARDS_PERIOD);

        let default_min_weekly_rewards_value =
            BigUint::from(DEFAULT_MIN_WEEKLY_REWARDS_DOLLARS_VALUE)
                * BigUint::from(10u32).pow(USDC_DECIMALS);
        self.min_weekly_rewards_value()
            .set(default_min_weekly_rewards_value);

        let current_epoch = self.blockchain().get_block_epoch();
        self.first_week_start_epoch().set(current_epoch);

        self.set_paused(true);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
