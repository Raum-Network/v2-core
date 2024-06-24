use soroban_sdk::{
    Env,
    Address,
    testutils::{
        Address as _, 
        AuthorizedFunction, 
        AuthorizedInvocation,
        MockAuth,
        MockAuthInvoke,
    },
    IntoVal,
    Symbol,
    xdr::{
        ToXdr,
        ScAddress,
        ScVal,
        // ScObject,
        PublicKey,
        AccountId,
        Uint256
    },
    Bytes,
    TryFromVal,
};
use core::mem;

mod pair {
    soroban_sdk::contractimport!(file = "../pair/target/wasm32-unknown-unknown/release/raumfi_pair.wasm");
    pub type RaumFiPairClient<'a> = Client<'a>;
}
mod token {
    soroban_sdk::contractimport!(file = "../token/target/wasm32-unknown-unknown/release/rntoken.wasm");
    pub type TokenClient<'a> = Client<'a>;
}
mod factory {
    soroban_sdk::contractimport!(file = "./target/wasm32-unknown-unknown/release/raumfi_factory.wasm");
    pub type _RaumFiFactoryClient<'a> = Client<'a>; 
}
use pair::RaumFiPairClient;
use token::TokenClient;
use crate::{ RaumFiFactory, RaumFiFactoryClient};

struct RaumFiFactoryTest<'a> {
    env: Env,
    alice: Address,
    bob: Address,
    factory: RaumFiFactoryClient<'a>,
    token_0: TokenClient<'a>,
    token_1: TokenClient<'a>,
    pair: RaumFiPairClient<'a>
}

impl<'a> RaumFiFactoryTest<'a> {
    fn new() -> Self {
        
        let env: Env = Default::default();
        env.mock_all_auths();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let mut token_0: TokenClient<'a> = TokenClient::new(&env, &env.register_stellar_asset_contract(alice.clone()));
        let mut token_1: TokenClient<'a> = TokenClient::new(&env, &env.register_stellar_asset_contract(alice.clone()));
        if &token_1.address < &token_0.address {
            mem::swap(&mut token_0, &mut token_1);
        } else 
        if &token_1.address == &token_0.address {
            panic!("token contract ids are equal");
        }
        
        let factory_address = &env.register_contract(None, RaumFiFactory);
        let pair_hash = env.deployer().upload_contract_wasm(pair::WASM);
        let factory = RaumFiFactoryClient::new(&env, &factory_address);
        factory.initialize(&alice, &pair_hash);
        factory.create_pair(&token_0.address, &token_1.address);
        let pair_address = factory.get_pair(&token_0.address, &token_1.address);
        let pair = RaumFiPairClient::new(&env, &pair_address);

        RaumFiFactoryTest {
            env,
            alice,
            bob,
            factory,
            token_0,
            token_1,
            pair
        }
    }
}

#[test]
pub fn create_and_register_factory_contract() {
    let _factory_test = RaumFiFactoryTest::new();
}

#[test]
pub fn token_client_ne() {
    let factory_test = RaumFiFactoryTest::new();
    assert_ne!(factory_test.token_0.address, factory_test.token_1.address);
}

#[test]
pub fn setter_is_alice() {
    let factory_test = RaumFiFactoryTest::new();
    assert_eq!(factory_test.factory.fee_to_setter(), factory_test.alice);
}

#[test]
pub fn fees_are_not_enabled() {
    let factory_test = RaumFiFactoryTest::new();
    assert_eq!(factory_test.factory.fees_enabled(), false);
}

#[test]
pub fn set_fee_to_setter_bob() {
    let factory_test = RaumFiFactoryTest::new();
    let bob = factory_test.bob;
    factory_test.factory.set_fee_to_setter(&bob);
    let setter = factory_test.factory.fee_to_setter();
    assert_eq!(setter, bob);
}

