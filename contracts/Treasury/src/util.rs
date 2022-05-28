use crate::error::ContractError;

use cosmwasm_std::{ Storage, Uint128, Addr, StdResult, StdError, Response, Env, QuerierWrapper, Querier};
use terraswap::querier;
use crate::state::{
    OPERATOR, INITIALIZED, START_TIME, EPOCH, EPOCH_SUPPLY_CONTRACTION_LEFT,
    EXCLUDED_FROM_TOTALSUPPLY, TOMB, TBOND, TSHARE, MASONRY,
    BOND_TREASURY, TOMB_ORACLE, TOMB_PRICE_ONE, TOMB_PRICE_CEILING,
    SEIGNIORAGE_SAVED, SUPPLY_TIERS, MAX_EXPANSION_TIERS,
    MAX_SUPPLY_EXPANSION_PERCENT, BOND_DEPLETION_FLOOR_PERCENT,
    SEIGNIORAGE_EXPANSION_FLOOR_PERCENT, MAX_SUPPLY_CONTRACTION_PERCENT,
    MAX_DEBT_RATIO_PERCENT, BOND_SUPPLY_EXPANSION_PERCENT,
    BOOTSTRAP_EPOCHS, BOOTSTRAP_SUPPLY_EXPANSION_PERCENT,
    PREVIOUS_EPOCH_TOMB_PRICE, MAX_DISCOUNT_RATE,
    MAX_PREMIUM_RATE, DISCOUNT_PERCENT, PREMIUM_PERCENT,
    PREMIUM_THRESHOLD, MINTING_FACTOR_FOR_PAYING_DEBT,
    DAOFUND, DAOFUND_SHARED_PERCENT, DEVFUND,
    DEVFUND_SHARED_PERCENT
};
use crate::contract::{PERIOD};
use BasisAsset::msg::{QueryMsg as BasisAssetQuery};
use IMasonry::msg::{QueryMsg as MasonryQuery};
use Oracle::msg::{QueryMsg as OracleQuery};
use terraswap::querier::{query_token_balance};
use terraswap::asset::{AssetInfo};

pub const ETHER: u128 = 1_000_000_000_000_000_000u128;

pub fn check_onlyoperator(storage: &dyn Storage, sender: Addr) -> Result<Response, ContractError> {
    let operator = OPERATOR.load(storage)?;
    if operator != sender {
        return Err(ContractError::Unauthorized{});
    }
    Ok(Response::new())
}
pub fn check_condition(storage: &dyn Storage, env: Env) -> Result<Response, ContractError>{
    let starttime = START_TIME.load(storage)?;

    if Uint128::from(env.block.time.seconds()) < starttime {
        return Err(ContractError::NotStartedYet{ });
    }
    Ok(Response::new())
}
pub fn check_epoch(storage: &mut dyn Storage, env: Env) -> Result<Response, ContractError>{

    if Uint128::from(env.block.time.seconds()) < next_epoch_point(storage)? {
        return Err(ContractError::NotOpenedYet{ });
    }
    let mut epoch = EPOCH.load(storage)?;
    epoch += Uint128::from(1u128);
    EPOCH.save(storage, &epoch);
    
    // epochSupplyContractionLeft = (getTombPrice() > tombPriceCeiling) ? 0 : getTombCirculatingSupply().mul(maxSupplyContractionPercent).div(10000);

    Ok(Response::new())
}
pub fn get_basisasset_operator(querier: QuerierWrapper, token: Addr) -> StdResult<Addr> {
    let operator: Addr = querier.query_wasm_smart(
        token, 
        &BasisAssetQuery::Operator {  }
    )?;
    Ok(operator)
}
pub fn get_masonry_operator(querier: QuerierWrapper, token: Addr) -> StdResult<Addr> {
    let operator: Addr = querier.query_wasm_smart(
        token, 
        &MasonryQuery::Operator {  }
    )?;
    Ok(operator)
}
pub fn check_operator(storage: &dyn Storage, querier: QuerierWrapper,  env: Env) -> Result<Response, ContractError>{
    let tomb = get_basisasset_operator(querier, TOMB.load(storage)?)?;
    let tbond = get_basisasset_operator(querier, TBOND.load(storage)?)?;
    let tshare = get_basisasset_operator(querier, TSHARE.load(storage)?)?;
    let mansonry = get_masonry_operator(querier, MASONRY.load(storage)?)?;
    let contract = env.contract.address;

    if tomb != contract || tbond != contract || tshare != contract || mansonry != contract {
        return Err(ContractError::NeedMorePermission{ })
    }
    Ok(Response::new())
}
pub fn check_not_initialized(storage: &dyn Storage) -> Result<Response, ContractError> {
    let initialized = INITIALIZED.load(storage)?;
    if initialized {
        return Err(ContractError::AlreadyInitialized{})
    }
    Ok(Response::new())
}

