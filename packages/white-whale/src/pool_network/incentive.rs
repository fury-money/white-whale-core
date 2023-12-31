use std::collections::{BTreeMap, HashMap};
use std::fmt;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal256, Uint128};

use crate::pool_network::asset::{Asset, AssetInfo};

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the LP token that the incentive should be tied to.
    pub lp_asset: AssetInfo,
    /// Fee distributor contract address.
    pub fee_distributor_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Makes a snapshot of the current global weight, at the current epoch.
    TakeGlobalWeightSnapshot {},
    /// Opens a new liquidity flow
    OpenFlow {
        /// The epoch at which the flow will start. If unspecified, the flow will start at the
        /// current epoch.
        start_epoch: Option<u64>,
        /// The epoch at which the flow should end. If unspecified, the flow will default to end at
        /// 14 epochs from the current one.
        end_epoch: Option<u64>,
        /// The type of distribution curve. If unspecified, the distribution will be linear.
        curve: Option<Curve>,
        /// The asset to be distributed in this flow.
        flow_asset: Asset,
        /// If set, the label will be used to identify the flow, in addition to the flow_id.
        flow_label: Option<String>,
    },
    /// Closes an existing liquidity flow.
    ///
    /// Sender of the message must either be the contract admin or the creator of the flow.
    CloseFlow {
        /// The identifier of the flow to close.
        flow_identifier: FlowIdentifier,
    },
    /// Creates a new position to earn flow rewards.
    OpenPosition {
        /// The amount to add to the position.
        amount: Uint128,
        /// The amount of time (in seconds) before the LP tokens can be redeemed.
        unbonding_duration: u64,
        /// The receiver of the new position.
        ///
        /// This is mostly used for the frontend helper contract.
        ///
        /// If left empty, defaults to the message sender.
        receiver: Option<String>,
    },
    /// Expands an existing position to earn more flow rewards.
    ExpandPosition {
        /// The amount to add to the existing position.
        amount: Uint128,
        /// The unbond completion timestamp to identify the position to add to.
        unbonding_duration: u64,
        /// The receiver of the expanded position.
        ///
        /// This is mostly used for the frontend helper contract.
        ///
        /// If left empty, defaults to the message sender.
        receiver: Option<String>,
    },
    /// Closes an existing position to stop earning flow rewards.
    ClosePosition {
        /// The unbonding duration of the position to close.
        unbonding_duration: u64,
    },
    /// Withdraws the LP tokens from a closed position once the unbonding duration has passed.
    Withdraw {},
    /// Claims the flow rewards.
    Claim {},
    /// Expands an existing flow.
    ExpandFlow {
        /// The identifier of the flow to expand, whether an id or a label.
        flow_identifier: FlowIdentifier,
        /// The epoch at which the flow should end. If not set, the flow will be expanded a default value of 14 epochs.
        end_epoch: Option<u64>,
        /// The asset to expand this flow with.
        flow_asset: Asset,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

/// Represents a flow.
#[cw_serde]
pub struct Flow {
    /// A unique identifier of the flow.
    pub flow_id: u64,
    /// An alternative flow label.
    pub flow_label: Option<String>,
    /// The account which opened the flow and can manage it.
    pub flow_creator: Addr,
    /// The asset the flow was created to distribute.
    pub flow_asset: Asset,
    /// The amount of the `flow_asset` that has been claimed so far.
    pub claimed_amount: Uint128,
    /// The type of curve the flow has.
    pub curve: Curve,
    //todo not doing anything for now
    /// The epoch at which the flow starts.
    pub start_epoch: u64,
    /// The epoch at which the flow ends.
    pub end_epoch: u64,
    /// emitted tokens
    pub emitted_tokens: HashMap<u64, Uint128>,
    /// A map containing the amount of tokens it was expanded to at a given epoch. This is used
    /// to calculate the right amount of tokens to distribute at a given epoch when a flow is expanded.
    pub asset_history: BTreeMap<u64, (Uint128, u64)>,
}

/// Represents a position that accumulates flow rewards.
///
/// An address can have multiple incentive positions active at once.
#[cw_serde]
pub struct OpenPosition {
    /// The amount of LP tokens that are put up to earn incentives.
    pub amount: Uint128,
    /// Represents the amount of time in seconds the user must wait after unbonding for the LP tokens to be released.
    pub unbonding_duration: u64,
}

impl fmt::Display for OpenPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OpenPosition(amount: {}, unbonding_duration: {})",
            self.amount, self.unbonding_duration
        )
    }
}

