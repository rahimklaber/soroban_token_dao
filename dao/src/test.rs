#![cfg(test)]

extern crate std;
use std::println;

use crate::{token::tokenclient};
use ed25519_dalek::Keypair;
use rand::thread_rng;
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{symbol, Bytes, Env};
use soroban_sdk::{testutils::Accounts};

fn generate_keypair() -> Keypair {
    Keypair::generate(&mut thread_rng())
}

#[test]
fn test() {
    let env: Env = Default::default();
    let token_contract_id = env.register_contract_wasm(None, tokenclient::WASM);

    let user_1 = env.accounts().generate();
    let user_2 = env.accounts().generate();

    let token_client = tokenclient::Client::new(&env, token_contract_id);

    token_client.initialize(
        &user_1.clone().into(),
        &7,
        &Bytes::from_array(&env, b"DAO TOKEN"),
        &Bytes::from_array(&env, b"DTOKEN"),
    );

    token_client.with_source_account(&user_1).mint(
        &soroban_auth::Signature::Invoker,
        &0,
        &user_2.clone().into(),
        &100,
    );

    assert_eq!(100, token_client.balance(&user_2.clone().into()));
    assert_eq!(0, token_client.power(&user_2.clone().into()));

    env.budget().reset();
    token_client.with_source_account(&user_2).delegate(
        &soroban_auth::Signature::Invoker,
        &0,
        &user_2.clone().into(),
        &10,
    );

    assert_eq!(10, token_client.power(&user_2.clone().into()));
    assert_eq!(10, token_client.power_at(&user_2.clone().into(), &0));
    assert_eq!(90, token_client.balance(&user_2.clone().into()));

    env.ledger().set(LedgerInfo{
        timestamp: env.ledger().timestamp() + 1,
        protocol_version: 1,
        sequence_number: 1,
        network_passphrase: Default::default(),
        base_reserve: 1,
    });

    token_client.with_source_account(&user_2).delegate(
        &soroban_auth::Signature::Invoker,
        &0,
        &user_1.clone().into(),
        &10,
    );
    
    assert_eq!(10, token_client.power(&user_1.clone().into()));
    assert_eq!(0, token_client.power_at(&user_1.clone().into(), &0));
    assert_eq!(80, token_client.balance(&user_2.clone().into()));
    

    env.ledger().set(LedgerInfo{
        timestamp: env.ledger().timestamp() + 1,
        protocol_version: 1,
        sequence_number: 2,
        network_passphrase: Default::default(),
        base_reserve: 1,
    });

    token_client.with_source_account(&user_2).r_delegate(
        &soroban_auth::Signature::Invoker,
        &0,
        &user_1.clone().into(),
        &10,
    );

    assert_eq!(0, token_client.power(&user_1.clone().into()));
    assert_eq!(90, token_client.balance(&user_2.clone().into()));
    assert_eq!(0, token_client.get_d_a(&user_2.clone().into(), &user_1.clone().into()));
    assert_eq!(10, token_client.get_d_a(&user_2.clone().into(), &user_2.clone().into()));


}
