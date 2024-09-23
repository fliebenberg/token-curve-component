use scrypto_test::prelude::*;

use super::{AccInfo, TestRunnerType};

pub fn token_buy(
    xrd_to_send: Decimal,
    from_account: &AccInfo,
    token_curve_address: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> TransactionReceiptV1 {
    let token_buy_manifest = ManifestBuilder::new()
        .lock_fee(from_account.address.clone(), dec!("10"))
        .call_method(
            from_account.address.clone(),
            "withdraw",
            manifest_args![XRD, xrd_to_send.clone()],
        )
        .take_all_from_worktop(XRD, "tx_bucket")
        .call_method_with_name_lookup(token_curve_address.clone(), "buy", |lookup| {
            (lookup.bucket("tx_bucket"),)
        })
        .try_deposit_entire_worktop_or_abort(from_account.address, None)
        .build();
    let receipt = test_runner.execute_manifest(
        token_buy_manifest,
        vec![NonFungibleGlobalId::from_public_key(&from_account.pubkey)],
    );

    if receipt.is_commit_failure() {
        panic!("Problem with staking XRD to pool! {:?}", receipt);
    }
    // let result = receipt.expect_commit_success();
    receipt
}
