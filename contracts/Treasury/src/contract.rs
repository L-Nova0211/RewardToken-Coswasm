#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    Addr, to_binary, DepsMut, Env, MessageInfo, Response, QuerierWrapper,
    Uint128, CosmosMsg, WasmMsg, Storage, StdResult, StdError
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
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
use crate::util::{ETHER, check_onlyoperator, check_operator, check_condition, get_tomb_price, 
    get_bond_discount_rate, get_tomb_circulating_supply, get_total_supply,
    get_bond_premium_rate, check_epoch
};
use terraswap::querier::{query_token_balance};
use Oracle::msg::{ExecuteMsg as OracleMsg};
use BasisAsset::msg::{ExecuteMsg as BasisAssetMsg};
use IMasonry::msg::{ExecuteMsg as MasonryMsg};
use BondTreasury::msg::{QueryMsg as BondTreasuryQuery};

// version info for migration info
const CONTRACT_NAME: &str = "Treasury";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PERIOD: u128 = 21_600u128;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Initialize{ tomb, tbond, tshare, tomb_oracle, masonry, genesis_pool, bond_treasury, start_time }
            => try_initialize(deps, env, info, tomb, tbond, tshare, tomb_oracle, masonry, genesis_pool, bond_treasury, start_time  ),

        ExecuteMsg::SetOperator { operator } 
            =>  try_setoperator(deps, env, info, operator),

        ExecuteMsg::SetMasonry { masonry } 
            =>  try_setmasonry(deps, env, info, masonry),

        ExecuteMsg::SetBondTreasury { bond_treasury } 
            =>  try_setbondtreasury(deps, env, info, bond_treasury),

        ExecuteMsg::SetTombOracle { tomb_oracle } 
            =>  try_settomboracle(deps, env, info, tomb_oracle),
        
        ExecuteMsg::SetTombPriceCeiling { tomb_price_ceiling } 
            =>  try_settombpriceceiling(deps, env, info, tomb_price_ceiling),          
            
        ExecuteMsg::SetMaxSupplyExpansionPercents{ max_supply_expansion_percent }
            =>  try_set_max_supply_expansion_percents(deps, env, info, max_supply_expansion_percent),
    
        ExecuteMsg::SetSupplyTiersEntry{ index, value }
            =>  try_set_supply_tiers_entry(deps, env, info, index, value),

        ExecuteMsg::SetBondDepletionFloorPercent{ bond_depletion_floor_percent }
            =>  try_set_bond_depletion_floor_percent(deps, env, info, bond_depletion_floor_percent),

        ExecuteMsg::SetMaxSupplyContractionPercent{ max_supply_contraction_percent }
            =>  try_set_max_supply_contraction_percent(deps, env, info, max_supply_contraction_percent),

        ExecuteMsg::SetMaxDebtRatioPercent{ max_debt_ratio_percent }
            =>  try_set_max_debt_ratio_percent(deps, env, info, max_debt_ratio_percent),

        ExecuteMsg::SetBootstrap{ bootstrap_epochs, bootstrap_supply_expansion_percent }
            =>  try_set_bootstrap(deps, env, info, bootstrap_epochs, bootstrap_supply_expansion_percent),
        
        ExecuteMsg::SetExtraFunds { dao_fund, dao_fund_shared_percent, dev_fund, dev_fund_shared_percent }
            =>  try_set_extra_funds(deps, env, info, dao_fund, dao_fund_shared_percent, dev_fund, dev_fund_shared_percent),
        
        ExecuteMsg::SetMaxDiscountRate { max_discount_rate }
            =>  try_set_max_discount_rate(deps, env, info, max_discount_rate),

        ExecuteMsg::SetMaxPremiumRate { max_premium_rate }
            =>  try_set_max_premium_rate(deps, env, info, max_premium_rate),

        ExecuteMsg::SetDiscountPercent { discount_percent }
            =>  try_set_discount_percent(deps, env, info, discount_percent),

        ExecuteMsg::SetPremiumThreshold { premium_threshold }
            =>  try_set_premium_threshold(deps, env, info, premium_threshold),

        ExecuteMsg::SetPremiumPercent { premium_percent }
            =>  try_set_premium_percent(deps, env, info, premium_percent),

        ExecuteMsg::SetMintingFactorForPayingDebt { minting_factor_for_paying_debt }
            =>  try_set_minting_factor_for_paying_debt(deps, env, info, minting_factor_for_paying_debt),

        ExecuteMsg::SetBondSupplyExpansionPercent { bond_supply_expansion_percent }
            =>  try_set_bond_supply_expansion_percent(deps, env, info, bond_supply_expansion_percent),

        ExecuteMsg::UpdateTombPrice{ }
            =>  try_update_tomb_price(deps, env, info),

        ExecuteMsg::BuyBonds { tomb_amount, target_price }
            =>  try_buy_bonds(deps, env, info, tomb_amount, target_price),

        ExecuteMsg::RedeemBonds { bond_amount, target_price }
            => try_redeem_bonds(deps, env, info, bond_amount, target_price),
            
        ExecuteMsg::SendToMasonry { amount }
            => try_send_to_masonry(deps, env, info, amount),

        ExecuteMsg::SendToBondTreasury { amount }
            => try_send_to_bond_treasury(deps, env, info, amount),

        ExecuteMsg::AllocateSeigniorage {  }
            => try_allocate_seigniorage(deps, env, info),

        ExecuteMsg::GovernanceRecoverUnsupported { token, amount, to }
            => try_governance_recover_unsupported(deps, env, info, token, amount, to),

        ExecuteMsg::MasonrySetOperator { operator }
            => try_masonry_set_operator(deps, info, operator),

        ExecuteMsg::MasonrySetLockup { withdraw_lockup_epochs, reward_lockup_epochs }
            => try_masonry_set_lockup(deps, info, withdraw_lockup_epochs, reward_lockup_epochs),

        ExecuteMsg::MasonryAllocationSeigniorage { amount }
            => try_masonry_allocation_seigniorage(deps, info, amount),
        
        ExecuteMsg::MasonryGovernanceRecoverUnsupported { token, amount, to }
            =>  try_masonry_governance_recover_unsupported(deps, env, info, token, amount, to)
    }
}
pub fn try_initialize(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tomb: Addr,
    tbond: Addr,
    tshare: Addr,
    tomb_oracle: Addr,
    masonry: Addr,
    genesis_pool: Addr,
    bond_treasury: Addr,
    start_time: Uint128
)
    -> Result<Response, ContractError>
{
    TOMB.save(deps.storage, &tomb)?;
    TBOND.save(deps.storage, &tbond)?;
    TSHARE.save(deps.storage, &tshare)?;
    TOMB_ORACLE.save(deps.storage, &tomb_oracle)?;
    MASONRY.save(deps.storage, &masonry)?;
    BOND_TREASURY.save(deps.storage, &bond_treasury)?;
    START_TIME.save(deps.storage, &start_time)?;

    TOMB_PRICE_ONE.save(deps.storage, &Uint128::from(ETHER))?;
    TOMB_PRICE_CEILING.save(deps.storage, &(Uint128::from(ETHER) * Uint128::from(101u128) / Uint128::from(100u128)));

    // exclude contracts from total supply
    let mut excluded_from_total_supply: Vec<Addr> = Vec::new();
    excluded_from_total_supply.push(genesis_pool);
    excluded_from_total_supply.push(bond_treasury);
    EXCLUDED_FROM_TOTALSUPPLY.save(deps.storage, &excluded_from_total_supply);

    // Dynamic max expansion percent
    let supply_tiers = vec![Uint128::zero(), Uint128::from(500_000u128)*Uint128::from(ETHER),
            Uint128::from(1_000_000u128) * Uint128::from(ETHER), Uint128::from(1_500_000u128) * Uint128::from(ETHER), 
            Uint128::from(2_000_000u128) * Uint128::from(ETHER), Uint128::from(5_000_000u128) * Uint128::from(ETHER), 
            Uint128::from(10_000_000u128) * Uint128::from(ETHER), Uint128::from(20_000_000u128) * Uint128::from(ETHER), 
            Uint128::from(50_000_000u128) * Uint128::from(ETHER)];
    SUPPLY_TIERS.save(deps.storage, &supply_tiers)?;

    let max_expansion_tiers = vec![Uint128::from(450u128), Uint128::from(400u128), 
            Uint128::from(350u128), Uint128::from(300u128), Uint128::from(250u128), 
            Uint128::from(200u128), Uint128::from(150u128), Uint128::from(125u128), 
            Uint128::from(100u128) ];
    MAX_EXPANSION_TIERS.save(deps.storage, &max_expansion_tiers)?;

    MAX_SUPPLY_EXPANSION_PERCENT.save(deps.storage, &Uint128::from(400u128))?;
    BOND_DEPLETION_FLOOR_PERCENT.save(deps.storage, &Uint128::from(10_000u128))?;
    SEIGNIORAGE_EXPANSION_FLOOR_PERCENT.save(deps.storage, &Uint128::from(3_500u128))?;
    MAX_SUPPLY_CONTRACTION_PERCENT.save(deps.storage, &Uint128::from(300u128))?;
    MAX_DEBT_RATIO_PERCENT.save(deps.storage, &Uint128::from(3_500u128))?;

    BOND_SUPPLY_EXPANSION_PERCENT.save(deps.storage, &Uint128::from(500u128))?;

    PREMIUM_THRESHOLD.save(deps.storage, &Uint128::from(110u128))?;
    PREMIUM_PERCENT.save(deps.storage, &Uint128::from(7_000u128))?;
    

    // First 12 epochs with 5% expansion
    BOOTSTRAP_EPOCHS.save(deps.storage, &Uint128::from(12u128))?;
    BOOTSTRAP_SUPPLY_EXPANSION_PERCENT.save(deps.storage, &Uint128::from(500u128))?;

    // set seigniorageSaved to it's balance
    let seignorage_saved = query_token_balance(&deps.querier, tomb, env.contract.address)?;
    SEIGNIORAGE_SAVED.save(deps.storage, &seignorage_saved)?;

    INITIALIZED.save(deps.storage, &true);
    OPERATOR.save(deps.storage, &info.sender);

    Ok(Response::new())
}

