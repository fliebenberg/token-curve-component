use crate::radix_meme_main::radix_meme_main::RadixMemeMain;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct OwnerBadgeData {
    #[mutable]
    pub name: String,
}

#[derive(ScryptoSbor, NonFungibleData)]
struct FairLaunchReceiptData {
    xrd_amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RadixMemeTokenCreateEvent {
    token_address: ResourceAddress,
    component_address: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RadixMemeTokenTradeEvent {
    token_address: ResourceAddress,
    side: String,
    fair_launch_period: bool,
    token_amount: Decimal,
    xrd_amount: Decimal,
    end_price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RadixMemeClaimTokensEvent {
    tokens_claimed: Decimal,
    xrd_amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RadixMemeClaimFeeEvent {
    fee_claimed: Decimal,
}

#[blueprint]
#[events(
    RadixMemeTokenCreateEvent,
    RadixMemeTokenTradeEvent,
    RadixMemeClaimFeeEvent
)]
mod radix_meme_token_curve {

    enable_function_auth! {
        new => AccessRule::AllowAll;
    }

    // enable_method_auth! {
    //     roles {
    //         admin => updatable_by: [OWNER];
    //         owner => updatable_by: [OWNER];
    //     },
    //     methods {
    //         new_token_curve_simple => PUBLIC;
    //         change_default_parameters => restrict_to: [admin];
    //         claim_fee_amount => restrict_to: [owner];
    //         claim_all_fees => restrict_to:[owner];
    //         transfer_fees => PUBLIC;
    //     }
    // }

    enable_method_auth! {
        roles {
            creator => updatable_by: [OWNER];
            radix_meme_admin => updatable_by: [radix_meme_admin];
        },
        methods {
            buy => PUBLIC;
            buy_amount => PUBLIC;
            sell => PUBLIC;
            sell_for_xrd_amount => PUBLIC;
            claim_fair_launch_tokens => PUBLIC;
            claim_all_fees => restrict_to: [radix_meme_admin];
        }
    }
    struct RadixMemeTokenCurve {
        pub parent_address: ComponentAddress, // address of the parent component that this bonding curve component is part of
        pub address: ComponentAddress,        // the address of this bonding curve component
        pub owner_badge_address: ResourceAddress, // the address of the owner badge for this token and component
        pub dapp_def_address: GlobalAddress,      // the dapp def account address for this component
        pub token_manager: ResourceManager, // the resource manager for the token created as part of this component
        pub max_token_supply: Decimal, // the maximum supply of the token that will be available after it is listed on a dex
        pub max_token_supply_to_trade: Decimal, // the maximum supply of the token that can be traded on the bonding curve
        pub max_xrd_market_cap: Decimal, // the maximum market cap in XRD that will be reached when the max tokens have been traded on bonding curve
        pub max_xrd: Decimal, // the maximum XRD that will be received into this component
        pub tx_fee_perc: Decimal, // fee % taken on every tx, specified in decimals 1% = 0.01,
        pub listing_fee_perc: Decimal, // fee % taken when a token is listed on external dex, specified in decimals 1% = 0.01
        pub creator_fee_perc: Decimal, // fee % paid to the token creator when the token is listed on a dex, specified in decimals 1% = 0.01
        pub multiplier: PreciseDecimal, // the constant multiplier that is used in the bonding curve calcs. This is based on the max_supply and max_xrd values.
        pub xrd_vault: Vault,           // the vault that holds all the XRD recived by the component
        pub fee_vault: Vault,           // vault that holds all the fees earned by the component
        pub creator_fee_vault: Vault,   // vault that holds fees earned by the creator of the token
        pub last_price: Decimal,        // the price reached with the last trade on the component
        pub current_supply: Decimal, // the current supply of the token associated with this component
        pub fair_launch_period_mins: u32, // the number of mins allocated for a fair launch period
        pub in_fair_launch_period: bool, // indicates whether the token is still in its fair_launch_period
        pub fair_launch_receipt_manager: ResourceManager, // teh resource manager for fair launch receipts
        pub fair_launch_tokens: Vault, // vault containing tokens that are bought during fair launch period
        pub fair_launch_xrd: Decimal, // amount of xrd corresponding to tokens in fair launch tokens vault - used to determine tokens that cna be claimed
        pub time_created: i64, // the date the token curve was created in seconds since unix epoch - included for easy lookup
        pub target_reached: i64, // the date the token reached its target market cap in seconds since unix epoch
    }

    impl RadixMemeTokenCurve {
        // a function that creates a new bonding curve component
        // the function takes in several values that are used to launch the new token and set up the bonding curve component
        // the function returns a global instance of the component, a bucket with the owner badge for the new token and the address of the newly created component
        pub fn new(
            name: String,
            symbol: String,
            description: String,
            icon_url: String,
            telegram_url: String,
            x_url: String,
            website_url: String,
            max_token_supply: Decimal,
            max_token_supply_to_trade: Decimal,
            max_xrd_market_cap: Decimal,
            tx_fee_perc: Decimal,
            listing_fee_perc: Decimal,
            creator_fee_perc: Decimal,
            fair_launch_period_mins: u32,
            parent_address: ComponentAddress,
            parent_owner_rule: AccessRule,
        ) -> (
            Global<RadixMemeTokenCurve>,
            NonFungibleBucket,
            ComponentAddress,
        ) {
            assert!(tx_fee_perc < Decimal::ONE, "tx_fee_perc cannot be >= 1. tx_fee_perc is specified in decimals, e.g. 1% = 0.01. ");
            assert!(listing_fee_perc < Decimal::ONE, "listing_fee_perc cannot be >= 1. listing_fee_perc is specified in decimals, e.g. 1% = 0.01. ");
            let _parent_instance = Global::<RadixMemeMain>::from(parent_address.clone()); // checks that the function was called from a TokenCurves component
                                                                                          // let require_parent = rule!(require(global_caller(parent_address.clone())));
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<RadixMemeTokenCurve>::blueprint_id());
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
                    "name" => format!("{} owner badge.", symbol.clone()), updatable;
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

            let mut social_urls_vec: Vec<Url> = vec![];
            if telegram_url.len() > 0 {
                social_urls_vec.push(Url::of(telegram_url.clone()));
            }
            if x_url.len() > 0 {
                social_urls_vec.push(Url::of(x_url));
            }
            if website_url.len() > 0 {
                social_urls_vec.push(Url::of(website_url));
            }

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
                    "description" => format!("{} Token created on Radix.meme.", description), updatable;
                    "icon_url" => Url::of(icon_url.clone()), updatable;
                    "social_urls" => social_urls_vec.clone(), updatable;
                    "tags" => "RadixMemeToken", updatable;
                    "radix_meme_component" => format!("{:?}", component_address.clone()), updatable;
                }
            ))
            .create_with_no_initial_supply();
            let token_address = token_manager.address();

