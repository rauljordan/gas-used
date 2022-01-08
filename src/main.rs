use std::collections::HashMap;

use reqwest;
use reqwest::Client;
use serde::Deserialize;
use clap::{App,Arg};
use ethers::utils::format_units;
use ethers::types::U256;

const ENDPOINT: &str = "https://api.etherscan.io/api?module=account";

#[derive(Deserialize, Debug)]
struct Resp {
    result: Vec<Transaction>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Transaction {
    from: String,
    gasUsed: String,
    gasPrice: String,
}

#[tokio::main]
pub async fn main() {
    let matches = App::new("gas-used")
                          .version("1.0")
                          .author("Raul Jordan <raul@prysmaticlabs.com>")
                          .about("Simple CLI tool to count the gas used by accounts interacting with a specific contract")
                          .arg(Arg::new("api-key")
                               .long("api-key")
                               .help("Etherscan API key")
                               .takes_value(true))
                          .arg(Arg::new("contract")
                               .long("contract")
                               .help("The contract address to retrieve transactions for")
                               .takes_value(true))
                          .arg(Arg::new("addresses")
                               .help("Account address(es) that interacted with the contract")
                               .multiple_occurrences(true)
                               .takes_value(true))
                          .get_matches();
    let addresses = matches.values_of("addresses").unwrap().collect::<Vec<_>>();
    let api_key = matches.value_of("api-key").unwrap();
    let contract = matches.value_of("contract").unwrap();
    println!("Computing gas used for addresses: {:?}", addresses);
    println!("Contract address: {}", contract);
    let res: Result<Vec<Transaction>, reqwest::Error> = query_transactions(
        &api_key,
        &contract,
    ).await; 
    match res {
        Ok(txs) => count_gas_used(addresses, txs).unwrap(),
        Err(e) => println!("Error fetching transactions: {:?}", e),
    }
}

async fn query_transactions(api_key: &str, address: &str) -> Result<Vec<Transaction>, reqwest::Error> {
    let mut txs = vec![];
    let client = Client::new();
    let mut page: u32 = 1;
    loop {
        match txs_by_page(&client, &api_key, &address, page).await {
            Ok(retrieved_txs) => {
                if retrieved_txs.len() == 0 {
                    return Ok(txs);
                }
                txs.extend(retrieved_txs);
                page += 1;
            },
            Err(e) => return Err(e),
        }
    }
}

async fn txs_by_page(client: &Client, api_key: &str, address: &str, page: u32) -> Result<Vec<Transaction>, reqwest::Error> {
    println!("Querying page {} of Etherscan API", &page);
    let http_resp = client.get(ENDPOINT)
        .query(&[
            ("action", "txlist"),
            ("address", address), 
            ("apikey", api_key), 
            ("sort", "desc"),
            ("page", &page.to_string()),
            ("offset", "200"),
        ])
        .send()
        .await?;
    let r: Resp = http_resp.json().await?;
    Ok(r.result)
}

fn count_gas_used(addresses: Vec<&str>, txs: Vec<Transaction>) -> Result<(), &'static str> {
    let mut gas_used_by_owners: HashMap<&str, U256> = HashMap::new();
    for addr in addresses.iter() {
        gas_used_by_owners.insert(addr, U256::from(0));
    }
    for tx in txs.iter() {
        let gas_price = U256::from_dec_str(&tx.gasPrice).unwrap_or(U256::from(0));
        let gas_used = U256::from_dec_str(&tx.gasUsed).unwrap_or(U256::from(0));
        if let Some(total_eth_used) = gas_price.checked_mul(gas_used) {
            let from = tx.from.to_string();
            if let Some(addr_total) = gas_used_by_owners.get_mut(&*from) {
                *addr_total = addr_total.checked_add(total_eth_used).unwrap();
            }
        }
    }
    for (addr, total_gas) in &gas_used_by_owners {
        let gas_used_in_eth = format_units(
            total_gas, "ether",
        ).unwrap_or("0".to_string());
        println!("Address {:?} has used {} ETH for gas", addr, gas_used_in_eth);
    }
    Ok(())
}