pub fn try_setoperator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: Addr
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    OPERATOR.save(deps.storage, &operator)?;
    Ok(Response::new().add_attribute("action", "set operator"))
}

pub fn try_setmasonry(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    masonry: Addr
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    MASONRY.save(deps.storage, &masonry)?;
    Ok(Response::new().add_attribute("action", "set masonry"))
}

pub fn try_setbondtreasury(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bond_treasury: Addr
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    BOND_TREASURY.save(deps.storage, &bond_treasury)?;
    Ok(Response::new().add_attribute("action", "set bond treasury"))
}

pub fn try_settomboracle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tomb_oracle: Addr
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    TOMB_ORACLE.save(deps.storage, &tomb_oracle)?;
    Ok(Response::new().add_attribute("action", "set tomb oracle"))
}

pub fn try_settombpriceceiling(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tomb_price_ceiling: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    // [$1.0, $1.2]
    let tomb_price_one = TOMB_PRICE_ONE.load(deps.storage)?;
    if tomb_price_ceiling < tomb_price_one || 
        tomb_price_ceiling > tomb_price_one * Uint128::from(120u128) / Uint128::from(100u128) {
            return Err(ContractError::OutofRange {  });
    }
    TOMB_PRICE_CEILING.save(deps.storage, &tomb_price_ceiling)?;
    Ok(Response::new().add_attribute("action", "set tomb_price_ceiling"))
}

