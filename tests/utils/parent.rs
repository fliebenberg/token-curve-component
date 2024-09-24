use meme_token::token_curves::test_bindings::TokenCurves;
use scrypto_test::prelude::*;

use super::*;

pub fn create_parent_component(
    owner_badge_address: &ResourceAddress,
    max_token_supply: Decimal,
    max_xrd: Decimal,
    account: &AccInfo,
    test_runner: &mut TestRunnerType,
) -> (ComponentAddress, ComponentAddress) {
    let package_address = test_runner.compile_and_publish(this_package!());
    let new_component_manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "TokenCurves",
            "new",
            manifest_args![
                "Radix Meme Tokens Component",
                "The main com-onent for the Radix Meme Token Creator",
                "https://radix.meme",
                max_token_supply,
                max_xrd,
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
) -> TokenCurves {
    let pool_state = get_component_state::<TokenCurves, NoExtension, InMemorySubstateDatabase>(
        parent_address.clone(),
        test_runner,
    );
    pool_state
}
