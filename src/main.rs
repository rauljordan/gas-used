use clap::{App,Arg};
use ::gas_used::{Transaction,query_transactions,compute_gas_used_by_addrs};

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
        Ok(txs) => {
            compute_gas_used_by_addrs(addresses, txs).unwrap();
        }
        Err(e) => println!("Error fetching transactions: {:?}", e),
    }
}