pub fn try_set_max_supply_expansion_percents(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    max_supply_expansion_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    // [$1.0, $1.2]
    if max_supply_expansion_percent < Uint128::from(10u128) || 
        max_supply_expansion_percent > Uint128::from(1000u128) {
            return Err(ContractError::OutofRange {  });
    }
    MAX_SUPPLY_EXPANSION_PERCENT.save(deps.storage, &max_supply_expansion_percent)?;
    Ok(Response::new().add_attribute("action", "set tomb_price_ceiling"))
}

pub fn try_set_supply_tiers_entry(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: Uint128, 
    value: Uint128
)
    -> Result<Response, ContractError>  
{
    check_onlyoperator(deps.storage, info.sender)?;
    if index < Uint128::zero() || index >= Uint128::from(9u128){
        return Err(ContractError::IndexOutOfRange{ });
    }
    
    let mut supply_tiers = SUPPLY_TIERS.load(deps.storage)?;
    if value <= supply_tiers[index.u128() as usize - 1] || 
        value >= supply_tiers[index.u128() as usize + 1] {
        return Err(ContractError::IndexOutOfRange {  })
    }
    supply_tiers[index.u128() as usize] = value;
    SUPPLY_TIERS.save(deps.storage, &supply_tiers)?;
    Ok(Response::new().add_attribute("action", "set supply tiers entry"))
}

pub fn try_set_max_expansion_tiers_entry(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: Uint128,
    value: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if index < Uint128::zero() || index >= Uint128::from(9u128){
        return Err(ContractError::IndexOutOfRange{ });
    }
    
    if value < Uint128::from(10u128) || value > Uint128::from(1000u128){
        return Err(ContractError::ValueOutOfRange {  })
    }
    let mut max_expansion_tiers = MAX_EXPANSION_TIERS.load(deps.storage)?;
    max_expansion_tiers[index.u128() as usize] = value;
    MAX_EXPANSION_TIERS.save(deps.storage,&max_expansion_tiers)?;

    Ok(Response::new().add_attribute("action", "set max expansion tiers entry"))
}

