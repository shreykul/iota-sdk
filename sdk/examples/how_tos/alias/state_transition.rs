// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will update the state metadata of an alias output.
//!
//! Make sure that `STRONGHOLD_SNAPSHOT_PATH` and `WALLET_DB_PATH` already exist by
//! running the `./how_tos/accounts_and_addresses/create_account.rs` example!
//!
//! Rename `.env.example` to `.env` first, then run the command:
//! ```sh
//! cargo run --release --all-features --example state_transition
//! ```

use iota_sdk::{types::block::output::AliasOutputBuilder, wallet::Result, Wallet};

// The metadata for the next state
const NEW_STATE_METADATA: &str = "updated state metadata 1";

#[tokio::main]
async fn main() -> Result<()> {
    //  This example uses secrets in environment variables for simplicity which should not be done in production.
    dotenvy::dotenv().ok();

    let wallet = Wallet::builder()
        .with_storage_path(&std::env::var("WALLET_DB_PATH").unwrap())
        .finish()
        .await?;
    let account = wallet.get_account("Alice").await?;

    // Set the stronghold password
    wallet
        .set_stronghold_password(std::env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    // May want to ensure the account is synced before sending a transaction.
    let balance = account.sync(None).await?;

    // Get the first alias
    let alias_id = balance
        .aliases()
        .first()
        .expect("No alias output available in the account.");

    let alias_output_data = account
        .unspent_alias_output(alias_id)
        .await?
        .expect("alias not found in unspent outputs");
    println!(
        "Alias '{alias_id}' found in unspent output: '{}'",
        alias_output_data.output_id
    );

    let token_supply = account.client().get_token_supply().await?;
    let rent_structure = account.client().get_rent_structure().await?;

    let alias_output = alias_output_data.output.as_alias();
    let updated_alias_output = AliasOutputBuilder::from(alias_output)
        // Minimum required storage deposit will change if the new metadata has a different size, so we will update
        // the amount
        .with_minimum_storage_deposit(rent_structure)
        .with_state_metadata(NEW_STATE_METADATA.as_bytes().to_vec())
        .with_state_index(alias_output.state_index() + 1)
        .finish_output(token_supply)?;

    println!("Sending transaction...",);
    let transaction = account.send_outputs(vec![updated_alias_output], None).await?;
    println!("Transaction sent: {}", transaction.transaction_id);

    let block_id = account
        .retry_transaction_until_included(&transaction.transaction_id, None, None)
        .await?;
    println!(
        "Block included: {}/block/{}",
        std::env::var("EXPLORER_URL").unwrap(),
        block_id
    );

    Ok(())
}
