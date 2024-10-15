use scrypto_test::prelude::*;

pub mod utils;

#[test]
fn setup_env_test() {
    let env = utils::setup_test_env(0, false);
    println!(
        "Test env owner account address: {:?}",
        env.owner_account.address
    );
}

#[test]
fn simple_buy_sell_tests() {
    let mut env = utils::setup_test_env(0, false);
    // println!("Token state before buy:");
    // utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let _first_buy_receipt = utils::txs::token_buy(
        dec!(100),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    // println!("First buy receipt: {:?}", _first_buy_receipt);
    // println!("Token state after buy:");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.004481404746557164"),
        "Incorrect price after buy"
    );
    assert!(
        token_state.current_supply == dec!("66943.295008216952188266"),
        "Incorrect supply after buy. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("100"),
        "Incorrect XRD in component after first buy. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("66943.295008216952188266"),
        "Incorrect token Balance in account after first buy. {:?}",
        token_balance.clone()
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("9900"),
        "Incorrect XRD Balance in account after first buy. {:?}",
        xrd_balance.clone()
    );

    let _first_sell_receipt = utils::txs::token_sell(
        dec!("66943.295008216952188266"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    println!("Receipt after first_sell: {:?}", _first_sell_receipt);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0"),
        "Incorrect current price after sell. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("0"),
        "Incorrect supply after sell. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("0"),
        "Incorrect XRD in component after first sell. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("0"),
        "Incorrect token Balance in account after first sell. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("10000"),
        "Incorrect XRD Balance in account after first sell. {:?}",
        xrd_balance.clone()
    );

    let _second_buy_receipt = utils::txs::token_buy_amount(
        dec!("66943.295008216952188266"),
        dec!("100"),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.004481404746557164"),
        "Incorrect current price after 2nd buy. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("66943.295008216952188266"),
        "Incorrect supply after 2nd buy. {:?}",
        token_state.current_supply
    );
    // utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("100"),
        "Incorrect XRD in component after 2nd buy. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("66943.295008216952188266"),
        "Incorrect token Balance in account after 2nd buy. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("9900"),
        "Incorrect XRD Balance in account after 2nd buy. {:?}",
        xrd_balance.clone()
    );

    let _second_sell_receipt = utils::txs::token_sell_for_xrd_amount(
        dec!("50"),
        dec!("66943.295008216952188265"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.002823108086643085"),
        "Incorrect current price after sell for xrd amount"
    );
    assert!(
        token_state.current_supply == dec!("53132.928459130553302387"),
        "Incorrect supply after sell for xrd amount. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("50"),
        "Incorrect XRD in component after sell for xrd amount. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("53132.928459130553302387"),
        "Incorrect TOken Balance in wallet after sell for XRD amount. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("9950"),
        "Incorrect XRD Balance in account after sell for XRD amount. {:?}",
        xrd_balance.clone()
    );

    let _last_sell_receipt = utils::txs::token_sell(
        dec!("53132.928459130553302387"),
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
        "Incorrect current price after last sell"
    );
    assert!(
        token_state.current_supply == dec!("0"),
        "Incorrect supply after last sell"
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("0"),
        "Incorrect XRD in component after last sell. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("0"),
        "Incorrect Token Balance in wallet after last sell. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("10000"),
        "Incorrect XRD Balance in account after last sell. {:?}",
        xrd_balance.clone()
    );
}

#[test]
fn buy_sell_all_tokens_tests() {
    let mut env = utils::setup_test_env(0, false);
    utils::load_account_with_xrd(&env.owner_account, dec!("500000"), &mut env.test_runner);
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    println!("XRD Balance in account: {:?}", xrd_balance);

    let _first_buy_receipt = utils::txs::token_buy(
        dec!(500000),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("1"),
        "Incorrect price after buy"
    );
    assert!(
        token_state.current_supply == dec!("1000000"),
        "Incorrect supply after buy. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("333333.333333333333333333"),
        "Incorrect XRD in component after first buy. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("1000000"),
        "Incorrect token Balance in account after first buy. {:?}",
        token_balance.clone()
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("176666.666666666666666667"),
        "Incorrect XRD Balance in account after first buy. {:?}",
        xrd_balance.clone()
    );

    let _first_sell_receipt = utils::txs::token_sell_for_xrd_amount(
        dec!("333333.333333333333333333"),
        dec!(1000000),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    println!("Sell receipt: {:?}", _first_sell_receipt);
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.0000000000000001"), // result not zero because of limited accuracy of scrypto calcs
        "Incorrect price after sell. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("0.01"),
        "Incorrect supply after sell. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("0"),
        "Incorrect XRD in component after first sell. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("0.01"),
        "Incorrect token Balance in account after first sell. {:?}",
        token_balance.clone()
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("510000"),
        "Incorrect XRD Balance in account after first buy. {:?}",
        xrd_balance.clone()
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