pub fn try_set_bond_depletion_floor_percent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bond_depletion_floor_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if bond_depletion_floor_percent < Uint128::from(500u128) 
        || bond_depletion_floor_percent > Uint128::from(10_000u128){
        return Err(ContractError::ValueOutOfRange {  });
    }
    BOND_DEPLETION_FLOOR_PERCENT.save(deps.storage, &bond_depletion_floor_percent)?;
    Ok(Response::new().add_attribute("action", "set bond deplection floor percent"))
}

pub fn try_set_max_supply_contraction_percent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    max_supply_contraction_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if max_supply_contraction_percent < Uint128::from(100u128) 
        || max_supply_contraction_percent > Uint128::from(1_500u128){
        return Err(ContractError::ValueOutOfRange {  });
    }
    MAX_SUPPLY_CONTRACTION_PERCENT.save(deps.storage, &max_supply_contraction_percent)?;
    Ok(Response::new().add_attribute("action", "set max supply contraction percent"))
}

pub fn try_set_max_debt_ratio_percent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    max_debt_ratio_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if max_debt_ratio_percent < Uint128::from(1_000u128) 
        || max_debt_ratio_percent > Uint128::from(10_000u128){
        return Err(ContractError::ValueOutOfRange {  });
    }
    MAX_DEBT_RATIO_PERCENT.save(deps.storage, &max_debt_ratio_percent)?;
    Ok(Response::new().add_attribute("action", "set max debt ratio percent"))
}

pub fn try_set_bootstrap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bootstrap_epochs: Uint128,
    bootstrap_supply_expansion_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if bootstrap_epochs > Uint128::from(120u128) {
        return Err(ContractError::ValueOutOfRange {  });
    }
    if bootstrap_supply_expansion_percent < Uint128::from(100u128) ||
        bootstrap_supply_expansion_percent > Uint128::from(1_000u128){
        return Err(ContractError::ValueOutOfRange {  });
    }
    BOOTSTRAP_EPOCHS.save(deps.storage, &bootstrap_epochs)?;
    BOOTSTRAP_SUPPLY_EXPANSION_PERCENT.save(deps.storage, &bootstrap_supply_expansion_percent)?;

    Ok(Response::new().add_attribute("action", "set bootstrap"))
}


pub fn try_set_extra_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dao_fund: Addr,
    dao_fund_shared_percent: Uint128,
    dev_fund: Addr,
    dev_fund_shared_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if dao_fund == Addr::unchecked("".to_string()) ||
        dev_fund == Addr::unchecked("".to_string()) {
        return Err(ContractError::ZeroAddress {  });
    }
    if dao_fund_shared_percent > Uint128::from(3_000u128) ||
        dev_fund_shared_percent > Uint128::from(1_000u128) {
        return Err(ContractError::ValueOutOfRange {  });
    }
    DAOFUND.save(deps.storage, &dao_fund)?;
    DAOFUND_SHARED_PERCENT.save(deps.storage, &dao_fund_shared_percent)?;
    DEVFUND.save(deps.storage, &dev_fund)?;
    DEVFUND_SHARED_PERCENT.save(deps.storage, &dev_fund_shared_percent)?;
    Ok(Response::new().add_attribute("action", "set extra funds"))
}

pub fn try_set_max_discount_rate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    max_discount_rate: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    MAX_DISCOUNT_RATE.save(deps.storage, &max_discount_rate)?;
    Ok(Response::new().add_attribute("action", "set max discount rate"))
}

pub fn try_set_max_premium_rate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    max_premium_rate: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    MAX_PREMIUM_RATE.save(deps.storage, &max_premium_rate)?;
    Ok(Response::new().add_attribute("action", "set max premium rate"))
}

pub fn try_set_discount_percent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    discount_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if discount_percent > Uint128::from(20_000u128){
        return Err(ContractError::ValueOutOfRange {  });
    }
    DISCOUNT_PERCENT.save(deps.storage, &discount_percent)?;
    Ok(Response::new().add_attribute("action", "set discount percent"))
}

pub fn try_set_premium_threshold(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    premium_threshold: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if premium_threshold < TOMB_PRICE_CEILING.load(deps.storage)? ||
        premium_threshold > Uint128::from(150u128)
    {
        return Err(ContractError::ValueOutOfRange {  });
    }
    PREMIUM_THRESHOLD.save(deps.storage, &premium_threshold)?;
    Ok(Response::new().add_attribute("action", "set premium threshold"))
}


pub fn try_set_premium_percent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    premium_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if premium_percent > Uint128::from(20_000u128){
        return Err(ContractError::ValueOutOfRange {  });
    }
    PREMIUM_PERCENT.save(deps.storage, &premium_percent)?;
    Ok(Response::new().add_attribute("action", "set premium percent"))
}

