# oracle-contract

### Overview

A contract keeping price of the symbols. It exposes 2 methods

- GetPrice: allow others to query for symbol's price
- SetPrice: allow only off-chain bot to execute to set symbol's price

### Quick Start

1. Compile: `cargo wasm`
2. Build: `cargo run-script optimize`

### Potential improvement plans

1. Add method to add whitelisted address who can update the price (allow only the current owner to call this).
2. Add method to reset all the symbol's price.
3. Add method to update Admin.
4. Add logic to save the updated time and expose the updated time to the query.
