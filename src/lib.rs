use std::collections::HashMap;

use reqwest;
use reqwest::Client;
use serde::Deserialize;

use ethers::types::U256;
use ethers::utils::format_units;

const ENDPOINT: &str = "https://api.etherscan.io/api?module=account";

#[derive(Deserialize, Debug)]
struct Resp {
    result: Vec<Transaction>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Transaction {
    from: String,
    gasUsed: String,
    gasPrice: String,
}

// Retrieves all transactions from the Etherscan API until results
// are exhausted. TODO: Get around rate limits by adding sleep.
pub async fn query_transactions(api_key: &str, address: &str) -> Result<Vec<Transaction>, reqwest::Error> {
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

// Makes a request to the Etherscan API retrieving a list of account tx's
// for an address with an offset of 200.
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

// Computes the total gas used by specified addresses from
// a list of transactions.
pub fn compute_gas_used_by_addrs(
    addresses: Vec<&str>, txs: Vec<Transaction>,
) -> Result<HashMap<&str, U256>, &'static str> {
    let mut gas_used_by: HashMap<&str, U256> = HashMap::new();
    for addr in addresses.iter() {
        gas_used_by.insert(addr, U256::from(0));
    }
    for tx in txs.iter() {
        let gas_price = U256::from_dec_str(&tx.gasPrice).unwrap_or(U256::from(0));
        let gas_used = U256::from_dec_str(&tx.gasUsed).unwrap_or(U256::from(0));
        if let Some(total_eth_used) = gas_price.checked_mul(gas_used) {
            let from = tx.from.to_string();
            if let Some(addr_total) = gas_used_by.get_mut(&*from) {
                *addr_total = addr_total.checked_add(total_eth_used).unwrap();
            }
        }
    }
    for (addr, total_gas) in &gas_used_by {
        let gas_used_in_eth = format_units(
            total_gas, "ether",
        ).unwrap_or("0".to_string());
        println!("Address {:?} has used {} ETH for gas", addr, gas_used_in_eth);
    }
    Ok(gas_used_by)
}

#[cfg(test)]
mod test {
    use super::*;
    use ethers::utils::{
        WEI_IN_ETHER,format_ether,parse_units,parse_ether,
    };
    #[test]
    fn test_count_gas_used() {
        let price = parse_units(WEI_IN_ETHER, "wei").unwrap();
        let txs = vec![
            Transaction {
                from: String::from("foo"),
                gasUsed: U256::from(1).to_string(),
                gasPrice: price.to_string(),
            },
            Transaction {
                from: String::from("foo"),
                gasUsed: U256::from(1).to_string(),
                gasPrice: price.to_string(),
            },
            Transaction {
                from: String::from("bar"),
                gasUsed: U256::from(1).to_string(),
                gasPrice: price.to_string(),
            }
        ];
        let addresses = vec!["foo", "bar", "baz"];
        match compute_gas_used_by_addrs(addresses, txs) {
            Ok(gas_used_by) => {
                match gas_used_by.get("foo") {
                    Some(total) => {
                        let want_eth = format_ether(parse_ether("2").unwrap());
                        assert_eq!(format_ether(*total), want_eth);
                    },
                    None => panic!("foo not found"),
                }
                match gas_used_by.get("bar") {
                    Some(total) => {
                        let want_eth = format_ether(parse_ether("1").unwrap());
                        assert_eq!(format_ether(*total), want_eth);
                    },
                    None => panic!("bar not found"),
                }
                match gas_used_by.get("baz") {
                    Some(total) => {
                        assert_eq!(format_ether(*total), U256::from(0));
                    },
                    None => panic!("foo not found"),
                }
            },
            Err(_) => panic!("should not error")
        }
    }
}