pub fn try_set_minting_factor_for_paying_debt(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minting_factor_for_paying_debt: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    if minting_factor_for_paying_debt < Uint128::from(10_000u128) ||
        minting_factor_for_paying_debt > Uint128::from(20_000u128){
        return Err(ContractError::ValueOutOfRange {  });
    }
    MINTING_FACTOR_FOR_PAYING_DEBT.save(deps.storage, &minting_factor_for_paying_debt)?;
    Ok(Response::new().add_attribute("action", "set mingting factor for paying debt"))
}

pub fn try_set_bond_supply_expansion_percent(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bond_supply_expansion_percent: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    BOND_SUPPLY_EXPANSION_PERCENT.save(deps.storage, &bond_supply_expansion_percent)?;
    Ok(Response::new().add_attribute("action", "set bond supply expansion percent"))
}

pub fn try_update_tomb_price(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {

    let msg_update = WasmMsg::Execute {
        contract_addr: TOMB_ORACLE.load(deps.storage)?.to_string(),
        msg: to_binary(
            &OracleMsg::Update {  }
        ).unwrap(),
        funds: vec![]
    };
    Ok(Response::new()
        .add_attribute("action", "Oracle Update")
        .add_message(msg_update)
    )
}

pub fn try_buy_bonds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tomb_amount: Uint128,
    target_price: Uint128
)
    -> Result<Response, ContractError>
{
    check_condition(deps.storage, env.clone())?;
    check_operator(deps.storage, deps.querier, env.clone())?;

    if tomb_amount <= Uint128::zero() {
        return Err(ContractError::ZeroValue {  });
    }

    let tomb_price = get_tomb_price(deps.storage, &deps.querier)?;
    if tomb_price == target_price {
        return Err(ContractError::TreasuryError { 
            msg: "tomb price moved".to_string()
        });
    }

    if tomb_price >= TOMB_PRICE_ONE.load(deps.storage)? {
        return Err(ContractError::TreasuryError { 
            msg: "tombPrice not eligible for bond purchase".to_string() 
        });
    }

    if tomb_amount > EPOCH_SUPPLY_CONTRACTION_LEFT.load(deps.storage)? {
        return Err(ContractError::TreasuryError { 
            msg: "not enough bond left to purchase".to_string() 
        });
    }

    let rate = get_bond_discount_rate(deps.storage, &deps.querier)?;
    if rate <= Uint128::zero() {
        return Err(ContractError::TreasuryError { msg: "invalid bond rate".to_string() });
    }

    let bond_amount = tomb_amount * rate / Uint128::from(ETHER);
    let tomb_supply = get_tomb_circulating_supply(deps.storage, &deps.querier)?;
    let tbond_total_supply = get_total_supply(&deps.querier, TBOND.load(deps.storage)?)?;
    let new_bond_supply = tbond_total_supply + bond_amount;
    let max_debt_ratio_percent = MAX_DEBT_RATIO_PERCENT.load(deps.storage)?;

    if new_bond_supply > tomb_supply * max_debt_ratio_percent / Uint128::from(10_000u128) {
        return Err(ContractError::TreasuryError {
            msg: "over max debt ratio".to_string() 
        });
    }

    let msg_burnfrom = WasmMsg::Execute { 
        contract_addr: TOMB.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::BurnFrom{
                    from: info.sender.to_string(),
                    amount: tomb_amount
                }
            )?,
        funds: vec![]
    };
    let msg_mint = WasmMsg::Execute { 
        contract_addr: TOMB.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::Mint { 
                recipient: info.sender.to_string(), 
                amount: bond_amount 
                }
            )?,
        funds: vec![]
    };

    let mut epoch_supply_contraction_left = EPOCH_SUPPLY_CONTRACTION_LEFT.load(deps.storage)?;
    epoch_supply_contraction_left -= tomb_amount;
    EPOCH_SUPPLY_CONTRACTION_LEFT.save(deps.storage, &epoch_supply_contraction_left)?;

    let mut _deps = deps;
    let _env = env.clone();
    let _info = info.clone();

    execute(_deps.branch(), _env, _info, ExecuteMsg::UpdateTombPrice {  })?;

    Ok(Response::new()
        .add_attribute("action", "buy bonds")
        .add_messages([msg_burnfrom, msg_mint])
    )
}

