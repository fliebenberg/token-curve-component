use scrypto_test::prelude::*;

pub mod utils;

#[test]
fn setup_env_test() {
    let env = utils::setup_test_env();
    println!(
        "Test env owner account address: {:?}",
        env.owner_account.address
    );
}

#[test]
fn simple_buy_sell_tests() {
    let mut env = utils::setup_test_env();
    // println!("Token state before buy:");
    // utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let first_buy_receipt = utils::txs::token_buy(
        dec!(100),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    // println!("First buy receipt: {:?}", first_buy_receipt);
    // println!("Token state after buy:");
    // utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.004481404746557164"),
        "Incorrect price after buy"
    );
    assert!(
        token_state.current_supply == dec!("66943.295008216952188265"),
        "Incorrect supply after buy"
    );

    let first_sell_receipt = utils::txs::token_sell(
        dec!("66943.295008216952188265"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0"),
        "Incorrect current price after sell"
    );
    assert!(
        token_state.current_supply == dec!("0"),
        "Incorrect supply after sell"
    );

    let second_buy_receipt = utils::txs::token_buy_amount(
        dec!("66943.295008216952188265"),
        dec!("100"),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );

    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.004481404746557164"),
        "Incorrect current price after sell"
    );
    assert!(
        token_state.current_supply == dec!("66943.295008216952188265"),
        "Incorrect supply after sell"
    );

    println!("Before 2nd sell: ");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    println!("Token balance: {:?}", token_balance);

    let second_sell_receipt = utils::txs::token_sell_for_xrd_amount(
        dec!("50"),
        dec!("66943.295008216952188265"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    println!("After 2nd sell: ");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.002823108086643085"),
        "Incorrect current price after sell for xrd amount"
    );
    assert!(
        token_state.current_supply == dec!("53132.928459130553302386"),
        "Incorrect supply after sell for xrd amount"
    );

    let last_sell_receipt = utils::txs::token_sell(
        dec!("53132.928459130553302386"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    println!("After last sell: ");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0"),
        "Incorrect current price after sell for xrd amount"
    );
    assert!(
        token_state.current_supply == dec!("0"),
        "Incorrect supply after sell for xrd amount"
    );
}

// // use meme_token::test_bindings::*;

// #[test]
// fn test_hello() {
//     // Setup the environment
//     let mut test_runner = TestRunnerBuilder::new().build();

//     // Create an account
//     let (public_key, _private_key, account) = test_runner.new_allocated_account();

//     // Publish package
//     let package_address = test_runner.compile_and_publish(this_package!());

//     // Test the `instantiate_hello` function.
//     let manifest = ManifestBuilder::new()
//         .call_function(
//             package_address,
//             "Hello",
//             "instantiate_hello",
//             manifest_args!(),
//         )
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("{:?}\n", receipt);
//     let component = receipt.expect_commit(true).new_component_addresses()[0];

//     // Test the `free_token` method.
//     let manifest = ManifestBuilder::new()
//         .call_method(component, "free_token", manifest_args!())
//         .call_method(
//             account,
//             "deposit_batch",
//             manifest_args!(ManifestExpression::EntireWorktop),
//         )
//         .build();
//     let receipt = test_runner.execute_manifest_ignoring_fee(
//         manifest,
//         vec![NonFungibleGlobalId::from_public_key(&public_key)],
//     );
//     println!("{:?}\n", receipt);
//     receipt.expect_commit_success();
// }

// #[test]
// fn test_hello_with_test_environment() -> Result<(), RuntimeError> {
//     // Arrange
//     let mut env = TestEnvironment::new();
//     let package_address = Package::compile_and_publish(this_package!(), &mut env)?;

//     let mut hello = Hello::instantiate_hello(package_address, &mut env)?;

//     // Act
//     let bucket = hello.free_token(&mut env)?;

//     // Assert
//     let amount = bucket.amount(&mut env)?;
//     assert_eq!(amount, dec!("1"));

//     Ok(())
// }
