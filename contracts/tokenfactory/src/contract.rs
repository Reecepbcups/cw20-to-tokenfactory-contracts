#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::TokenFactoryError;
use crate::msg::{ExecuteMsg, GetDenomResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use osmo_bindings::{OsmosisMsg, OsmosisQuery, OsmosisQuerier };

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tokenfactory-demo";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<OsmosisQuery>,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, TokenFactoryError> {
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<OsmosisQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<OsmosisMsg>, TokenFactoryError> {
    match msg {
        ExecuteMsg::CreateDenom { subdenom } => create_denom(deps, subdenom),
        ExecuteMsg::ChangeAdmin {
            denom,
            new_admin_address,
        } => change_admin(deps, denom, new_admin_address),
        ExecuteMsg::MintTokens {
            denom,
            amount,
            mint_to_address,
        } => mint_tokens(deps, denom, amount, mint_to_address),
        ExecuteMsg::BurnTokens {
            denom,
            amount,
            mint_to_address,
        } => burn_tokens(deps, denom, amount, mint_to_address),
    }
}

pub fn create_denom(_deps: DepsMut<OsmosisQuery>, subdenom: String) -> Result<Response<OsmosisMsg>, TokenFactoryError> {
    if subdenom.eq("") {
        return Err(TokenFactoryError::InvalidSubdenom {
            subdenom
        });
    }

    let create_denom_msg = OsmosisMsg::CreateDenom{subdenom};

    let res = Response::new()
    .add_attribute("method", "burn_tokens")
    .add_message(<OsmosisMsg>::from(create_denom_msg));

    Ok(res)
}

pub fn change_admin(
    _deps: DepsMut<OsmosisQuery>,
    denom: String,
    new_admin_address: String,
) -> Result<Response<OsmosisMsg>, TokenFactoryError> {
    let change_admin_msg = OsmosisMsg::ChangeAdmin{denom, new_admin_address};

    let res = Response::new()
    .add_attribute("method", "burn_tokens")
    .add_message(<OsmosisMsg>::from(change_admin_msg));

    Ok(res)
}

pub fn mint_tokens(
    _deps: DepsMut<OsmosisQuery>,
    denom: String,
    amount: Uint128,
    mint_to_address: String,
) -> Result<Response<OsmosisMsg>, TokenFactoryError> {

    let mint_tokens_msg = OsmosisMsg::MintTokens{denom, amount, mint_to_address};

    let res = Response::new()
    .add_attribute("method", "burn_tokens")
    .add_message(<OsmosisMsg>::from(mint_tokens_msg));

    Ok(res)
}

pub fn burn_tokens(
    _deps: DepsMut<OsmosisQuery>,
    denom: String,
    amount: Uint128,
    burn_from_address: String,
) -> Result<Response<OsmosisMsg>, TokenFactoryError> {

    let burn_token_msg = OsmosisMsg::burn_contract_tokens(denom, amount, burn_from_address);

    let res = Response::new()
        .add_attribute("method", "burn_tokens")
        .add_message(<OsmosisMsg>::from(burn_token_msg));

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<OsmosisQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDenom {
            creator_address,
            subdenom,
        } => to_binary(&get_denom(deps, creator_address, subdenom)),
    }
}

fn get_denom(deps: Deps<OsmosisQuery>, creator_addr: String, subdenom: String) -> GetDenomResponse {
    let querier = OsmosisQuerier::new(&deps.querier);
    let response = querier.full_denom(creator_addr, subdenom).unwrap();

    let get_denom_response = GetDenomResponse{ denom: response.denom };
    get_denom_response
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR,};
    use cosmwasm_std::{
        coins, Coin, OwnedDeps, from_binary
    };
    use osmo_bindings::{ OsmosisQuery };
    use osmo_bindings_test::{ OsmosisApp };
    use std::marker::PhantomData;

    pub fn mock_dependencies(
        contract_balance: &[Coin],
    ) -> OwnedDeps<MockStorage, MockApi, OsmosisApp, OsmosisQuery> {
        let custom_querier = OsmosisApp::new();
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: custom_querier,
            custom_query_type: PhantomData,
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(1000, "uosmo"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn create_denom_get_denom() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(1000, "uosmo"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        const DENOM_NAME: &str = "mydenom";

        let subdenom: String = String::from(DENOM_NAME);

        let msg = ExecuteMsg::CreateDenom { subdenom };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        let get_denom_query = QueryMsg::GetDenom{ creator_address: String::from(MOCK_CONTRACT_ADDR), subdenom: String::from(DENOM_NAME)};


        let response = query(deps.as_ref(), mock_env(), get_denom_query).unwrap();

        let get_denom_response: GetDenomResponse = from_binary(&response).unwrap();

        assert_eq!(format!("factory/{}/{}", MOCK_CONTRACT_ADDR, DENOM_NAME), get_denom_response.denom);
    }

    #[test]
    fn create_denom_invalid_subdenom() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(1000, "uosmo"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        let subdenom: String = String::from("");

        let msg = ExecuteMsg::CreateDenom { subdenom };
        let info = mock_info("creator", &coins(2, "token"));
        let actual = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(TokenFactoryError::InvalidSubdenom{subdenom: String::from("")}, actual);
    }
}
