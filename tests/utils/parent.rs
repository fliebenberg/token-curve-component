use meme_token::radix_meme_main::test_bindings::RadixMemeMain;
use scrypto_test::prelude::*;

use super::*;

pub fn create_parent_component(
    owner_badge_address: &ResourceAddress,
    max_token_supply: Decimal,
    max_token_supply_to_trade: Decimal,
    max_xrd_market_cap: Decimal,
    fair_launch_period_mins: u32,
    tx_fee_perc: Decimal,
    listing_fee_perc: Decimal,
    creator_fee_perc: Decimal,
    token_creation_fee: Decimal,
    account: &AccInfo,
    test_runner: &mut TestRunnerType,
) -> (ComponentAddress, ComponentAddress) {
    let package_address = test_runner.compile_and_publish(this_package!());
    let new_component_manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "RadixMemeMain",
            "new",
            manifest_args![
                "Radix Meme Tokens Main Component",
                "The main component for the Radix Meme Token Creator",
                "https://radix.meme",
                max_token_supply,
                max_token_supply_to_trade,
                max_xrd_market_cap,
                fair_launch_period_mins,
                tx_fee_perc,
                listing_fee_perc,
                creator_fee_perc,
                token_creation_fee,
                owner_badge_address,
            ],
        )
        .try_deposit_entire_worktop_or_abort(account.address, None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        new_component_manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.pubkey)],
    );

    // println!("Create Token Curves Component Receipt: {:?}\n", receipt);
    if receipt.is_commit_failure() {
        panic!("Problem with creating TokenCurves component! {:?}", receipt);
    }
    let result = receipt.expect_commit_success();
    println!(
        "New TokenCurves components: {:?}",
        result.new_component_addresses()
    );
    let component_address = result.new_component_addresses()[0];
    // println!("TokenCurves component address: {:?}", component_address);
    let dapp_def = result.new_component_addresses()[1];
    // println!("TokenCurvese dapp definition address: {:?}", dapp_def);
    (component_address, dapp_def)
}

pub fn get_parent_state(
    parent_address: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> RadixMemeMain {
    let pool_state = get_component_state::<RadixMemeMain, NoExtension, InMemorySubstateDatabase>(
        parent_address.clone(),
        test_runner,
    );
    pool_state
}
