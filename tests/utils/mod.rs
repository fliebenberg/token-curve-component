use radix_engine_interface::prelude::*;
use scrypto::this_package;
use scrypto_test::prelude::*;
use scrypto_unit::*;

pub mod txs;

pub type TestRunnerType = TestRunner<NoExtension, InMemorySubstateDatabase>;
pub struct AccInfo {
    pub address: ComponentAddress,
    pub pubkey: Secp256k1PublicKey,
}
pub struct TestEnv {
    pub test_runner: TestRunnerType,
    pub owner_account: AccInfo,
    pub owner_badge_address: ResourceAddress,
    pub parent_component_address: ComponentAddress,
    pub parent_dapp_def: ComponentAddress,
    pub token1_component: ComponentAddress,
}

pub fn setup_test_env() -> TestEnv {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let owner_account = create_new_account(&mut test_runner);
    let owner_badge_address =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_MAXIMUM, owner_account.address);
    let (parent_component, parent_dapp_def) = create_token_curves_component(
        &owner_badge_address,
        dec!("1000000"),
        dec!("1000000"),
        &owner_account,
        &mut test_runner,
    );
    let (token_component, token_dapp_def) = create_token_curve_component(
        String::from("First Token"),
        String::from("FIRST"),
        String::from("The first token on Radix Meme Tokens"),
        String::from("https://dexteronradix.com/dexter-logo-and-lettering.svg"),
        String::from("telegram"),
        String::from("x"),
        String::from("https://radix.meme"),
        &parent_component,
        &owner_account,
        &mut test_runner,
    );
    TestEnv {
        test_runner,
        owner_account,
        owner_badge_address,
        parent_component_address: parent_component,
        parent_dapp_def,
        token1_component: token_component,
    }
}

pub fn create_new_account(test_runner: &mut TestRunnerType) -> AccInfo {
    let (pubkey, _, address) = test_runner.new_allocated_account();
    AccInfo { address, pubkey }
}

pub fn create_token_curves_component(
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

pub fn create_token_curve_component(
    name: String,
    symbol: String,
    description: String,
    icon_url: String,
    telegram: String,
    x: String,
    website: String,
    component_address: &ComponentAddress,
    account: &AccInfo,
    test_runner: &mut TestRunnerType,
) -> (ComponentAddress, ComponentAddress) {
    let new_component_manifest = ManifestBuilder::new()
        .call_method(
            component_address.clone(),
            "new_token_curve_simple",
            manifest_args![name, symbol, description, icon_url, telegram, x, website,],
        )
        .try_deposit_entire_worktop_or_abort(account.address, None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        new_component_manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.pubkey)],
    );

    // println!("Create Token Curves Component Receipt: {:?}\n", receipt);
    if receipt.is_commit_failure() {
        panic!("Problem with creating Token component! {:?}", receipt);
    }
    let result = receipt.expect_commit_success();
    println!(
        "New Token components: {:?}",
        result.new_component_addresses()
    );
    let component_address = result.new_component_addresses()[0];
    // println!("TokenCurves component address: {:?}", component_address);
    let dapp_def = result.new_component_addresses()[1];
    // println!("TokenCurvese dapp definition address: {:?}", dapp_def);
    (component_address, dapp_def)
}
