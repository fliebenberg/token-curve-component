use crate::token_curve::token_curve::{TokenCurve, TokenCurveFunctions};
use scrypto::prelude::*;

#[blueprint]
mod token_curves {
    enable_function_auth! {
        new => AccessRule::AllowAll;
    }

    struct TokenCurves {
        pub address: ComponentAddress,
        pub owner_badge_manager: ResourceManager,
        pub max_token_supply: Decimal, // the maximum token supply available for trading on the bonding curve
        pub max_xrd: Decimal, // the maximum xrd that will be needed to get to the maximum token supply
        pub multiplier: PreciseDecimal, // a constant used in the bonding curve calcs that is determined by the max_token_supply and max_xrd values
        pub tokens: KeyValueStore<ComponentAddress, bool>, // a simple list of the tokens launched and whether they are still active
    }

    impl TokenCurves {
        // function to create a new TokenCurves (parent) instance. This instance will be used to launch and keep track of the individual token curve components
        // takes in the resource address to be used as owner badge and other values needed to create the parent component.
        pub fn new(
            name: String,
            description: String,
            info_url: String,
            max_token_supply: Decimal,
            max_xrd: Decimal,
            owner_badge_address: ResourceAddress,
        ) -> Global<TokenCurves> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<TokenCurves>::blueprint_id());
            let divisor = PreciseDecimal::from(max_token_supply)
                .checked_powi(3)
                .expect("Problem in calculating multiplier. powi(3)");
            let multiplier = PreciseDecimal::from(max_xrd)
                .checked_div(divisor)
                .expect("Problem in calculating multiplier. First div");

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

            TokenCurves {
                address: component_address,
                owner_badge_manager: ResourceManager::from_address(owner_badge_address.clone()),
                max_token_supply,
                max_xrd,
                multiplier,
                tokens: KeyValueStore::new(),
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
        ) -> (Global<TokenCurve>, NonFungibleBucket) {
            let (new_instance, owner_badge, component_address) = Blueprint::<TokenCurve>::new(
                name,
                symbol,
                description,
                icon_url,
                telegram,
                x,
                website,
                self.max_token_supply.clone(),
                self.max_xrd.clone(),
                self.multiplier.clone(),
                self.address.clone(),
            );
            self.tokens.insert(component_address.clone(), true);
            (new_instance, owner_badge)
        }
    }
}
