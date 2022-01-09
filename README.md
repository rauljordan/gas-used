<!-- omit in TOC -->
# Gas Used

> **Simple CLI tool to count the gas used by accounts interacting with a specific Ethereum contract**

[![Crates.io](https://img.shields.io/crates/v/clap?style=flat-square)](https://crates.io/crates/gas-used)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Rust](https://github.com/rauljordan/gas-used/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/rauljordan/gas-used/actions/workflows/rust.yml)

Licensed under [MIT](LICENSE-MIT).

1. [About](#about)
2. [Usage](#usage)

## About

This is a simple CLI that can compute the total amount of gas used by a list of account addresses interacting with a specific contract on Ethereum. It can, for example, be used to count the total gas used by owners of a multisig contract when interacting with the multisig.

## Installing

```
git clone https://github.com/rauljordan/gas-used && cd gas-used
cargo install --bin gas-used --path .
```

## Usage

```
gas-used \
  --api-key=<ETHERSCAN_API_KEY> \
  --contract=<CONTRACT_ADDRESS> \
  -- ADDRESS_1 ADDRESS_2
```

Sample output
```
Computing gas used for addresses: 0xf, 0x3
Contract address: 0x9b984d5a03980d8dc0a24506c968465424c81dbe
Querying page 1 of Etherscan API
Querying page 2 of Etherscan API
Querying page 3 of Etherscan API
Querying page 4 of Etherscan API
Address "0xf" has used 0.775665286702632882 ETH for gas
Address "0x3" has used 1.220864757568855979 ETH for gas
