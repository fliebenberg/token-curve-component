use scrypto::prelude::*;

#[blueprint]
mod token_curves {
    struct TokenCurves {
        owner_badge_manager: ResourceManager,
    }

    impl TokenCurves {
        pub fn new(owner_badge_address: ResourceAddress) -> Global<TokenCurves> {
            let (address_reservation, _component_address) =
                Runtime::allocate_component_address(<TokenCurves>::blueprint_id());
            TokenCurves {
                owner_badge_manager: ResourceManager::from_address(owner_badge_address.clone()),
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
    }
}