pub fn is_initialized(storage: &dyn Storage) -> StdResult<bool>{
    Ok(INITIALIZED.load(storage)?)
}
// epoch
pub fn next_epoch_point(storage: &dyn Storage) -> StdResult<Uint128> {
    let starttime = START_TIME.load(storage)?;
    let epoch = EPOCH.load(storage)?;
    Ok(starttime + epoch * Uint128::from(PERIOD))
}

// oracle
pub fn get_tomb_price(storage: &dyn Storage, querier: &QuerierWrapper) -> StdResult<Uint128> {
    let tomb = TOMB.load(storage)?;    

    let tomb_asset = AssetInfo::Token { contract_addr: tomb.to_string()};
    let price: Uint128 = querier.query_wasm_smart(
        TOMB_ORACLE.load(storage)?,
        &OracleQuery::Consult { 
                token: tomb_asset, 
                amount_in: Uint128::from(ETHER)
            }
    )?;

    Ok(price)
}
pub fn get_tomb_updated_price(storage: &dyn Storage, querier: &QuerierWrapper) -> StdResult<Uint128> {
    let tomb = TOMB.load(storage)?;    

    let tomb_asset = AssetInfo::Token { contract_addr: tomb.to_string()};
    let price: Uint128 = querier.query_wasm_smart(
        TOMB_ORACLE.load(storage)?,
        &OracleQuery::Twap { 
                token: tomb_asset, 
                amount_in: Uint128::from(ETHER) 
            }
    )?;

    Ok(price)
}
pub fn get_reserve(storage: &dyn Storage) -> StdResult<Uint128> {
    Ok(SEIGNIORAGE_SAVED.load(storage)?)
}

pub fn get_burnable_tomb_left(storage: &dyn Storage, querier: &QuerierWrapper) -> StdResult<Uint128> {
    let tomb_price = get_tomb_price(storage, querier)?;
    let tomb_price_one = TOMB_PRICE_ONE.load(storage)?;
    let mut burnable_tomb_left = Uint128::zero();
    if tomb_price <= tomb_price_one {
        let tomb_supply = get_tomb_circulating_supply(storage, querier)?;
        let max_debt_ratio_percent = MAX_DEBT_RATIO_PERCENT.load(storage)?;
        let bond_max_supply = tomb_supply * max_debt_ratio_percent / Uint128::from(10_000u128);
        
        let bond_supply: Uint128 = querier.query_wasm_smart(
            TBOND.load(storage)?, 
            &BasisAssetQuery::TotalSupply {  }
        )?;

        if bond_max_supply > bond_supply {
            let max_mintable_bond = bond_max_supply - bond_supply;
            let max_burnable_tomb = max_mintable_bond * tomb_price / Uint128::from(ETHER);
            let epoch_supply_contract_left = EPOCH_SUPPLY_CONTRACTION_LEFT.load(storage)?;
            
            if epoch_supply_contract_left > max_burnable_tomb {
                burnable_tomb_left = max_burnable_tomb;
            } else {
                burnable_tomb_left = epoch_supply_contract_left
            }
        }
    }
    Ok(burnable_tomb_left)
}