/// Represents a position that has moved from the [`OpenPosition`] state.
///
/// This position is no longer accumulating rewards, and the underlying tokens are claimable after `unbonding_duration`.
#[cw_serde]
pub struct ClosedPosition {
    /// The amount of LP tokens that the user is unbonding in this position.
    pub amount: Uint128,
    /// The block timestamp when the user can withdraw the position to retrieve the underlying `amount` of LP tokens.
    pub unbonding_timestamp: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieves the current contract configuration.
    #[returns(ConfigResponse)]
    Config {},
    /// Retrieves a specific flow. If start_epoch and end_epoch are set, the asset_history and
    /// emitted_tokens will be filtered to only include epochs within the range. The maximum gap between
    /// the start_epoch and end_epoch is 100 epochs.
    #[returns(FlowResponse)]
    Flow {
        /// The id of the flow to find.
        flow_identifier: FlowIdentifier,
        /// If set, filters the asset_history and emitted_tokens to only include epochs from start_epoch.
        start_epoch: Option<u64>,
        /// If set, filters the asset_history and emitted_tokens to only include epochs until end_epoch.
        end_epoch: Option<u64>,
    },
    /// Retrieves the current flows. If start_epoch and end_epoch are set, the asset_history and
    /// emitted_tokens will be filtered to only include epochs within the range. The maximum gap between
    /// the start_epoch and end_epoch is 100 epochs.
    #[returns(FlowsResponse)]
    Flows {
        /// If set, filters the asset_history and emitted_tokens to only include epochs from start_epoch.
        start_epoch: Option<u64>,
        /// If set, filters the asset_history and emitted_tokens to only include epochs until end_epoch.
        end_epoch: Option<u64>,
    },
    /// Retrieves the positions for an address.
    #[returns(PositionsResponse)]
    Positions {
        /// The address to get positions for.
        address: String,
    },
    /// Retrieves the rewards for an address.
    #[returns(RewardsResponse)]
    Rewards {
        /// The address to get all the incentive rewards for.
        address: String,
    },
    /// Retrieves the rewards for an address.
    #[returns(GlobalWeightResponse)]
    GlobalWeight {
        /// The epoch to get the global weight for.
        epoch_id: u64,
    },
    /// Retrieves the rewards/weight share of an address for the current epoch.
    #[returns(RewardsShareResponse)]
    CurrentEpochRewardsShare {
        /// The address to query the rewards share for.
        address: String,
    },
}

/// Stores the reply data set in the response when instantiating an incentive contract.
#[cw_serde]
pub struct InstantiateReplyCallback {
    /// The address of the LP token that is tied to the incentive contract.
    pub lp_asset: AssetInfo,
}

/// Represents the configuration of the incentive contract.
#[cw_serde]
pub struct Config {
    /// The address of the incentive factory.
    pub factory_address: Addr,

    /// Fee distributor contract.
    pub fee_distributor_address: Addr,

    /// The LP token asset tied to the incentive contract.
    pub lp_asset: AssetInfo,
}

/// The type of distribution curve to exist.
#[cw_serde]
pub enum Curve {
    /// A linear curve that releases assets as we approach the end of the flow period.
    Linear,
}

impl std::fmt::Display for Curve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Curve::Linear => write!(f, "Linear"),
        }
    }
}

pub type ConfigResponse = Config;

#[cw_serde]
pub struct FlowResponse {
    //TODO why is this returning a Option<Flow>? why not a flow directly?
    /// The flow that was searched for.
    pub flow: Option<Flow>,
}

#[cw_serde]
pub struct FlowsResponse {
    /// The current flows.
    pub flows: Vec<Flow>,
}

#[cw_serde]
pub enum QueryPosition {
    /// Represents a position that a user has deposited, but not yet begun to unbond.
    OpenPosition {
        /// The amount of LP tokens the user deposited into the position.
        amount: Uint128,
        /// The amount of time (in seconds) the user must wait after they begin the unbonding process.
        unbonding_duration: u64,
        /// The amount of weight the position has.
        weight: Uint128,
    },
    /// Represents a position that a user has initiated the unbonding process on. The position may or may not be withdrawable.
    ClosedPosition {
        /// The amount of LP tokens the user deposited into the position, and will receive after they withdraw.
        amount: Uint128,
        /// The timestamp (in seconds) the user unbonded at.
        unbonding_timestamp: u64,
        /// The amount of weight the position has.
        weight: Uint128,
    },
}

#[cw_serde]
pub struct PositionsResponse {
    /// The current time of the blockchain.
    pub timestamp: u64,
    /// All the positions a user has.
    pub positions: Vec<QueryPosition>,
}

#[cw_serde]
pub struct RewardsResponse {
    /// The rewards that is available to a user if they executed the `claim` function at this point.
    pub rewards: Vec<Asset>,
}

#[cw_serde]
pub struct GlobalWeightResponse {
    /// the global weight of the incentive contract for the given epoch
    pub global_weight: Uint128,
    /// Epoch id for which the global weight is calculated
    pub epoch_id: u64,
}

#[cw_serde]
pub struct RewardsShareResponse {
    pub address: Addr,
    pub global_weight: Uint128,
    pub address_weight: Uint128,
    pub share: Decimal256,
    pub epoch_id: u64,
}

#[cw_serde]
pub enum FlowIdentifier {
    Id(u64),
    Label(String),
}

impl fmt::Display for FlowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowIdentifier::Id(flow_id) => write!(f, "flow_id: {}", flow_id),
            FlowIdentifier::Label(flow_label) => write!(f, "flow_label: {}", flow_label),
        }
    }
}
