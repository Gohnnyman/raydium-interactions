// tests/test.rs

use client::{
    config::Config, create_mint, create_pool, create_token_account, decrease_liquidity,
    increase_liquidity, mint_to_token_account,
};
use std::path::PathBuf;

/// Helper function to load the test configuration file.
/// Assumes that "tests/config_test.toml" exists relative to the workspace root.
fn load_config() -> Config {
    let mut config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config_path.push("tests/config_test.toml");
    Config::from_file(config_path)
        .expect("Failed to load config file. Please ensure the file exists and is valid.")
}

/// Test mint creation for Raydium.
#[test]
fn test_mint_token() {
    let config = load_config();
    let mint = create_mint(&config).expect("Failed to create mint");
    println!("Created mint: {}", mint);

    // Assert that the mint is not an empty string.
    let mint_str = mint.to_string();
    assert!(!mint_str.is_empty(), "Mint should not be an empty string");
}

/// Test creating a token account for a given mint.
#[test]
fn test_create_token_account() {
    let config = load_config();
    let mint = create_mint(&config).expect("Failed to create mint");
    let token_account =
        create_token_account(&config, &mint).expect("Failed to create token account");
    println!("Created token account: {}", token_account);

    // Ensure the token account identifier is non-empty.
    let token_account_str = token_account.to_string();
    assert!(
        !token_account_str.is_empty(),
        "Token account should not be empty"
    );
}

/// Test minting tokens to a token account.
#[test]
fn test_mint_to_token_account() {
    let config = load_config();
    let mint = create_mint(&config).expect("Failed to create mint");
    let token_account =
        create_token_account(&config, &mint).expect("Failed to create token account");

    // Attempt to mint tokens into the account.
    mint_to_token_account(&config, &mint, &token_account, 1000)
        .expect("Failed to mint to token account");

    println!("Minted tokens to token account: {}", token_account);
}

/// Test creating a new pool.
#[test]
fn test_create_pool() {
    let config = load_config();

    let mint1 = create_mint(&config).expect("Failed to create mint");
    let token_account1 =
        create_token_account(&config, &mint1).expect("Failed to create token account");

    let mint2 = create_mint(&config).expect("Failed to create mint");
    let token_account2 =
        create_token_account(&config, &mint2).expect("Failed to create token account");

    let config_index = 0;
    let open_time = 0;
    let price = 10.0;

    // Attempt to mint tokens into the account.
    mint_to_token_account(&config, &mint1, &token_account1, 1000)
        .expect("Failed to mint to token account");

    // Attempt to mint tokens into the account.
    mint_to_token_account(&config, &mint2, &token_account2, 1000)
        .expect("Failed to mint to token account");

    let pool = create_pool(&config, config_index, price, mint1, mint2, open_time)
        .expect("Failed to create pool");

    println!("Created pool: {}", pool);
    let pool_str = pool.to_string();
    assert!(!pool_str.is_empty(), "Pool should not be empty");
}

/// Test increasing and decreasing liquidity in a pool.
#[test]
fn test_liquidity_operations() {
    // Creating pool
    let config = load_config();

    let mint1 = create_mint(&config).expect("Failed to create mint");
    let token_account1 =
        create_token_account(&config, &mint1).expect("Failed to create token account");

    let mint2 = create_mint(&config).expect("Failed to create mint");
    let token_account2 =
        create_token_account(&config, &mint2).expect("Failed to create token account");

    let config_index = 0;
    let open_time = 0;
    let price = 10.0;

    // Attempt to mint tokens into the account.
    mint_to_token_account(&config, &mint1, &token_account1, 100_000)
        .expect("Failed to mint to token account");

    // Attempt to mint tokens into the account.
    mint_to_token_account(&config, &mint2, &token_account2, 100_000)
        .expect("Failed to mint to token account");

    let pool = create_pool(&config, config_index, price, mint1, mint2, open_time)
        .expect("Failed to create pool");

    // Increasing liquidity

    let tick_lower_price = 1.0;
    let tick_upper_price = 100.0;
    let input_amount = 100;

    increase_liquidity(
        &config,
        tick_lower_price,
        tick_upper_price,
        true, // Example flag (could be direction or a boolean option)
        input_amount,
        pool,
        config.global.slippage,
    )
    .expect("Failed to increase liquidity");

    println!("Waiting for liquidity to be added to the pool...");
    std::thread::sleep(std::time::Duration::from_secs(30));

    increase_liquidity(
        &config,
        tick_lower_price,
        tick_upper_price,
        true, // Example flag (could be direction or a boolean option)
        input_amount,
        pool,
        config.global.slippage,
    )
    .expect("Failed to increase liquidity");

    // Decreasing liquidity

    let liquidity_to_decrease = Some(10);

    decrease_liquidity(
        &config,
        tick_lower_price,
        tick_upper_price,
        liquidity_to_decrease,
        pool,
        config.global.slippage,
    )
    .expect("Failed to decrease liquidity");

    // Decreasing all liquidity and closing the position

    println!("Waiting for liquidity to be reduced from the pool...");
    std::thread::sleep(std::time::Duration::from_secs(30));

    decrease_liquidity(
        &config,
        tick_lower_price,
        tick_upper_price,
        None,
        pool,
        config.global.slippage,
    )
    .expect("Failed to decrease liquidity");
}