pub fn try_redeem_bonds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bond_amount: Uint128, 
    target_price: Uint128
)
    ->Result<Response, ContractError>
{
    check_condition(deps.storage, env.clone())?;
    check_operator(deps.storage, deps.querier, env.clone())?;

    if bond_amount <= Uint128::zero() {
        return Err(ContractError::TreasuryError { msg: "can not redeem bonds with zero amount".to_string()});
    }

    let tomb_price = get_tomb_price(deps.storage, &deps.querier)?;
    if tomb_price == target_price {
        return Err(ContractError::TreasuryError { 
            msg: "tomb price moved".to_string()
        });
    }

    if tomb_price <= TOMB_PRICE_ONE.load(deps.storage)? {
        return Err(ContractError::TreasuryError { 
            msg: "tombPrice not eligible for bond purchase".to_string() 
        });
    }

    let rate = get_bond_premium_rate(deps.storage, &deps.querier)?;
    if rate <= Uint128::zero() {
        return Err(ContractError::TreasuryError { msg: "invalid bond rate".to_string() });
    }

    let tomb_amount = bond_amount * rate / Uint128::from(ETHER);
    let tomb = TOMB.load(deps.storage)?;
    let tomb_balance = query_token_balance(&deps.querier, tomb, env.clone().contract.address)?;
    if tomb_balance < tomb_amount {
        return Err(ContractError::TreasuryError { 
            msg: "Treasury: treasury has no more budget".to_string()
        });
    }
    let mut seigniorage_saved = SEIGNIORAGE_SAVED.load(deps.storage)?;
    if seigniorage_saved > tomb_amount {
        seigniorage_saved -= tomb_amount
    }else {
        seigniorage_saved = Uint128::zero()
    }
    SEIGNIORAGE_SAVED.save(deps.storage, &seigniorage_saved)?;

    let msg_burnfrom = WasmMsg::Execute { 
        contract_addr: TBOND.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::BurnFrom{
                    from: info.sender.to_string(),
                    amount: bond_amount
                }
            )?,
        funds: vec![]
    };
    let msg_transfer = WasmMsg::Execute { 
        contract_addr: TOMB.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::Transfer {  
                recipient: info.sender.to_string(), 
                amount: tomb_amount 
                }
            )?,
        funds: vec![]
    };

    let mut _deps = deps;
    let _env = env.clone();
    let _info = info.clone();

    execute(_deps.branch(), _env, _info, ExecuteMsg::UpdateTombPrice {  })?;

    Ok(Response::new()
        .add_attribute("action", "redeem bonds")
        .add_messages([msg_burnfrom, msg_transfer])
    )
}

pub fn try_send_to_masonry(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
)
    ->Result<Response, ContractError>
{
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    let tomb = TOMB.load(deps.storage)?;

    let msg_mint = WasmMsg::Execute { 
        contract_addr: tomb.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::Transfer {  
                recipient: env.contract.address.to_string(), 
                amount: amount 
                }
            )?,
        funds: vec![]
    };
    msgs.push(CosmosMsg::Wasm(msg_mint));

    let mut daofund_shared_amount = Uint128::zero();
    let daofund_shared_percent = DAOFUND_SHARED_PERCENT.load(deps.storage)?;
    if daofund_shared_percent > Uint128::zero() {
        daofund_shared_amount = amount * daofund_shared_percent / Uint128::from(10_000u128);
        
        let msg_transfer = WasmMsg::Execute { 
            contract_addr: tomb.to_string(), 
            msg: to_binary(
                &BasisAssetMsg::Transfer { 
                    recipient: DAOFUND.load(deps.storage)?.to_string(), 
                    amount: daofund_shared_amount
                }
            )?, 
            funds: vec![]
        };
        msgs.push(CosmosMsg::Wasm(msg_transfer));
    }

    let mut devfund_shared_amount = Uint128::zero();
    let devfund_shared_percent = DEVFUND_SHARED_PERCENT.load(deps.storage)?;
    if devfund_shared_percent > Uint128::zero() {
        devfund_shared_amount = amount * devfund_shared_percent / Uint128::from(10_000u128);
        
        let msg_transfer = WasmMsg::Execute { 
            contract_addr: tomb.to_string(), 
            msg: to_binary(
                &BasisAssetMsg::Transfer { 
                    recipient: DEVFUND.load(deps.storage)?.to_string(), 
                    amount: devfund_shared_amount
                }
            )?, 
            funds: vec![]
        };
        msgs.push(CosmosMsg::Wasm(msg_transfer));
    }

    let _amount = amount - daofund_shared_amount - devfund_shared_amount;

    let msg_approve_0 = WasmMsg::Execute { 
        contract_addr: tomb.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::Approve { 
                spender: MASONRY.load(deps.storage)?.to_string(), 
                amount: Uint128::zero()
            }
        )?, 
        funds: vec![]
    };
    msgs.push(CosmosMsg::Wasm(msg_approve_0));

    let msg_approve_1 = WasmMsg::Execute { 
        contract_addr: tomb.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::Approve { 
                spender: MASONRY.load(deps.storage)?.to_string(), 
                amount: _amount
            }
        )?, 
        funds: vec![]
    };
    msgs.push(CosmosMsg::Wasm(msg_approve_1));

    let msg_allocate = WasmMsg::Execute { 
        contract_addr: MASONRY.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &MasonryMsg::AllocateSeigniorage {  
                amount: _amount
            }
        )?, 
        funds: vec![]
    };
    msgs.push(CosmosMsg::Wasm(msg_allocate));

    Ok(Response::new()
        .add_attribute("action", "send to masonry")
        .add_messages(msgs)
    )
}

