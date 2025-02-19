# Raydium Interactions CLI
A command-line interface (CLI) tool to manage Raydium operations on Solana. This program allows you to perform the following operations:

Raydium Commands:
* Mint a new token.
* Create a token account.
* Mint tokens to an account.
* Increase or decrease liquidity in a pool.
* Create a new pool.
* Run test routines.

Soland Commands:
* `todo!()`

# Known Issues
* Due to lack of time, the program does not handle errors gracefully. It will panic on any error.
* The program firstly was intended to be a CLI tool not only for Raydium but also for Solend. However, due to time constraints and Solend docs (un)availability, the Solend part was not implemented.

# Installation

1. Clone the Repository: `git clone https://github.com/Gohnnyman/raydium-interactions`
2. Install Rust and Cargo: Ensure you have Rust installed. Visit https://rust-lang.org/tools/install for installation instructions.
3. Build the Project: cargo build --release

# Configuration

The program uses a TOML configuration file. By default, it looks for a file named "config.toml" in the project root. You can specify an alternative configuration file using the "-c" or "--config" option.

The configuration file should have the following structure:

```toml
[global]
http_url = "https://api.devnet.solana.com" # Solana RPC URL
ws_url = "wss://api.devnet.solana.com/" # Solana Websocket URL
payer_path = "~/.config/solana/id.json" # Path to the Solana payer account
raydium_v3_program = "devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH" # Raydium V3 program ID on this Solana network
slippage = 0.01 # Slippage for token swaps
```

# Usage

After building the project, run the CLI using Cargo:
```
cargo run -p client
```
To view the help menu:
```
cargo run -p client -- --help
```

To see raydium commands:
```
cargo run -p client -- raydium --help
```
Output will be:
```
Raydium-related operations

Usage: client raydium <COMMAND>

Commands:
  mint-token             Mint a new token
  create-token-account   Create a token account for the specified mint
  mint-to-token-account  Mint tokens to an existing token account
  increase-liquidity     Increase liquidity in a pool by specifying the price range and input amount
  decrease-liquidity     Decrease liquidity from a pool by specifying the price range and liquidity
  create-pool            Create a new pool using the provided parameters
  help                   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

# Testing
Integration tests are available in the "tests/" folder.

Configure the config_test.toml file with the necessary parameters for the tests and run: 
```
cargo test
```
It will take some time, but it will run the tests.



