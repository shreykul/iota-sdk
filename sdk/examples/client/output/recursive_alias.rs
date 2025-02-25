// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will create three alias outputs, where the first one can control the other two (recursively).
//!
//! Rename `.env.example` to `.env` first, then run the command:
//! ```sh
//! cargo run --release --example recursive_alias
//! ```

use iota_sdk::{
    client::{api::GetAddressesOptions, request_funds_from_faucet, secret::SecretManager, Client, Result},
    types::block::{
        address::{Address, AliasAddress},
        output::{
            feature::{IssuerFeature, SenderFeature},
            unlock_condition::{GovernorAddressUnlockCondition, StateControllerAddressUnlockCondition},
            AliasId, AliasOutputBuilder, Output, OutputId,
        },
        payload::Payload,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses secrets in environment variables for simplicity which should not be done in production.
    // Configure your own mnemonic in the ".env" file. Since the output amount cannot be zero, the seed must contain
    // non-zero balance.
    dotenvy::dotenv().ok();

    // Create a node client.
    let client = Client::builder()
        .with_node(&std::env::var("NODE_URL").unwrap())?
        .finish()
        .await?;

    let secret_manager = SecretManager::try_from_mnemonic(std::env::var("MNEMONIC").unwrap())?;

    let address = secret_manager
        .generate_ed25519_addresses(GetAddressesOptions::from_client(&client).await?.with_range(0..1))
        .await?[0];

    println!(
        "Requesting funds (waiting 15s): {}",
        request_funds_from_faucet(&std::env::var("FAUCET_URL").unwrap(), &address).await?,
    );
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let rent_structure = client.get_rent_structure().await?;
    let token_supply = client.get_token_supply().await?;

    //////////////////////////////////
    // create three new alias outputs
    //////////////////////////////////
    let alias_output_builder = AliasOutputBuilder::new_with_minimum_storage_deposit(rent_structure, AliasId::null())
        .add_feature(SenderFeature::new(address))
        .with_state_metadata([1, 2, 3])
        .add_immutable_feature(IssuerFeature::new(address))
        .add_unlock_condition(StateControllerAddressUnlockCondition::new(address))
        .add_unlock_condition(GovernorAddressUnlockCondition::new(address));

    let outputs = vec![alias_output_builder.clone().finish_output(token_supply)?; 3];

    let block_1 = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;

    println!(
        "Block with new alias outputs sent: {}/block/{}",
        std::env::var("EXPLORER_URL").unwrap(),
        block_1.id()
    );

    let _ = client.retry_until_included(&block_1.id(), None, None).await?;

    //////////////////////////////////
    // create second transaction with the actual AliasId (BLAKE2b-256 hash of the Output ID that created the alias) and
    // make both alias outputs controlled by the first one
    //////////////////////////////////
    let alias_output_ids = get_new_alias_output_ids(block_1.payload().unwrap())?;
    let alias_id_0 = AliasId::from(&alias_output_ids[0]);
    let alias_id_1 = AliasId::from(&alias_output_ids[1]);
    let alias_id_2 = AliasId::from(&alias_output_ids[2]);

    let alias_0_address = Address::Alias(AliasAddress::new(alias_id_0));
    let alias_1_address = Address::Alias(AliasAddress::new(alias_id_1));

    let outputs = [
        // make second alias output be controlled by the first one
        alias_output_builder
            .clone()
            .with_alias_id(alias_id_1)
            // add a sender feature with the first alias
            .replace_feature(SenderFeature::new(alias_0_address))
            .with_state_index(0)
            .replace_unlock_condition(StateControllerAddressUnlockCondition::new(alias_0_address))
            .replace_unlock_condition(GovernorAddressUnlockCondition::new(alias_0_address))
            .finish_output(token_supply)?,
        // make third alias output be controlled by the second one (indirectly also by the first one)
        alias_output_builder
            .clone()
            .with_alias_id(alias_id_2)
            .with_state_index(0)
            .replace_unlock_condition(StateControllerAddressUnlockCondition::new(alias_1_address))
            .replace_unlock_condition(GovernorAddressUnlockCondition::new(alias_1_address))
            .finish_output(token_supply)?,
    ];

    let block_2 = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;
    println!(
        "Block with alias id set and ownership assigned to the first alias sent: {}/block/{}",
        std::env::var("EXPLORER_URL").unwrap(),
        block_2.id()
    );
    let _ = client.retry_until_included(&block_2.id(), None, None).await?;

    //////////////////////////////////
    // create third transaction with the third alias output updated
    //////////////////////////////////
    let outputs = [alias_output_builder
        .clone()
        .with_alias_id(alias_id_2)
        .with_state_index(1)
        .with_state_metadata([3, 2, 1])
        .replace_unlock_condition(StateControllerAddressUnlockCondition::new(alias_1_address))
        .replace_unlock_condition(GovernorAddressUnlockCondition::new(alias_1_address))
        .finish_output(token_supply)?];

    let block_3 = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;
    println!(
        "Block with state metadata of the third alias updated sent: {}/block/{}",
        std::env::var("EXPLORER_URL").unwrap(),
        block_3.id()
    );

    let _ = client.retry_until_included(&block_3.id(), None, None).await?;

    //////////////////////////////////
    // create fourth transaction with the third alias output updated again
    //////////////////////////////////
    let outputs = [alias_output_builder
        .with_alias_id(alias_id_2)
        .with_state_index(2)
        .with_state_metadata([2, 1, 3])
        .replace_unlock_condition(StateControllerAddressUnlockCondition::new(alias_1_address))
        .replace_unlock_condition(GovernorAddressUnlockCondition::new(alias_1_address))
        .finish_output(token_supply)?];

    let block_3 = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;
    println!(
        "Another block with state metadata of the third alias updated sent: {}/block/{}",
        std::env::var("EXPLORER_URL").unwrap(),
        block_3.id()
    );

    let _ = client.retry_until_included(&block_3.id(), None, None).await?;
    Ok(())
}

// helper function to get the output ids for new created alias outputs (alias id is null)
fn get_new_alias_output_ids(payload: &Payload) -> Result<Vec<OutputId>> {
    let mut output_ids = Vec::new();
    match payload {
        Payload::Transaction(tx_payload) => {
            for (index, output) in tx_payload.essence().as_regular().outputs().iter().enumerate() {
                if let Output::Alias(alias_output) = output {
                    if alias_output.alias_id().is_null() {
                        output_ids.push(OutputId::new(tx_payload.id(), index.try_into().unwrap())?);
                    }
                }
            }
        }
        _ => panic!("No tx payload"),
    }
    Ok(output_ids)
}
