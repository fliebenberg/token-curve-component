use crate::token_curves::token_curves::TokenCurves;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct OwnerBadgeData {
    #[mutable]
    pub name: String,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RadixMemeTokenCreateEvent {
    token_address: ResourceAddress,
}
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RadixMemeTokenTradeEvent {
    token_address: ResourceAddress,
    side: String,
    token_amount: Decimal,
    xrd_amount: Decimal,
    end_price: Decimal,
}

#[blueprint]
#[events(RadixMemeTokenTradeEvent)]
mod token_curve {
    enable_function_auth! {
        new => AccessRule::AllowAll;
    }
    struct TokenCurve {
        pub parent_address: ComponentAddress, // address of the parent component that this bonding curve component is part of
        pub address: ComponentAddress,        // the address of this bonding curve component
        pub owner_badge_address: ResourceAddress, // the address of the owner badge for this token and component
        pub dapp_def_address: GlobalAddress,      // the dapp def account address for this component
        pub token_manager: ResourceManager, // the resource manager for the token created as part of this component
        pub max_supply: Decimal, // the maximum supply of the token that can be created/traded through this component
        pub max_xrd: Decimal,    // the maximum XRD that will be received into this component
        pub multiplier: PreciseDecimal, // the constant multiplier that is used in the bonding curve calcs. This is based on the max_supply and max_xrd values.
        pub xrd_vault: Vault,           // the vault that holds all the XRD recived by the component
        pub last_price: Decimal,        // the price reached with the last trade on the component
        pub current_supply: Decimal, // the current supply of the token associated with this component
    }

    impl TokenCurve {
        // a function that creates a new bonding curve component
        // the function takes in several values that are used to launch the new token and set up the bonding curve component
        // the function returns a global instance of the component, a bucket with the owner badge for the new token and the address of the newly created component
        pub fn new(
            name: String,
            symbol: String,
            description: String,
            icon_url: String,
            telegram: String,
            x: String,
            website: String,
            max_supply: Decimal,
            max_xrd: Decimal,
            multiplier: PreciseDecimal,
            parent_address: ComponentAddress,
        ) -> (Global<TokenCurve>, NonFungibleBucket, ComponentAddress) {
            let _parent_instance = Global::<TokenCurves>::from(parent_address.clone()); // checks that the function was called from a TokenCurves component
                                                                                        // let require_parent = rule!(require(global_caller(parent_address.clone())));
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<TokenCurve>::blueprint_id());
            let require_component_rule = rule!(require(global_caller(component_address.clone())));

            let owner_badge = ResourceBuilder::new_ruid_non_fungible(OwnerRole::Updatable(
                AccessRule::AllowAll,
            )) // this will be reset to any who owns the token after the token has been created
            .mint_roles(mint_roles! {
                minter => rule!(allow_all);
                minter_updater => rule!(allow_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(deny_all);
                burner_updater => rule!(deny_all);
            })
            .metadata(metadata!(
                init {
                    "name" => format!("{} owner badge.", name.clone()), updatable;
                    "symbol" => format!("{}", symbol.clone()), updatable;
                    "icon_url" => Url::of(icon_url.clone()), updatable;
                    "radix_meme_component" => format!("{:?}", component_address.clone()), locked;
                    "tags" => vec!["Dexter", "TokenCurve"], updatable;
                }
            ))
            .mint_initial_supply([OwnerBadgeData {
                name: "Owner Badge 1".to_owned(),
            }]);
            let owner_badge_manager = owner_badge.resource_manager();
            owner_badge_manager.set_mintable(rule!(require(owner_badge.resource_address()))); // any owner badge holder can mint more owner badges
            owner_badge_manager.lock_mintable();
            owner_badge_manager.set_owner_role(rule!(require(owner_badge.resource_address()))); // set owner role to be anyone with an owner badge

            let token_manager = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(
                require(owner_badge.resource_address())
            )))
            .divisibility(DIVISIBILITY_MAXIMUM)
            .mint_roles(mint_roles! {
                minter => require_component_rule.clone();
                minter_updater => require_component_rule.clone();
            })
            .burn_roles(burn_roles! {
                burner => require_component_rule.clone();
                burner_updater => require_component_rule.clone();
            })
            .metadata(metadata!(
                init {
                    "name" => name.clone(), updatable;
                    "symbol" => symbol.clone(), updatable;
                    "description" => description.clone(), updatable;
                    "icon_url" => Url::of(icon_url.clone()), updatable;
                    "telegram" => telegram.clone(), updatable;
                    "x" => x.clone(), updatable;
                    "website" => website.clone(), updatable;
                    "tags" => "RadixMemeToken", updatable;
                }
            ))
            .create_with_no_initial_supply();

            // each component creates its own dapp definition account with permission granted to the token owner to change the metadata in future
            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None); // will reset owner role after dapp def metadata has been set
            dapp_def_account.set_metadata("account_type", String::from("dapp definition"));
            dapp_def_account.set_metadata("name", format!("Radix Meme Token Curve: {}", symbol));
            dapp_def_account
                .set_metadata("description", format!("Radix Meme Token Curve: {}", name));
            dapp_def_account.set_metadata(
                "icon_url",
                Url::of("https://app.hydratestake.com/assets/hydrate_icon_light_blue.png"),
            );
            dapp_def_account.set_metadata(
                "claimed_entities",
                vec![GlobalAddress::from(component_address.clone())],
            );
            dapp_def_account.set_owner_role(rule!(require(owner_badge.resource_address())));
            let dapp_def_address = GlobalAddress::from(dapp_def_account.address());

            let new_token_curve = TokenCurve {
                parent_address,
                address: component_address.clone(),
                owner_badge_address: owner_badge.resource_address(),
                dapp_def_address,
                token_manager,
                current_supply: Decimal::ZERO,
                max_supply,
                max_xrd,
                multiplier,
                xrd_vault: Vault::new(XRD),
                last_price: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                owner_badge.resource_address()
            ))))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => format!("Binding Curve for {}", symbol.clone()), updatable;
                    "description" => format!("Radix Meme Token Bonding Curve component for token {} ({})", name.clone(), symbol.clone()), updatable;
                    "info_url" => Url::of(String::from("https://radix.meme")), updatable;
                    "tags" => vec!["Meme","Token", "Curve"], updatable;
                    "dapp_definition" => dapp_def_address.clone(), updatable;
                }
            })
            .globalize();
            (new_token_curve, owner_badge, component_address)
        }

        // function to buy tokens form the bonding curve using the sent XRD
        // function takes a bucket with XRD to use to buy new tokens
        // function returns a bucket with the bought tokens as well as a bucket with any remaining XRD (if any)
        pub fn buy(&mut self, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only buy tokens with XRD"
            );
            let mut xrd_amount = in_bucket.amount();
            if self.xrd_vault.amount() + xrd_amount > self.max_xrd {
                xrd_amount = self.max_xrd - self.xrd_vault.amount();
            }
            let mut out_bucket = Bucket::new(self.token_manager.address());
            if xrd_amount > Decimal::ZERO {
                let receive_tokens = TokenCurve::calculate_tokens_received(
                    xrd_amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
                if receive_tokens + self.current_supply > self.max_supply {
                    panic!("Unexpected error! Not enough tokens remaining for tx.")
                }
                out_bucket.put(self.token_manager.mint(receive_tokens.clone()));
                self.current_supply = self.current_supply + receive_tokens.clone();
                self.xrd_vault.put(in_bucket.take(xrd_amount));
                self.last_price =
                    TokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("buy"),
                    token_amount: out_bucket.amount(),
                    xrd_amount: xrd_amount.clone(),
                    end_price: self.last_price.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        // function to buy a specificly specified amount of tokens
        // the function takes in the specified value of tokens that must be bought as well as a bucket of XRD to pay for the tx
        // the function returns a bucket with the bought tokens as well as a bucket with any remaining XRD (if any)
        pub fn buy_amount(&mut self, amount: Decimal, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only buy tokens with XRD"
            );
            assert!(
                amount + self.current_supply <= self.max_supply,
                "Cannot buy requested amount of tokens. Not enough supply left"
            );
            let mut out_bucket = Bucket::new(self.token_manager.address());
            if amount > Decimal::ZERO {
                let xrd_required = TokenCurve::calculate_buy_price(
                    amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
                if xrd_required > in_bucket.amount() {
                    panic!("Not enough XRD sent for tx.");
                }
                if xrd_required + self.xrd_vault.amount() > self.max_xrd {
                    panic!("Unexpected error! Max XRD will be exceeded in tx.")
                }
                out_bucket.put(self.token_manager.mint(amount.clone()));
                self.current_supply = self.current_supply + amount;
                self.xrd_vault.put(in_bucket.take(xrd_required));
                self.last_price =
                    TokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("buy"),
                    token_amount: out_bucket.amount(),
                    xrd_amount: xrd_required.clone(),
                    end_price: self.last_price.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        // function to sell the tokens provided
        // function takes in a bucket of tokens to sell
        // function returns a bucket of XRD from the sale as well as a bucket with any remaining tokens (if any)
        pub fn sell(&mut self, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                in_bucket.resource_address() == self.token_manager.address(),
                "Wrong tokens sent in bucket"
            );
            let token_amount = in_bucket.amount();
            if token_amount > self.current_supply {
                panic!("Unexpected error! Sending more tokens to sell than current supply.");
            }
            let mut out_bucket = Bucket::new(XRD);
            if token_amount > Decimal::ZERO {
                let receive_xrd = TokenCurve::calculate_sell_price(
                    token_amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
                if receive_xrd > self.xrd_vault.amount() {
                    panic!("Unexpected error! Not enough XRD in component for sell tx.")
                }
                let burn_bucket = in_bucket.take(token_amount);
                burn_bucket.burn();
                self.current_supply = self.current_supply - token_amount.clone();
                out_bucket.put(self.xrd_vault.take(receive_xrd.clone()));
                self.last_price =
                    TokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("sell"),
                    token_amount: token_amount.clone(),
                    xrd_amount: out_bucket.amount(),
                    end_price: self.last_price.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        // function to sell tokens to the value of the specified XRD amount
        // the function takes in the amount of XRD to receive as well as a bucket of tokens to sell
        // the function returns a bucket with XRD and a bucket with any remaining tokens (if any)
        pub fn sell_for_xrd_amount(
            &mut self,
            amount: Decimal,
            mut in_bucket: Bucket,
        ) -> (Bucket, Bucket) {
            assert!(
                in_bucket.resource_address() == self.token_manager.address(),
                "Wrong tokens sent in bucket"
            );
            if amount > self.xrd_vault.amount() {
                panic!("Not enough XRD in vault for requested amount.");
            }
            let mut out_bucket = Bucket::new(XRD);
            info!("In bucket amount: {:?}", in_bucket.amount());
            if amount > Decimal::ZERO {
                let tokens_to_sell = TokenCurve::calculate_tokens_to_sell(
                    amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
                info!("Tokens required: {:?}", tokens_to_sell);
                if tokens_to_sell > in_bucket.amount() {
                    panic!("Not enough tokens supplied for required amount of XRD");
                }
                if tokens_to_sell > self.current_supply {
                    panic!("Unexpected error! Not enough token supply in component to sell.");
                }
                let burn_bucket = in_bucket.take(tokens_to_sell.clone());
                burn_bucket.burn();
                self.current_supply = self.current_supply - tokens_to_sell;
                out_bucket.put(self.xrd_vault.take(amount.clone()));
                self.last_price =
                    TokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("sell"),
                    token_amount: tokens_to_sell.clone(),
                    xrd_amount: out_bucket.amount(),
                    end_price: self.last_price.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        // the following calculation functions are all pure functions that (in future) can be moved to seperate components that represent different bonding curves.
        // This will allow for easier upgradability as well as easier addition of different types of bonding curves.

        // pure function to calculate the current price on the bonding curve based on the current token supply
        fn calculate_price(supply: &Decimal, multiplier: &PreciseDecimal) -> Decimal {
            Decimal::try_from(
                multiplier.clone()
                    * PreciseDecimal::from(supply.clone())
                        .checked_powi(2)
                        .expect("calculate_price problem. powi(2)"),
            )
            .expect("calculate_price problem. Cant convert precise decimal to decimal.")
        }

        // pure function to calculate the buy price (XRD required) in order to receive a specified amount of new tokens
        fn calculate_buy_price(
            new_tokens: Decimal,        // the amount of tokens to buy
            supply: Decimal,            // the supply of tokens before the buy transaction
            multiplier: PreciseDecimal, // the constant multiplier to use in the calcs (mased on max supply and max xrd)
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            if new_tokens > Decimal::ZERO {
                let precise_supply = PreciseDecimal::from(supply.clone());
                let precise_price: PreciseDecimal = multiplier
                    .clone()
                    .checked_div(3)
                    .expect("calculate_buy_price problem. Div 3");
                precise_price.checked_mul(
                    (precise_supply + new_tokens.clone())
                        .checked_powi(3)
                        .expect("calculate_buy_price problem. First Powi(3).")
                        + precise_supply
                            .checked_powi(3)
                            .expect("calculate_buy_price problem. Second Powi(3)."),
                );
                result = Decimal::try_from(precise_price)
                    .expect("calculate_buy_price problem. Cant convert precise decimal to decimal.")
            }
            result
        }

        // pure function to calculate how many tokens can be bought with the specified amount of XRD
        fn calculate_tokens_received(
            xrd_received: Decimal,      // the amount of XRD to spend to buy tokens
            supply: Decimal,            // the supply of tokens before the buy transaction
            multiplier: PreciseDecimal, // the constant multiplier to use in the calcs (mased on max supply and max xrd)
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            if xrd_received > Decimal::ZERO {
                let precise_xrd_received = PreciseDecimal::from(xrd_received.clone());
                let precise_supply = PreciseDecimal::from(supply.clone());
                let mut first_value = precise_xrd_received
                    .checked_div(multiplier.clone())
                    .expect("calculate_tokens_received problem. First div");
                first_value = first_value
                    .checked_mul(3)
                    .expect("calculate_tokens_received problem. First mul");
                info!("First value: {}", first_value);
                let second_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_tokens_received problem. First powi");
                info!("Second value: {}", second_value);
                let third_value = (first_value + second_value)
                    .checked_nth_root(3)
                    .expect("calculate_tokens_received problem. First root");
                info!("Third value: {}", third_value);
                let precise_result = third_value - precise_supply;
                result = Decimal::try_from(precise_result).expect(
                    "calculate_tokens_received problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }

        // function to calculate the sell price (XRD received) from selling the speficied number of tokens
        fn calculate_sell_price(
            sell_tokens: Decimal,       // the amount of tokens to sell
            supply: Decimal,            // the supply of tokens before the buy transaction
            multiplier: PreciseDecimal, // the constant multiplier to use in the calcs (mased on max supply and max xrd)
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            if sell_tokens > Decimal::ZERO {
                let precise_supply = PreciseDecimal::from(supply.clone());
                let precise_new_supply = precise_supply.clone() - sell_tokens.clone();
                let precise_price: PreciseDecimal = multiplier
                    .clone()
                    .checked_div(3)
                    .expect("calculate_buy_price problem. Div 3");
                precise_price.checked_mul(
                    (precise_supply.clone())
                        .checked_powi(3)
                        .expect("calculate_buy_price problem. First Powi(3).")
                        - (precise_new_supply.clone())
                            .checked_powi(3)
                            .expect("calculate_buy_price problem. Second Powi(3)."),
                );
                result = Decimal::try_from(precise_price).expect(
                    "calculate_buy_price problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }

        // function to calculate the amount of tokens to sell to receiv the specified amount of XRD
        fn calculate_tokens_to_sell(
            xrd_required: Decimal,      // the amount of XRD to receive from selling tokens
            supply: Decimal,            // the supply of tokens before the buy transaction
            multiplier: PreciseDecimal, // the constant multiplier to use in the calcs (mased on max supply and max xrd)
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            if xrd_required > Decimal::ZERO {
                let precise_xrd_required = PreciseDecimal::from(xrd_required.clone());
                // info!("Precise XRD required: {:?}", precise_xrd_required);
                let precise_supply = PreciseDecimal::from(supply.clone());
                // info!("Precise supply: {:?}", precise_supply);
                let mut first_value = precise_xrd_required
                    .checked_div(multiplier.clone())
                    .expect("calculate_tokens_to_sell problem. First div");
                first_value = first_value
                    .checked_mul(3)
                    .expect("calculate_tokens_to_sell problem. First mul");
                // info!("First value: {:?}", first_value);
                let second_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_tokens_to_sell problem. First powi");
                // info!("Second value: {:?}", second_value);
                let third_value = second_value - first_value;
                // info!("Third value: {:?}", third_value);
                let fourth_value = third_value
                    .checked_nth_root(3)
                    .expect("calculate_tokens_to_sell problem. First root");
                // info!("Fourth value: {:?}", fourth_value);
                let precise_result = precise_supply - fourth_value;
                // info!("Precise Result: {:?}", precise_result);
                result = Decimal::try_from(precise_result).expect(
                    "calculate_tokens_to_sell problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }
    }
}
