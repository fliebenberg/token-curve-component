use radix_engine_interface::prelude::*;
use scrypto::this_package;
use scrypto_test::prelude::*;
use scrypto_unit::*;

pub mod parent;
pub mod token;
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
    pub token1_address: ResourceAddress,
    pub tx_fee_perc: Decimal,
    pub listing_fee_perc: Decimal,
    pub creator_fee_perc: Decimal,
    pub token_creation_fee: Decimal,
}

pub fn setup_test_env(fair_launch_period: u32, with_fees: bool) -> TestEnv {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let owner_account = create_new_account(&mut test_runner);
    let owner_badge_address =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_MAXIMUM, owner_account.address);
    let mut tx_fee_perc = dec!("0");
    let mut listing_fee_perc = dec!("0");
    let mut creator_fee_perc = dec!("0");
    let mut token_creation_fee = dec!("0");
    if with_fees {
        tx_fee_perc = dec!("0.01");
        listing_fee_perc = dec!("0.05");
        creator_fee_perc = dec!("0.05");
        token_creation_fee = dec!("100");
    }

    let (parent_component, parent_dapp_def) = parent::create_parent_component(
        &owner_badge_address,
        dec!("1000000"),
        dec!("1000000"),
        dec!("1000000"),
        fair_launch_period,
        tx_fee_perc.clone(),
        listing_fee_perc.clone(),
        creator_fee_perc.clone(),
        token_creation_fee.clone(),
        &owner_account,
        &mut test_runner,
    );
    let (token1_component, _token1_dapp_def, token1_address) = token::create_token_curve_component(
        String::from("First Token"),
        String::from("FIRST"),
        String::from("The first token on Radix Meme Tokens"),
        String::from("https://dexteronradix.com/dexter-logo-and-lettering.svg"),
        String::from(""),
        String::from(""),
        String::from("https://radix.meme"),
        token_creation_fee.clone(),
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
        token1_component,
        token1_address,
        tx_fee_perc,
        listing_fee_perc,
        creator_fee_perc,
        token_creation_fee,
    }
}

pub fn create_new_account(test_runner: &mut TestRunnerType) -> AccInfo {
    let (pubkey, _, address) = test_runner.new_allocated_account();
    AccInfo { address, pubkey }
}

pub fn load_account_with_xrd(account: &AccInfo, amount: Decimal, test_runner: &mut TestRunnerType) {
    const FREE_AMOUNT: Decimal = dec!("10000");
    let mut allocated_amount = Decimal::ZERO;
    while allocated_amount < amount {
        let txmanifest = ManifestBuilder::new()
            .get_free_xrd_from_faucet()
            .deposit_batch(account.address)
            .build();
        let _receipt = test_runner.execute_manifest_ignoring_fee(
            txmanifest,
            vec![NonFungibleGlobalId::from_public_key(&account.pubkey)],
        );
        allocated_amount += FREE_AMOUNT;
    }
}

pub fn get_component_state<T: ScryptoDecode, E: NativeVmExtension, D: TestDatabase>(
    component_address: ComponentAddress,
    test_runner: &mut TestRunner<E, D>,
) -> T {
    let node_id: &NodeId = component_address.as_node_id();
    let partition_number = MAIN_BASE_PARTITION;
    let substate_key: &SubstateKey = &ComponentField::State0.into();

    let substate = test_runner
        .substate_db()
        .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<T>>(
            node_id,
            partition_number,
            substate_key,
        );
    substate.unwrap().into_payload()
}
