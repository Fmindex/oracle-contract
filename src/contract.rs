#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Uint128;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetPriceResponse, InstantiateMsg, QueryMsg};
use crate::state::{OwnerData, OWNER_INFO, PRICES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:oracle-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner_info = OwnerData {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OWNER_INFO.save(deps.storage, &owner_info)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetPrice { symbol, price } => try_set_price(deps, info, symbol, price),
    }
}

pub fn try_set_price(
    deps: DepsMut,
    info: MessageInfo,
    symbol: String,
    price: Uint128,
) -> Result<Response, ContractError> {
    // check authorization
    let owner_data = OWNER_INFO.load(deps.storage).unwrap();
    if info.sender != owner_data.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate zero (Assume price cannot be zero)
    if price == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // update price
    PRICES.update(
        deps.storage,
        symbol,
        |mut _prices| -> Result<_, ContractError> { Ok(price) },
    )?;
    Ok(Response::new().add_attribute("method", "set_price"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice { symbol } => to_binary(&query_get_price(deps, symbol)?),
    }
}

fn query_get_price(deps: Deps, symbol: String) -> StdResult<GetPriceResponse> {
    let price = PRICES.may_load(deps.storage, symbol);
    if price == Ok(None) {
        return Err(StdError::not_found("Symbol"));
    }
    Ok(GetPriceResponse {
        price: price.unwrap().unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn set_price() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SetPrice {
            symbol: "LUNA".to_string(),
            price: Uint128::new(1000000),
        };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can update the price
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::SetPrice {
            symbol: "LUNA".to_string(),
            price: Uint128::new(1000000),
        };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be "1000000"
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetPrice {
                symbol: "LUNA".to_string(),
            },
        )
        .unwrap();
        let value: GetPriceResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(1000000), value.price);

        // price cannot be 0
        let unauth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::SetPrice {
            symbol: "LUNA".to_string(),
            price: Uint128::new(0),
        };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::InvalidZeroAmount {}) => {}
            _ => panic!("Must return invalid zero amount error"),
        }
    }
}
