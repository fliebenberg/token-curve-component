use crate::radix_meme_token_curve::radix_meme_token_curve::{
    RadixMemeTokenCurve, RadixMemeTokenCurveFunctions,
};
use scrypto::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RadixMemeChangeDefaultEvent {
    field_name: String,
    old_value: String,
    new_value: String,
}
#[blueprint]
mod radix_meme_main {
    enable_function_auth! {
        new => AccessRule::AllowAll;
    }

    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER];
            owner => updatable_by: [OWNER];
        },
        methods {
            new_token_curve_simple => PUBLIC;
            change_default_parameters => restrict_to: [admin];
            claim_fee_amount => restrict_to: [owner];
            claim_all_fees => restrict_to:[owner];
            transfer_fees => PUBLIC;
        }
    }

    struct RadixMemeMain {
        pub address: ComponentAddress,
        pub owner_badge_manager: ResourceManager,
        pub max_token_supply: Decimal, // the maximum token supply after listing on external dex
        pub max_token_supply_to_trade: Decimal, // the maximum token supply available for trading on the bonding curve
        pub max_xrd_market_cap: Decimal, // the maximum market cap in XRD that will be reached when the max tokens have been traded on the bonding curve
        pub tokens: KeyValueStore<ComponentAddress, bool>, // a simple list of the tokens launched and whether they are still active
        pub tx_fee_perc: Decimal, // fee % taken on every tx, specified in decimals 1% = 0.01
        pub listing_fee_perc: Decimal, // fee % paid to redix.meme when a token is listed on a dex, specified in decimals 1% = 0.01
        pub creator_fee_perc: Decimal, // fee % paid to the token creator when a token is listed on a dex, specified in decimals 1% = 0.01
        pub fees_vault: Vault,         // vault to hold fees
    }

    impl RadixMemeMain {
        // function to create a new TokenCurves (parent) instance. This instance will be used to launch and keep track of the individual token curve components
        // takes in the resource address to be used as owner badge and other values needed to create the parent component.
        pub fn new(
            name: String,
            description: String,
            info_url: String,
            max_token_supply: Decimal,
            max_token_supply_to_trade: Decimal,
            max_xrd_market_cap: Decimal,
            tx_fee_perc: Decimal,
            listing_fee_perc: Decimal,
            creator_fee_perc: Decimal,
            owner_badge_address: ResourceAddress,
        ) -> Global<RadixMemeMain> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<RadixMemeMain>::blueprint_id());
            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None); // will reset owner role after dapp def metadata has been set
            dapp_def_account.set_metadata("account_type", String::from("dapp definition"));
            dapp_def_account.set_metadata("name", name.clone());
            dapp_def_account.set_metadata("description", description.clone());
            dapp_def_account.set_metadata("info_url", Url::of(info_url.clone()));
            dapp_def_account.set_metadata(
                "claimed_entities",
                vec![GlobalAddress::from(component_address.clone())],
            );
            dapp_def_account.set_owner_role(rule!(require(owner_badge_address)));
            let dapp_def_address = GlobalAddress::from(dapp_def_account.address());

            RadixMemeMain {
                address: component_address,
                owner_badge_manager: ResourceManager::from_address(owner_badge_address.clone()),
                max_token_supply,
                max_token_supply_to_trade,
                max_xrd_market_cap,
                tx_fee_perc,
                listing_fee_perc,
                creator_fee_perc,
                tokens: KeyValueStore::new(),
                fees_vault: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                owner_badge_address.clone()
            ))))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                "name" => name, updatable;
                "description" => description, updatable;
                "info_url" => Url::of(info_url), updatable;
                "tags" => vec!["Token", "Meme", "Launcher"], updatable;
                "dapp_definition" => dapp_def_address.clone(), updatable;
                }
            })
            .globalize()
        }

        // function to create an individual token bonding curve component
        // takes in values used to set up the new token and its bonding curve component
        // returns a global instance of the new component as well as an owner badge for the token.
        pub fn new_token_curve_simple(
            &mut self,
            name: String,
            symbol: String,
            description: String,
            icon_url: String,
            telegram: String,
            x: String,
            website: String,
        ) -> (Global<RadixMemeTokenCurve>, NonFungibleBucket) {
            let (new_instance, owner_badge, component_address) =
                Blueprint::<RadixMemeTokenCurve>::new(
                    name,
                    symbol,
                    description,
                    icon_url,
                    telegram,
                    x,
                    website,
                    self.max_token_supply.clone(),
                    self.max_token_supply_to_trade.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.tx_fee_perc.clone(),
                    self.listing_fee_perc.clone(),
                    self.creator_fee_perc.clone(),
                    self.address.clone(),
                );
            self.tokens.insert(component_address.clone(), true);
            (new_instance, owner_badge)
        }

        pub fn change_default_parameters(&mut self, param_values: Vec<(String, String)>) {
            for (param_name, param_value) in param_values {
                self.change_default_parameter(param_name, param_value);
            }
        }

        fn change_default_parameter(&mut self, param_name: String, param_value: String) {
            let old_value: String;
            let new_value = param_value.clone();
            match param_name.as_str() {
                "max_token_supply" => {
                    old_value = self.max_token_supply.to_string();
                    self.max_token_supply = Decimal::try_from(param_value).expect(
                        "Could not convert parameter value for max_token_supply to Decimal",
                    );
                }
                "max_token_supply_to_trade" => {
                    old_value = self.max_token_supply_to_trade.to_string();
                    self.max_token_supply_to_trade = Decimal::try_from(
                        param_value,
                    ).expect(
                        "Could not convert parameter value for max_token_supply_to_trade to Decimal",
                    );
                }
                "max_xrd_market_cap" => {
                    old_value = self.max_xrd_market_cap.to_string();
                    self.max_xrd_market_cap = Decimal::try_from(param_value).expect(
                        "Could not convert parameter value for max_xrd_market_cap to Decimal",
                    )
                }
                "tx_fee_perc" => {
                    old_value = self.tx_fee_perc.to_string();
                    self.tx_fee_perc = Decimal::try_from(param_value)
                        .expect("Could not convert parameter value for tx_fee_perc to Decimal")
                }
                "listing_fee_perc" => {
                    old_value = self.listing_fee_perc.to_string();
                    self.listing_fee_perc = Decimal::try_from(param_value)
                        .expect("Could not convert parameter value for listing_fee_perc to Decimal")
                }
                "creator_fee_perc" => {
                    old_value = self.creator_fee_perc.to_string();
                    self.creator_fee_perc = Decimal::try_from(param_value)
                        .expect("Could not convert parameter value for creator_fee_perc to Decimal")
                }
                _ => panic!("Could not match parameter name"),
            };
            Runtime::emit_event(RadixMemeChangeDefaultEvent {
                field_name: param_name.clone(),
                old_value,
                new_value,
            });
        }

        pub fn claim_fee_amount(&mut self, amount: Decimal) -> Bucket {
            assert!(
                amount <= self.fees_vault.amount(),
                "Not enough fees in vault."
            );
            let out_bucket = self.fees_vault.take(amount);
            out_bucket
        }

        pub fn claim_all_fees(&mut self) -> Bucket {
            self.fees_vault.take_all()
        }

        pub fn transfer_fees(&mut self, in_bucket: Bucket) {
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only transfer XRD fees to the RadixMemeMain component."
            );
            self.fees_vault.put(in_bucket);
        }
    }
}
