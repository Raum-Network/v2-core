#![cfg(test)]
extern crate std;
use soroban_sdk::{testutils::{Address as _},
    Address, 
    BytesN, 
    Env,
    String};
use crate::{RaumFiFactoryClient};

// **** TOKEN CONTRACT ****
mod token {
    soroban_sdk::contractimport!(file = "../token/target/wasm32-unknown-unknown/release/rntoken.wasm");
    pub type TokenClient<'a> = Client<'a>;
}
use token::TokenClient;
fn create_token_contract<'a>(e: &Env) -> TokenClient<'a> {
    let token_address = &e.register_contract_wasm(None, token::WASM);
    let token = TokenClient::new(e, token_address);
    token
}

//  **** PAIR WASM ****
fn pair_token_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../pair/target/wasm32-unknown-unknown/release/raumfi_pair.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
}

// **** PAIR CONTRACT ****
mod pair {
    soroban_sdk::contractimport!(file = "../pair/target/wasm32-unknown-unknown/release/raumfi_pair.wasm");
    pub type RaumFiPairClient<'a> = Client<'a>;
}
use pair::RaumFiPairClient;


//  **** FACTORY CONTRACT **** 
fn create_factory_contract<'a>(e: & Env) -> RaumFiFactoryClient<'a> {
    let factory = RaumFiFactoryClient::new(e, &e.register_contract(None, crate::RaumFiFactory {}));
    factory
}


// THE TEST
pub struct RaumFiFactoryTest<'a> {
    env: Env,
    admin: Address,
    user: Address,
    token_0: TokenClient<'a>,
    token_1: TokenClient<'a>,
    token_2: TokenClient<'a>,
    token_3: TokenClient<'a>,
    pair_wasm: BytesN<32>,
    contract: RaumFiFactoryClient<'a>,
}

impl<'a> RaumFiFactoryTest<'a> {
    fn setup() -> Self {

        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let mut token_0 = create_token_contract(&env);
        let mut token_1 = create_token_contract(&env);
        if &token_1.address < &token_0.address {
            std::mem::swap(&mut token_0, &mut token_1);
        }

        let mut token_2 = create_token_contract(&env);
        let mut token_3 = create_token_contract(&env);
        if &token_3.address < &token_2.address {
            std::mem::swap(&mut token_2, &mut token_3);
        }

        token_0.initialize(&admin, &7, &String::from_str(&env, "Token 0"), &String::from_str(&env, "TOKEN0"));
        token_1.initialize(&admin, &7, &String::from_str(&env, "Token 1"), &String::from_str(&env, "TOKEN1"));
        token_2.initialize(&admin, &7, &String::from_str(&env, "Token 2"), &String::from_str(&env, "TOKEN2"));
        token_3.initialize(&admin, &7, &String::from_str(&env, "Token 3"), &String::from_str(&env, "TOKEN3"));

        
        let pair_wasm = pair_token_wasm(&env);  
        let contract = create_factory_contract(&env);

        env.budget().reset_unlimited();
    
        RaumFiFactoryTest {
            env,
            admin,
            user,
            token_0,
            token_1,
            token_2,
            token_3,
            pair_wasm,
            contract,
        }
    }
}

mod initialize;
mod fee_to_setter;
mod pairs;
mod events;

pub mod deterministic;