pub fn try_send_to_bond_treasury(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
)
    -> Result<Response, ContractError>
{
    let bond_treasury = BOND_TREASURY.load(deps.storage)?;
    let treasury_balance = query_token_balance(
        &deps.querier, TOMB.load(deps.storage)?, bond_treasury.clone())?;

    let treasury_vested: Uint128 = deps.querier.query_wasm_smart(
        BOND_TREASURY.load(deps.storage)?, 
        &BondTreasuryQuery::TotalVested {  }
    )?;
    
    if treasury_vested >= treasury_balance {
        return Ok(Response::new());
    } else{
        let unspent = treasury_balance - treasury_vested;
        if amount > unspent {
            let msg = WasmMsg::Execute { 
                contract_addr: TOMB.load(deps.storage)?.to_string(), 
                msg: to_binary(
                    &BasisAssetMsg::Mint { 
                        recipient: bond_treasury.to_string(), 
                        amount: amount - unspent 
                    })?, 
                funds: vec![]
            };
            return Ok(Response::new()
                .add_attribute("action", "send to bond treasury")
                .add_message(msg)
            );
        }
        Ok(Response::new())
    }
}

pub fn calculate_max_supply_expansion_percent(
    storage: &mut dyn Storage,
    tomb_supply: Uint128
)
    ->StdResult<Uint128>
{
    let supply_tiers = SUPPLY_TIERS.load(storage)?;
    let max_expansion_tiers = MAX_EXPANSION_TIERS.load(storage)?;

    for tier_id in (0..=8).rev() {
        if tomb_supply >= supply_tiers[tier_id] {
            MAX_SUPPLY_EXPANSION_PERCENT.save(storage, 
                &max_expansion_tiers[tier_id])?;
            break;
        }
    }
    Ok(MAX_SUPPLY_EXPANSION_PERCENT.load(storage)?)
}

pub fn try_allocate_seigniorage(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
 )
    ->Result<Response, ContractError>
{
    check_condition(deps.storage, env.clone())?;
    check_epoch(deps.storage, env.clone())?;
    check_operator(deps.storage, deps.querier, env.clone())?;
 
    let mut deps = deps;
    execute(deps.branch(), env.clone(), info.clone(), 
        ExecuteMsg::UpdateTombPrice{})?;

    let previous_epoch_tomb_price = get_tomb_price(deps.storage, &deps.querier)?;
    PREVIOUS_EPOCH_TOMB_PRICE.save(deps.storage, &previous_epoch_tomb_price)?;

    let mut seigniorage_saved = SEIGNIORAGE_SAVED.load(deps.storage)?;
    let tomb_supply = get_tomb_circulating_supply(deps.storage, &deps.querier)? 
                                    - seigniorage_saved;

    let bond_supply_expansion_percent = BOND_SUPPLY_EXPANSION_PERCENT.load(deps.storage)?;
    
    execute(deps.branch(), env.clone(), info.clone(),
        ExecuteMsg::SendToBondTreasury {
            amount: tomb_supply * bond_supply_expansion_percent / Uint128::from(10_000u128)
        })?;

    if EPOCH.load(deps.storage)? < BOOTSTRAP_EPOCHS.load(deps.storage)? {
        // 28 first epochs with 4.5% expansion
        let bootstrap_supply_expansion_percent = BOOTSTRAP_SUPPLY_EXPANSION_PERCENT.load(deps.storage)?;
        execute(deps.branch(), env.clone(), info.clone(),
            ExecuteMsg::SendToMasonry { 
                amount: tomb_supply * bootstrap_supply_expansion_percent / Uint128::from(10_000u128)
            })?;
    } else {
        if previous_epoch_tomb_price > TOMB_PRICE_CEILING.load(deps.storage)? {
            // Expansion ($TOMB Price > 1 $FTM): there is some seigniorage to be allocated
            let bond_supply = get_total_supply(&deps.querier, TBOND.load(deps.storage)?)?;
            let mut percentage = previous_epoch_tomb_price - TOMB_PRICE_ONE.load(deps.storage)?;
            let mut saved_for_bond = Uint128::zero();
            let saved_for_masonry: Uint128;
            let mse = calculate_max_supply_expansion_percent(deps.storage, tomb_supply)? * Uint128::from((10u64).pow(14u32));

            if percentage > mse {
                percentage = mse;
            }
            if seigniorage_saved >= bond_supply * BOND_DEPLETION_FLOOR_PERCENT.load(deps.storage)? / Uint128::from(10_000u128) {
                // saved enough to pay debt, mint as usual rate
                saved_for_masonry = tomb_supply * percentage / Uint128::from(ETHER);
            } else {
                // have not saved enough to pay debt, mint more
                let seigniorage = tomb_supply * percentage / Uint128::from(ETHER);
                saved_for_masonry = seigniorage * SEIGNIORAGE_EXPANSION_FLOOR_PERCENT.load(deps.storage)? / Uint128::from(10_000u128);
                saved_for_bond = seigniorage - saved_for_masonry;
                if MINTING_FACTOR_FOR_PAYING_DEBT.load(deps.storage)?> Uint128::zero() {
                    saved_for_bond = saved_for_bond * MINTING_FACTOR_FOR_PAYING_DEBT.load(deps.storage)? / Uint128::from(10_000u128);
                }
            }
            if saved_for_masonry > Uint128::zero() {
                execute(deps.branch(), env.clone(), info.clone(),
                    ExecuteMsg::SendToMasonry { amount: saved_for_masonry })?;
            }
            if saved_for_bond > Uint128::zero() {
                seigniorage_saved += saved_for_bond;
                SEIGNIORAGE_SAVED.save(deps.storage, &seigniorage_saved)?;

                let msg  = WasmMsg::Execute { 
                    contract_addr: TOMB.load(deps.storage)?.to_string(), 
                    msg: to_binary(
                        &BasisAssetMsg::Mint { 
                            recipient: env.contract.address.to_string(), 
                            amount: saved_for_bond }
                    )?, 
                    funds: vec![]
                };
                return Ok(Response::new()
                .add_attribute("action", "allocate seignorage")
                .add_message(msg));
            }
        }
    }
    Ok(Response::new()
        .add_attribute("action", "allocate seignorage"))
}