            let fair_launch_receipt_manager = ResourceBuilder::new_ruid_non_fungible::<FairLaunchReceiptData>(OwnerRole::Fixed(require_component_rule.clone()))
            .mint_roles(mint_roles! {
                minter => require_component_rule.clone();
                minter_updater => AccessRule::DenyAll;
            })
            .burn_roles(burn_roles! {
                burner => require_component_rule.clone();
                burner_updater => AccessRule::DenyAll;
            })
            .metadata(metadata!(
                init {
                    "name" => format!("{} Fair Launch Receipt", symbol.clone()), updatable;
                    "symbol" => format!("{}-FAIR", symbol.clone()), locked;
                    "description" => format!("Radix.meme Fair launch receipt for token {}. This receipt can be redeemed for tokens after the fair launch period has expired.", symbol.clone()), locked;
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

            let multiplier = RadixMemeTokenCurve::calculate_multiplier(
                max_xrd_market_cap.clone(),
                max_token_supply_to_trade.clone(),
            );
            let max_xrd = RadixMemeTokenCurve::calculate_max_xrd(
                multiplier.clone(),
                max_token_supply_to_trade.clone(),
            );

            let new_token_curve = RadixMemeTokenCurve {
                parent_address,
                address: component_address.clone(),
                owner_badge_address: owner_badge.resource_address(),
                dapp_def_address,
                token_manager,
                current_supply: Decimal::ZERO,
                max_token_supply,
                max_token_supply_to_trade,
                max_xrd_market_cap,
                max_xrd,
                tx_fee_perc,
                listing_fee_perc,
                creator_fee_perc,
                multiplier,
                xrd_vault: Vault::new(XRD),
                fee_vault: Vault::new(XRD),
                creator_fee_vault: Vault::new(XRD),
                last_price: Decimal::ZERO,
                fair_launch_period_mins,
                in_fair_launch_period: if fair_launch_period_mins > 0 {true} else {false},
                fair_launch_receipt_manager,
                fair_launch_tokens: Vault::new(token_address.clone()),
                fair_launch_xrd: Decimal::ZERO,
                time_created: Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch,
                target_reached: 0,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                owner_badge.resource_address()
            ))))
            .with_address(address_reservation)
            .roles(roles! {
                creator => rule!(require(owner_badge.resource_address()));
                radix_meme_admin => parent_owner_rule.clone();
            })
            .metadata(metadata! {
                init {
                    "name" => format!("Radix.meme: {}", symbol.clone()), updatable;
                    "description" => format!("Radix Meme Token Bonding Curve component for token {} ({})", name.clone(), symbol.clone()), updatable;
                    "info_url" => Url::of(String::from("https://radix.meme")), updatable;
                    "social_urls" => social_urls_vec.clone(), updatable;
                    "tags" => vec!["RadixMeme"], updatable;
                    "dapp_definition" => dapp_def_address.clone(), updatable;
                }
            })
            .globalize();
            Runtime::emit_event(RadixMemeTokenCreateEvent {
                token_address: token_address.clone(),
                component_address: component_address.clone(),
            });
            (new_token_curve, owner_badge, component_address)
        }

        // function to buy tokens from the bonding curve using the sent XRD
        // function takes a bucket with XRD to use to buy new tokens
        // function returns a bucket with the bought tokens as well as a bucket with any remaining XRD (if any)
        pub fn buy(&mut self, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                Runtime::get_tip_percentage() == 0,
                "Radix.meme does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only buy tokens with XRD"
            );
            self.check_in_fair_launch_period();
            let mut xrd_amount = in_bucket.amount();
            info!("XRD amount before fees: {}", xrd_amount);
            info!("Max xrd: {}", self.max_xrd);
            let available_xrd = self.max_xrd - self.xrd_vault.amount();
            let mut fee_amount = xrd_amount * self.tx_fee_perc;
            if xrd_amount > available_xrd {
                // calculate fee based on available xrd only
                fee_amount = available_xrd * self.tx_fee_perc;
                xrd_amount = xrd_amount - fee_amount;
                if xrd_amount >= available_xrd {
                    xrd_amount = available_xrd;
                    self.target_reached =
                        Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                }
            } else {
                xrd_amount = xrd_amount - fee_amount;
            };
            self.fee_vault.put(in_bucket.take(fee_amount));
            info!("XRD Amount after fees: {}", xrd_amount);
            let mut out_bucket = if self.in_fair_launch_period {
                // create bucket with fair launch receipt address
                Bucket::new(self.fair_launch_receipt_manager.address())
            } else {
                Bucket::new(self.token_manager.address())
            };
            if xrd_amount > Decimal::ZERO {
                let receive_tokens = RadixMemeTokenCurve::calculate_tokens_received(
                    xrd_amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
                if receive_tokens + self.current_supply > self.max_token_supply_to_trade {
                    panic!("Unexpected error! Not enough tokens remaining for tx.")
                }
                let new_tokens = self.token_manager.mint(receive_tokens.clone());
                let new_tokens_amount = new_tokens.amount();
                if self.in_fair_launch_period {
                    // in fair launch period buyer receives a receipt that can be used to claim tokens after the fair launch period.
                    self.fair_launch_tokens.put(new_tokens);
                    self.fair_launch_xrd = self.fair_launch_xrd + xrd_amount;
                    out_bucket.put(self.fair_launch_receipt_manager.mint_ruid_non_fungible(
                        FairLaunchReceiptData {
                            xrd_amount: xrd_amount.clone(),
                        },
                    ))
                } else {
                    out_bucket.put(new_tokens);
                }
                self.current_supply = self.current_supply + receive_tokens.clone();
                self.xrd_vault.put(in_bucket.take(xrd_amount));
                self.last_price =
                    RadixMemeTokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("buy"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
                    token_amount: new_tokens_amount.clone(),
                    xrd_amount: xrd_amount.clone(),
                    end_price: self.last_price.clone(),
                });
            }
            if self.target_reached > 0 {
                self.list_token();
            }
            (out_bucket, in_bucket)
        }

        // function to buy a specificly specified amount of tokens
        // the function takes in the specified value of tokens that must be bought as well as a bucket of XRD to pay for the tx
        // the function returns a bucket with the bought tokens as well as a bucket with any remaining XRD (if any)
        pub fn buy_amount(&mut self, amount: Decimal, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                Runtime::get_tip_percentage() == 0,
                "Radix.meme does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only buy tokens with XRD"
            );
            assert!(
                amount + self.current_supply <= self.max_token_supply_to_trade,
                "Cannot buy requested amount of tokens. Not enough supply left"
            );
            self.check_in_fair_launch_period();
            let mut out_bucket = if self.in_fair_launch_period {
                Bucket::new(self.fair_launch_receipt_manager.address())
            } else {
                Bucket::new(self.token_manager.address())
            };
            if amount > Decimal::ZERO {
                let xrd_required = RadixMemeTokenCurve::calculate_buy_price(
                    amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
                let fee_amount = xrd_required * self.tx_fee_perc;
                if xrd_required + fee_amount > in_bucket.amount() {
                    panic!("Not enough XRD sent for tx.");
                }
                if xrd_required + self.xrd_vault.amount() > self.max_xrd {
                    panic!("Unexpected error! Max XRD will be exceeded in tx.")
                }
                self.fee_vault.put(in_bucket.take(fee_amount));
                let new_tokens = self.token_manager.mint(amount.clone());
                let new_tokens_amount = new_tokens.amount();
                if self.in_fair_launch_period {
                    self.fair_launch_tokens.put(new_tokens);
                    self.fair_launch_xrd = self.fair_launch_xrd + xrd_required;
                    out_bucket.put(self.fair_launch_receipt_manager.mint_ruid_non_fungible(
                        FairLaunchReceiptData {
                            xrd_amount: xrd_required.clone(),
                        },
                    ))
                } else {
                    out_bucket.put(new_tokens);
                }
                self.current_supply = self.current_supply + amount;
                self.xrd_vault.put(in_bucket.take(xrd_required));
                self.last_price =
                    RadixMemeTokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                if self.xrd_vault.amount() >= self.max_xrd {
                    self.target_reached =
                        Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                    self.list_token();
                }
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("buy"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
                    token_amount: new_tokens_amount.clone(),
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
                Runtime::get_tip_percentage() == 0,
                "Radix.meme does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == self.token_manager.address(),
                "Wrong tokens sent in bucket"
            );
            self.check_in_fair_launch_period();
            if self.in_fair_launch_period {
                panic!("Cannot sell tokens during fair launch period.")
            }
            let token_amount = in_bucket.amount();
            if token_amount > self.current_supply {
                panic!("Unexpected error! Sending more tokens to sell than current supply.");
            }
            let mut out_bucket = Bucket::new(XRD);
            if token_amount > Decimal::ZERO {
                let mut receive_xrd = RadixMemeTokenCurve::calculate_sell_price(
                    token_amount.clone(),
                    self.current_supply.clone(),
                    self.multiplier.clone(),
                );
                if receive_xrd > self.xrd_vault.amount() {
                    panic!("Unexpected error! Not enough XRD in component for sell tx.")
                }
                let fee_amount = receive_xrd * self.tx_fee_perc;
                self.fee_vault.put(self.xrd_vault.take(fee_amount));
                receive_xrd = receive_xrd - fee_amount;
                let burn_bucket = in_bucket.take(token_amount);
                burn_bucket.burn();
                self.current_supply = self.current_supply - token_amount.clone();
                out_bucket.put(self.xrd_vault.take(receive_xrd.clone()));
                self.last_price =
                    RadixMemeTokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("sell"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
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
                Runtime::get_tip_percentage() == 0,
                "Radix.meme does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == self.token_manager.address(),
                "Wrong tokens sent in bucket"
            );
            self.check_in_fair_launch_period();
            if self.in_fair_launch_period {
                panic!("Cannot sell tokens during fair launch period.")
            }
            let fee_amount = amount * self.tx_fee_perc;
            if amount + fee_amount > self.xrd_vault.amount() {
                panic!("Not enough XRD in component vault for requested amount.");
            }
            self.fee_vault.put(self.xrd_vault.take(fee_amount));
            let mut out_bucket = Bucket::new(XRD);
            info!("In bucket amount: {:?}", in_bucket.amount());
            if amount > Decimal::ZERO {
                let tokens_to_sell = RadixMemeTokenCurve::calculate_tokens_to_sell(
                    amount.clone() + fee_amount.clone(),
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
                    RadixMemeTokenCurve::calculate_price(&self.current_supply, &self.multiplier);
                Runtime::emit_event(RadixMemeTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("sell"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
                    token_amount: tokens_to_sell.clone(),
                    xrd_amount: out_bucket.amount(),
                    end_price: self.last_price.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        // function to claim tokens allocated during fair launch period
        pub fn claim_fair_launch_tokens(&mut self, receipts_bucket: Bucket) -> Bucket {
            let mut out_bucket = Bucket::new(self.token_manager.address());
            let mut total_xrd = Decimal::ZERO;
            self.check_in_fair_launch_period();
            assert!(!self.in_fair_launch_period, "Fair launch period not finished. Fair launch tokens can only be claimed once fair launch period has finished.");
            assert!(
                receipts_bucket.resource_address() == self.fair_launch_receipt_manager.address(),
                "Incorrect tokens submitted for claim."
            );
            for receipt in receipts_bucket
                .as_non_fungible()
                .non_fungibles::<FairLaunchReceiptData>()
            {
                let receipt_data = receipt.data();
                total_xrd = total_xrd + receipt_data.xrd_amount;
                let mut claim_tokens = self.fair_launch_tokens.amount() * receipt_data.xrd_amount
                    / self.fair_launch_xrd;
                if claim_tokens > self.fair_launch_tokens.amount() {
                    claim_tokens = self.fair_launch_tokens.amount();
                }
                out_bucket.put(self.fair_launch_tokens.take(claim_tokens));
                self.fair_launch_xrd = self.fair_launch_xrd - receipt_data.xrd_amount;
            }
            receipts_bucket.burn();
            Runtime::emit_event(RadixMemeClaimTokensEvent {
                tokens_claimed: out_bucket.amount(),
                xrd_amount: total_xrd.clone(),
            });
            out_bucket
        }

        pub fn claim_all_fees(&mut self) -> Bucket {
            Runtime::emit_event(RadixMemeClaimFeeEvent {
                fee_claimed: self.fee_vault.amount(),
            });
            self.fee_vault.take_all()
        }

        fn check_in_fair_launch_period(&mut self) {
            if self.in_fair_launch_period
                && Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch
                    > self.time_created
                        + (self
                            .fair_launch_period_mins
                            .to_i64()
                            .expect("Could not convert fair_launch_period_mins to i64")
                            * 60)
            {
                self.end_fair_launch_period();
            }
        }

        fn end_fair_launch_period(&mut self) {
            self.in_fair_launch_period = false;
        }

        // method to launch the token on DEX(s)
        fn list_token(&mut self) {
            info!("Token will be listed!");
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
                let first_value: PreciseDecimal = multiplier
                    .checked_div(3)
                    .expect("calculate_buy_price problem. Div 3");
                let second_value = (precise_supply + new_tokens.clone())
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. First Powi(3).");
                let third_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. Second Powi(3).");
                let fourth_value = second_value + third_value;
                let precise_price = first_value
                    .checked_mul(fourth_value)
                    .expect("calculate_buy_price problem. Final Multiply.");
                result = Decimal::try_from(
                    precise_price
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_buy_price problem. Cant round precise decimal."),
                )
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
                info!("Miltiplier: {}", multiplier);
                let precise_xrd_received = PreciseDecimal::from(xrd_received.clone());
                info!("XRD Received: {}", precise_xrd_received);
                let precise_supply = PreciseDecimal::from(supply.clone());
                info!("Supply: {}", precise_supply);
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
                info!("Result: {}", precise_result);
                result = Decimal::try_from(
                    precise_result
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_tokens_received problem. Cant round precise decimal."),
                )
                .expect(
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

                let first_value: PreciseDecimal = multiplier
                    .clone()
                    .checked_div(3)
                    .expect("calculate_buy_price problem. Div 3");
                let second_value = (precise_supply.clone())
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. First Powi(3).");
                let third_value = (precise_new_supply.clone())
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. Second Powi(3).");
                let fourth_value = second_value - third_value;

                let precise_price = first_value
                    .checked_mul(fourth_value)
                    .expect("calculate_sell_price problem. Multiplication problem.");
                result = Decimal::try_from(
                    precise_price
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_sell_price problem. Cant round precise decimal."),
                )
                .expect("calculate_sell_price problem. Cant convert precise decimal to decimal.");
            }
            // info!("Sell price: {:?}", result);
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
                info!("Precise XRD required: {:?}", precise_xrd_required);
                let precise_supply = PreciseDecimal::from(supply.clone());
                info!("Precise supply: {:?}", precise_supply);
                let mut first_value: PreciseDecimal = precise_xrd_required
                    .checked_mul(3)
                    .expect("calculate_tokens_to_sell problem. First mul");
                info!("First value: {}", first_value);
                first_value = first_value
                    .checked_div(multiplier.clone())
                    .expect("calculate_tokens_to_sell problem. First div");
                info!("First value: {}", first_value);
                let second_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_tokens_to_sell problem. First powi");
                info!("Second value: {:?}", second_value);
                let third_value = second_value - first_value;
                info!("Third value: {:?}", third_value);
                let fourth_value = third_value
                    .checked_nth_root(3)
                    .expect("calculate_tokens_to_sell problem. First root");
                info!("Fourth value: {:?}", fourth_value);
                let precise_result = precise_supply - fourth_value;
                info!("Precise Result: {:?}", precise_result);
                result = Decimal::try_from(
                    precise_result
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_tokens_to_sell problem. Cant round precise decimal."),
                )
                .expect(
                    "calculate_tokens_to_sell problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }

        fn calculate_multiplier(
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal,
        ) -> PreciseDecimal {
            let divisor = PreciseDecimal::from(max_token_supply_to_trade)
                .checked_powi(3)
                .expect("Problem in calculating multiplier. powi(3)");
            let multiplier = PreciseDecimal::from(max_xrd_market_cap)
                .checked_div(divisor)
                .expect("Problem in calculating multiplier. First div");
            multiplier
        }

        fn calculate_max_xrd(
            multiplier: PreciseDecimal,
            max_token_supply_to_trade: Decimal,
        ) -> Decimal {
            let first_value: PreciseDecimal = multiplier
                .checked_div(3)
                .expect("Problem in calculating max_xrd. First div");
            let precise_max_supply = PreciseDecimal::from(max_token_supply_to_trade);
            let second_value: PreciseDecimal = precise_max_supply
                .checked_powi(3)
                .expect("Problem in calculating max_xrd. First powi");
            let precise_max_xrd: PreciseDecimal = first_value
                .checked_mul(second_value)
                .expect("Problem calculating max_xrd. First mul");
            Decimal::try_from(precise_max_xrd)
                .expect("Problem calculating max_xrd. Could not convert precise_max_xrd to decimal")
        }
    }
}