#[test]
pub fn authorize_bob() {
    let factory_test = RaumFiFactoryTest::new();
    let factory = factory_test.factory;
    let factory_address = factory.address.clone();
    let alice_address = factory_test.alice.clone();
    let bob = factory_test.bob.clone();
    factory.set_fee_to_setter(&bob);
    let auths = [(
        alice_address,
        AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
                factory_address,
                Symbol::new(&factory.env, "set_fee_to_setter"),
                (bob.clone(),).into_val(&factory.env)
            )),
            sub_invocations:[].into()
        }
    )];
    assert_eq!(factory.env.auths(), auths);
}

#[test]
pub fn set_fees_enabled() {
    let factory_test = RaumFiFactoryTest::new();
    let factory = factory_test.factory;
    factory.set_fees_enabled(&true);
    assert_eq!(factory.fees_enabled(), true);
}

#[test]
pub fn set_fee_to_factory_address() {
    let factory_test = RaumFiFactoryTest::new();
    let factory = factory_test.factory;
    factory.set_fees_enabled(&true);
    factory.set_fee_to(&factory.address);
    assert_eq!(factory.fee_to(), factory.address);
}

#[test]
pub fn add_pair() {
    let factory_test = RaumFiFactoryTest::new();
    let factory = factory_test.factory;
    let alice = factory_test.alice.clone();
    let token_a = TokenClient::new(&factory.env, &factory.env.register_stellar_asset_contract(alice.clone()));
    let token_b = TokenClient::new(&factory.env, &factory.env.register_stellar_asset_contract(alice.clone()));
    factory.create_pair(&token_a.address, &token_b.address);
    assert_eq!(factory.pair_exists(&token_a.address, &token_b.address), true);
    assert_eq!(factory.pair_exists(&token_b.address, &token_a.address), true);
}

#[test]
pub fn compare_pair_address() {
    let factory_test = RaumFiFactoryTest::new();
    let token_0_address = factory_test.token_0.address;
    let token_1_address = factory_test.token_1.address;
    let pair_address = factory_test.factory.get_pair(&token_0_address, &token_1_address);
    assert_eq!(pair_address, factory_test.pair.address);
}

#[test]
#[should_panic]
pub fn pair_is_unique_and_unequivocal() {
    let factory_test = RaumFiFactoryTest::new();
    let factory = factory_test.factory;
    let alice = factory_test.alice.clone();
    let token_a = TokenClient::new(&factory.env, &factory.env.register_stellar_asset_contract(alice.clone()));
    let token_b = TokenClient::new(&factory.env, &factory.env.register_stellar_asset_contract(alice.clone()));
    factory.create_pair(&token_a.address, &token_b.address);
    factory.create_pair(&token_a.address, &token_b.address);
}

#[test]
pub fn authorized_invocation() {
    let factory_test = RaumFiFactoryTest::new();
    let factory = factory_test.factory;
    let alice = factory_test.alice.clone();
    let bob = factory_test.bob.clone();

    // alice is not equal to bob
    assert_ne!(alice, bob);
    // alice is fee_to_setter
    assert_eq!(alice, factory.fee_to_setter());

    let _r = factory
        .mock_auths(&[MockAuth {
            address: &alice,
            invoke: &MockAuthInvoke {
                contract: &factory.address,
                fn_name: "set_fee_to_setter",
                args: (&bob,).into_val(&factory_test.env),
                sub_invokes: &[],
            },
        }])
        .set_fee_to_setter(&bob);

    // setter is bob
    assert_eq!(bob, factory.fee_to_setter());
}

#[test]
#[should_panic]
pub fn non_authorized_invocation() {
    let factory_test = RaumFiFactoryTest::new();
    let factory = factory_test.factory;
    let alice = factory_test.alice.clone();
    let bob = factory_test.bob.clone();

    // alice is not equal to bob
    assert_ne!(alice, bob);
    // alice is fee_to_setter
    assert_eq!(alice, factory.fee_to_setter());

    let _r = factory
        .mock_auths(&[MockAuth {
            address: &bob,
            invoke: &MockAuthInvoke {
                contract: &factory.address,
                fn_name: "set_fee_to_setter",
                args: (&bob,).into_val(&factory_test.env),
                sub_invokes: &[],
            },
        }])
        .set_fee_to_setter(&bob);
        
    // setter is bob
    assert_eq!(bob, factory.fee_to_setter());
}