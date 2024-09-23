use crate::token_curves::token_curves::TokenCurves;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct OwnerBadgeData {
    #[mutable]
    pub name: String,
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

    struct TokenCurve {
        parent_address: ComponentAddress,
        owner_badge_address: ResourceAddress,
        dapp_def_address: GlobalAddress,
        token_manager: ResourceManager,
        current_supply: Decimal,
        max_supply: Decimal,
        max_xrd: Decimal,
        multiplier: PreciseDecimal,
        xrd_vault: Vault,
        last_price: Decimal,
    }

    impl TokenCurve {
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
        ) -> (Global<TokenCurve>, NonFungibleBucket) {
            let _parent_instance = Global::<TokenCurves>::from(parent_address.clone()); // checks that the function was called from a TokenCurves component
            let require_parent = rule!(require(global_caller(parent_address.clone())));
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<TokenCurve>::blueprint_id());
            let require_component_rule = rule!(require(global_caller(component_address.clone())));
            let owner_badge =
                ResourceBuilder::new_ruid_non_fungible(OwnerRole::Updatable(AccessRule::AllowAll)) // this will be reset to any who owns the token after the token has been created
                    .metadata(metadata!(
                        init {
                            "name" => format!("{} owner badge.", name.clone()), updatable;
                            "symbol" => format!("{}", symbol.clone()), updatable;
                            "icon_url" => Url::of(icon_url.clone()), updatable;
                            "tags" => vec!["Dexter", "TokenCurve"], updatable;
                        }
                    ))
                    .mint_initial_supply([OwnerBadgeData {
                        name: "Owner Badge 1".to_owned(),
                    }]);
            let owner_badge_manager = owner_badge.resource_manager();
            owner_badge_manager.set_owner_role(rule!(require(owner_badge.resource_address()))); // set owner role to be anyone with an owner badge
            owner_badge_manager.set_mintable(rule!(require(owner_badge.resource_address()))); // any owner badge holder can mint more owner badges
            owner_badge_manager.set_burnable(rule!(require(owner_badge.resource_address()))); // any owner badge holder cna burn owner badges

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

            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None); // will reset owner role after dapp def metadata has been set
            dapp_def_account.set_metadata("account_type", String::from("dapp definition"));
            dapp_def_account.set_metadata("name", format!("Token Curve: {}", symbol));
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
            // .roles(roles! {
            //     hydrate_admin => admin_rule.clone();
            // })
            // .metadata(metadata! {
            //     init {
            //         "name" => name.clone(), updatable;
            //         "description" => description.clone(), updatable;
            //         "info_url" => Url::of(String::from("https://hydratestake.com")), updatable;
            //         "tags" => vec!["Hydrate"], updatable;
            //         "dapp_definition" => dapp_def_address.clone(), updatable;
            //     }
            // })
            .globalize();
            (new_token_curve, owner_badge)
        }

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

        pub fn sell(&mut self, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                in_bucket.resource_address() == self.token_manager.address(),
                "Wrong tokens sent in bucket"
            );
            let mut token_amount = in_bucket.amount();
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
            if amount > Decimal::ZERO {
                let tokens_to_sell = TokenCurve::calculate_tokens_to_sell(
                    amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
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

        fn calculate_price(supply: &Decimal, multiplier: &PreciseDecimal) -> Decimal {
            Decimal::try_from(
                multiplier.clone()
                    * PreciseDecimal::from(supply.clone())
                        .checked_powi(2)
                        .expect("calculate_price problem. powi(2)"),
            )
            .expect("calculate_price problem. Cant convert precise decimal to decimal.")
        }

        fn calculate_buy_price(
            new_tokens: Decimal,
            supply: Decimal,
            multiplier: PreciseDecimal,
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

        fn calculate_tokens_received(
            xrd_received: Decimal,
            supply: Decimal,
            multiplier: PreciseDecimal,
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            if xrd_received > Decimal::ZERO {
                let precise_xrd_received = PreciseDecimal::from(xrd_received.clone());
                let precise_supply = PreciseDecimal::from(supply.clone());
                let first_value = precise_xrd_received
                    .checked_div(multiplier.clone())
                    .expect("calculate_tokens_received problem. First div");
                first_value
                    .checked_mul(3)
                    .expect("calculate_tokens_received problem. First mul");
                let second_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_tokens_received problem. First powi");
                let third_value = (first_value - second_value)
                    .checked_nth_root(3)
                    .expect("calculate_tokens_received problem. First root");
                let precise_result = third_value - precise_supply;
                result = Decimal::try_from(precise_result).expect(
                    "calculate_tokens_received problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }

        fn calculate_sell_price(
            sell_tokens: Decimal,
            supply: Decimal,
            multiplier: PreciseDecimal,
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

        fn calculate_tokens_to_sell(
            xrd_required: Decimal,
            supply: Decimal,
            multiplier: PreciseDecimal,
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            if xrd_required > Decimal::ZERO {
                let precise_xrd_required = PreciseDecimal::from(xrd_required.clone());
                let precise_supply = PreciseDecimal::from(supply.clone());
                let first_value = precise_xrd_required
                    .checked_div(multiplier.clone())
                    .expect("calculate_tokens_to_sell problem. First div");
                first_value
                    .checked_mul(3)
                    .expect("calculate_tokens_to_sell problem. First mul");
                let second_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_tokens_to_sell problem. First powi");
                let third_value = (second_value - first_value)
                    .checked_nth_root(3)
                    .expect("calculate_tokens_to_sell problem. First root");
                let precise_result = precise_supply - third_value;
                result = Decimal::try_from(precise_result).expect(
                    "calculate_tokens_to_sell problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }
    }
}
