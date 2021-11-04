use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use web3::contract::{Contract, Options};
use web3::transports::WebSocket as T;
use web3::types::{H160, U256};

pub fn get_valid_timestamp(future_millis: u128) -> u128 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let time_millis = since_epoch.as_millis().checked_add(future_millis).unwrap();
    time_millis
}

/// Internal mint function for ERC20 token.
pub async fn mint(to: H160, amount: U256, to_contract: Contract<T>) -> Result<(), &'static str> {
    let balance: U256 = to_contract
        .query("balanceOf", to, None, Options::default(), None)
        .await
        .unwrap();
    let new_balance = match balance.checked_add(amount) {
        Some(x) => x,
        None => return Err("Overflow while minting new tokens."),
    };

    let supply: U256 = to_contract
        .query("totalSupply", (), None, Options::default(), None)
        .await
        .unwrap();

    let new_supply = match supply.checked_add(amount) {
        Some(x) => x,
        None => return Err("Overflow while minting new tokens."),
    };

    println!("New Total Supply: {}", new_supply);
    println!("New Token Balance: {}", new_balance);
    //Self::deposit_event(RawEvent::Transfer(None, Some(to), amount));
    Ok(())
}

/// Internal burn function for ERC20 token.
pub async fn burn(
    from: H160,
    amount: U256,
    from_contract: Contract<T>,
) -> Result<(), &'static str> {
    let balance: U256 = from_contract
        .query("balanceOf", from, None, Options::default(), None)
        .await
        .unwrap();
    let new_balance = match balance.checked_sub(amount) {
        Some(x) => x,
        None => return Err("Overflow while minting new tokens."),
    };

    let supply: U256 = from_contract
        .query("totalSupply", (), None, Options::default(), None)
        .await
        .unwrap();

    let new_supply = match supply.checked_sub(amount) {
        Some(x) => x,
        None => return Err("Overflow while minting new tokens."),
    };

    println!("New Total Supply: {}", new_supply);
    println!("New Token Balance: {}", new_balance);
    //Self::deposit_event(RawEvent::Transfer(None, Some(to), amount));
    Ok(())
}

pub fn integral(to_x: U256, exponent: usize, slope: usize) -> U256 {
    let nexp = match exponent.checked_add(1) {
        Some(x) => x,
        None => return 0.into(),
    };

    match (to_x.pow(nexp.into())).checked_mul(slope.into()) {
        Some(x) => return x,
        None => return 0.into(),
    }
}
