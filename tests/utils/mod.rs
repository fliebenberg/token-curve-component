use radix_engine_interface::prelude::*;
use scrypto::this_package;
use scrypto_test::prelude::*;
use scrypto_unit::*;

pub type TestRunnerType = TestRunner<NoExtension, InMemorySubstateDatabase>;
pub struct AccInfo {
    pub address: ComponentAddress,
    pub pubkey: Secp256k1PublicKey,
}
pub struct TestEnv {
    pub test_runner: TestRunnerType,
    pub owner_account: AccInfo,
    // pub user_account: AccInfo,
    // pub airdrop_account: AccInfo,
    // pub owner_badge_address: ResourceAddress,
    // pub admin_badge_address: ResourceAddress,
    // pub hydrate_address: ComponentAddress,
    // pub pool_address: ComponentAddress,
}

pub fn setup_test_env() -> TestEnv {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let owner_account = create_new_account(&mut test_runner);
    TestEnv {
        test_runner,
        owner_account,
    }
}

pub fn create_new_account(test_runner: &mut TestRunnerType) -> AccInfo {
    let (pubkey, _, address) = test_runner.new_allocated_account();
    AccInfo { address, pubkey }
}
