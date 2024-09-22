use crate::token_curve::token_curve::{TokenCurve, TokenCurveFunctions};
use scrypto::prelude::*;

#[blueprint]
mod token_curves {

    struct TokenCurves {
        owner_badge_manager: ResourceManager,
        max_token_supply: Decimal,
        max_xrd: Decimal,
        multiplier: PreciseDecimal,
    }

    impl TokenCurves {
        pub fn new(owner_badge_address: ResourceAddress) -> Global<TokenCurves> {
            let (address_reservation, _component_address) =
                Runtime::allocate_component_address(<TokenCurves>::blueprint_id());
            let max_supply = dec!("1000000");
            let max_xrd = dec!("1000000");
            let divisor = PreciseDecimal::from(max_supply)
                .checked_powi(3)
                .expect("Problem in calculating multiplier. powi(3)");
            let multiplier = PreciseDecimal::from(max_xrd.clone())
                .checked_div(divisor)
                .expect("Problem in calculating multiplier. First div");
            TokenCurves {
                owner_badge_manager: ResourceManager::from_address(owner_badge_address.clone()),
                max_token_supply: max_supply.clone(),
                max_xrd: max_xrd.clone(),
                multiplier,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                owner_badge_address.clone()
            ))))
            .with_address(address_reservation)
            // .roles(roles! {
            //     hydrate_admin => admin_rule.clone();
            // })
            // .metadata(metadata! {
            //     init {
            //     "name" => name.clone(), updatable;
            //     "description" => description.clone(), updatable;
            //     "info_url" => Url::of(String::from("https://hydratestake.com")), updatable;
            //     "tags" => vec!["Hydrate"], updatable;
            //     "dapp_definition" => dapp_def_address.clone(), updatable;
            //     }
            // })
            .globalize()
        }

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
            let (new_instance, owner_badge) = Blueprint::<TokenCurve>::new(
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
            );
            (new_instance, owner_badge)
        }
    }
}