pub fn try_governance_recover_unsupported(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Addr,
    amount: Uint128,
    to: Addr,
) 
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    // do not allow to drain core tokens
    let tomb = TOMB.load(deps.storage)?;
    let tbond = TBOND.load(deps.storage)?;
    let tshare = TSHARE.load(deps.storage)?;

    if token == tomb || token == tbond || token == tshare {
        return Err(ContractError::InvalidToken {  });
    }

    let msg = WasmMsg::Execute { 
        contract_addr: token.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::Transfer { recipient: to.to_string(), amount: amount }
        )?, 
        funds: vec![]
    };
    Ok(Response::new()
        .add_attribute("action", "goverance recover unsupported")
        .add_message(msg)
    )
}

pub fn try_masonry_set_operator(
    deps: DepsMut,
    info: MessageInfo,
    operator: Addr
)
    ->Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    let msg = WasmMsg::Execute { 
        contract_addr: MASONRY.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &MasonryMsg::SetOperator { operator: operator }
        )?, 
        funds: vec![]
    };
    Ok(Response::new()
        .add_attribute("action", "Masonry set operator")
        .add_message(msg)
    )
}

pub fn try_masonry_set_lockup(
    deps: DepsMut,
    info: MessageInfo,
    withdraw_lockup_epochs: Uint128,
    reward_lockup_epochs: Uint128
)
    ->Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    let msg = WasmMsg::Execute { 
        contract_addr: MASONRY.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &MasonryMsg::SetLockUp { withdraw_lockup_epochs, reward_lockup_epochs }
        )?, 
        funds: vec![]
    };
    Ok(Response::new()
        .add_attribute("action", "Masonry set operator")
        .add_message(msg)
    )
}

pub fn try_masonry_allocation_seigniorage(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
)
    ->Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    let msg = WasmMsg::Execute { 
        contract_addr: MASONRY.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &MasonryMsg::AllocateSeigniorage { amount }
        )?, 
        funds: vec![]
    };
    Ok(Response::new()
        .add_attribute("action", "Masonry allocation seigniorage")
        .add_message(msg)
    )
}

pub fn try_masonry_governance_recover_unsupported(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Addr,
    amount: Uint128,
    to: Addr
)
    ->Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    let msg = WasmMsg::Execute { 
        contract_addr: MASONRY.load(deps.storage)?.to_string(), 
        msg: to_binary(
            &MasonryMsg::GovernanceRecoverUnsupported { token, amount, to }
        )?, 
        funds: vec![]
    };
    Ok(Response::new()
        .add_attribute("action", "Masonry governance recover unsupported")
        .add_message(msg)
    )
}