pub fn get_redeemable_bonds(storage: &dyn Storage, querier: &QuerierWrapper, env: Env) -> StdResult<Uint128> {
    let tomb_price = get_tomb_price(storage, querier)?;
    let tomb_price_ceiling = TOMB_PRICE_CEILING.load(storage)?;

    let mut redeemable_bonds = Uint128::zero();
    if tomb_price > tomb_price_ceiling {
        let total_tomb = query_token_balance(querier, 
            TOMB.load(storage)?, env.contract.address)?;

        let rate = get_bond_premium_rate(storage, querier)?;
        if rate > Uint128::zero() {
            redeemable_bonds = total_tomb * Uint128::from(ETHER) / rate;
        }
    }
    Ok(redeemable_bonds)
}

pub fn get_bond_discount_rate(storage: &dyn Storage, querier: &QuerierWrapper) -> StdResult<Uint128>{
    let tomb_price = get_tomb_price(storage, querier)?;
    let tomb_price_one = TOMB_PRICE_ONE.load(storage)?;
    let mut rate = Uint128::zero();
    if tomb_price <=  tomb_price_one {
        let discount_percent = DISCOUNT_PERCENT.load(storage)?;
        if discount_percent == Uint128::zero() {
            // no discount
            rate = tomb_price_one;
        } else {
            let bond_amount = tomb_price_one * Uint128::from(ETHER) / tomb_price;
            let discount_amount = (bond_amount - tomb_price_one) * discount_percent / Uint128::from(10_000u128);
            rate = tomb_price_one + discount_amount;
            
            let max_discount_rate = MAX_DISCOUNT_RATE.load(storage)?;
            if max_discount_rate > Uint128::zero() && rate > max_discount_rate {
                rate = max_discount_rate;
            }
        }
    }
    Ok(rate)
}

pub fn get_bond_premium_rate(storage: &dyn Storage, querier: &QuerierWrapper) -> StdResult<Uint128> {
    let tomb_price = get_tomb_price(storage, querier)?;
    let tomb_price_ceiling = TOMB_PRICE_CEILING.load(storage)?;
    let mut rate = Uint128::zero();

    if tomb_price > tomb_price_ceiling {
        let tomb_price_one = TOMB_PRICE_ONE.load(storage)?;
        let premium_threshold = PREMIUM_THRESHOLD.load(storage)?;
        let tomb_price_premium_threshold = tomb_price_one * premium_threshold / Uint128::from(100u128);
        
        if tomb_price >= tomb_price_premium_threshold {
            //Price > 1.10
            let premium_percent = PREMIUM_PERCENT.load(storage)?;
            let premium_amount = (tomb_price - tomb_price_one) * premium_percent / Uint128::from(10_000u128);

            rate = tomb_price_one + premium_amount;
            let max_premium_rate = MAX_PREMIUM_RATE.load(storage)?;
            if max_premium_rate > Uint128::zero() && rate > max_premium_rate {
                rate = max_premium_rate;
            }
        } else {
            // no premium bonus
            rate = tomb_price_one;
        }
    }
    Ok(rate)
}

pub fn get_tomb_circulating_supply(storage: &dyn Storage, querier: &QuerierWrapper) -> StdResult<Uint128>{
    let tomb = TOMB.load(storage)?;
    let total_supply: Uint128 = querier.query_wasm_smart(
        tomb.clone(), 
        &BasisAssetQuery::TotalSupply {  }
    )?;
    let mut balance_excluded = Uint128::zero();
    let excluded_from_totalsupply = EXCLUDED_FROM_TOTALSUPPLY.load(storage)?;
    for entry_id in 0 .. excluded_from_totalsupply.len() {
        let balance = query_token_balance(querier, 
            tomb.clone(), excluded_from_totalsupply[entry_id].clone())?;
        balance_excluded += balance;
    }
    Ok(total_supply - balance_excluded)
}

pub fn get_total_supply(querier: &QuerierWrapper, token: Addr) -> StdResult<Uint128>{
    let total_supply: Uint128 = querier.query_wasm_smart(
        token, 
        &BasisAssetQuery::TotalSupply {  }
    )?;
    Ok(total_supply)
}