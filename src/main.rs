use secp256k1::SecretKey;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use web3::contract::{tokens::Tokenize, Contract, Options};
use web3::transports::WebSocket as T;
use web3::types::{Address, Bytes, TransactionParameters, H160, U256};

#[tokio::main]
async fn main() -> web3::Result<()> {
    dotenv::dotenv().ok(); //to read environment variables
                           //Instance that will be used to establish the connection to the test Ethereum network

    let web3s = web3::Web3::new(T::new(&env::var("INFURA_RINKEBY").unwrap()).await?);
    //Add an account address for example the test metamask wallet
    let mut accounts = web3s.eth().accounts().await?;
    accounts.push(H160::from_str(&env::var("OWNER_ACCOUNT_ADD").unwrap()).unwrap());

    let wei_to_eth: U256 = U256::exp10(18); //to make the conversion from Wei to ETH
    let mut balance = HashMap::new();

    for account in &accounts {
        balance.insert(
            account.to_string(),
            web3s.eth().balance(*account, None).await?,
        );
        println!(
            "Account Buyer: {:?} With Eth balance: {}",
            account,
            balance[&account.to_string()]
                .checked_div(wei_to_eth)
                .unwrap()
        );
        println!(
            "Current Balance Buyer {:?}",
            balance.get(&accounts[0].to_string()).unwrap()
        );
    }
    //basic parameters of the implementation of an ERC20 token
    //Test token address with liquidity
    let token_addr = Address::from_str(&env::var("TOKEN_ADDR").unwrap()).unwrap();
    let token_contract =
        Contract::from_json(web3s.eth(), token_addr, include_bytes!("erc20_abi.json")).unwrap();
    market_buy(token_addr, accounts[0], 173.into(), token_contract, 0, 1)
        .await
        .unwrap();

    Ok(())
}

async fn market_buy(
    origin_addr: H160,
    from: H160,
    tokens: U256,
    contract: Contract<T>,
    exp: usize,
    slp: usize,
) -> web3::Result<()> {
    let supply: U256 = contract
        .query("totalSupply", (), None, Options::default(), None)
        .await
        .unwrap();

    println!("Total Supply: {}", supply);
    let new_supply = supply.checked_add(tokens).unwrap();
    let integral_before = web3_tokenERC20::integral(supply, exp, slp);
    let integral_after = web3_tokenERC20::integral(new_supply, exp, slp);

    println!("Cost: {} Tokens", integral_after - integral_before);

    let web3s = web3::Web3::new(T::new(&env::var("INFURA_RINKEBY").unwrap()).await?);

    //swap ETH for ERC20 base token using a router to later simulate the use of bonding curve
    let router02_addr = Address::from_str(&env::var("ROUTER02_ADDR").unwrap()).unwrap();
    //Router02 smart contract ABI is used in JSON format
    //to be able to create a contract object and call its functions.
    let router02_contract = Contract::from_json(
        web3s.eth(),
        router02_addr,
        include_bytes!("router02_abi.json"),
    )
    .unwrap();

    //ETH must be swapped for the WETH token first,
    //its address is obtained and then it will be swapped for DAI (Token Base ERC20)
    //using the Router02 function: swapExactETHForTokens
    let weth_addr: Address = router02_contract
        .query("WETH", (), None, Options::default(), None)
        .await
        .unwrap();

    let valid_timestamp = web3_tokenERC20::get_valid_timestamp(300000);
    let _wei = U256::from_dec_str("50000000000000000").unwrap();

    let out_gas_estimate = router02_contract
        .estimate_gas(
            "swapExactETHForTokens",
            (
                U256::from_dec_str(&tokens.to_string()).unwrap(),
                vec![weth_addr, origin_addr],
                from,
                U256::from_dec_str(&valid_timestamp.to_string()).unwrap(),
            ),
            from,
            Options {
                value: Some(_wei),
                gas: Some(500_000.into()),
                ..Default::default()
            },
        )
        .await
        .expect("Error");

    let gas_price = web3s.eth().gas_price().await.unwrap();

    let data = router02_contract
        .abi()
        .function("swapExactETHForTokens")
        .unwrap()
        .encode_input(
            &(
                U256::from_dec_str(&tokens.to_string()).unwrap(),
                vec![weth_addr, origin_addr],
                from,
                U256::from_dec_str(&valid_timestamp.to_string()).unwrap(),
            )
                .into_tokens(),
        )
        .unwrap();

    let nonce = web3s.eth().transaction_count(from, None).await.unwrap();

    let transact_obj = TransactionParameters {
        nonce: Some(nonce),
        to: Some(router02_addr),
        value: _wei,
        gas_price: Some(gas_price),
        gas: out_gas_estimate,
        data: Bytes(data),
        ..Default::default()
    };

    let private_key = SecretKey::from_str(&env::var("TEST_KEY").unwrap()).unwrap();
    let signed_transaction = web3s
        .accounts()
        .sign_transaction(transact_obj, &private_key)
        .await
        .unwrap();

    let result = web3s
        .eth()
        .send_raw_transaction(signed_transaction.raw_transaction)
        .await
        .unwrap();

    web3_tokenERC20::mint(origin_addr, tokens, contract)
        .await
        .unwrap();

    println!(" result: {:?}", result);
    Ok(())
}
