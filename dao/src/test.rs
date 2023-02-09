#![cfg(test)]

extern crate std;
use std::println;

use ed25519_dalek::Keypair;
use rand::thread_rng;
use soroban_auth::Identifier;
use soroban_sdk::{Address::Contract, testutils::Accounts};
use soroban_sdk::{Env, symbol, Bytes};
use crate::{DaoContractClient, token};
use crate::{token::{tokenclient}, DaoContract};

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

    token_client.initialize(&user_1.clone().into(), &7, &Bytes::from_array(&env, b"DAO TOKEN"), &Bytes::from_array(&env, b"DTOKEN") );


    token_client
    .with_source_account(&user_1)
    .mint(&soroban_auth::Signature::Invoker, &0, &user_2.clone().into(), &1000000000);

    assert_eq!(1000000000, token_client.balance(&user_2.clone().into()));
    assert_eq!(0, token_client.power(&user_2.clone().into()));

    let budget_before = env.budget().cpu_instruction_cost();
    println!("budget before : {:?}", budget_before);

    let res = token_client
    .with_source_account(&user_2)
    .try_delegate(&soroban_auth::Signature::Invoker, &0, &user_2.clone().into(), &10);

    println!("Result : {:?}", res);

    let budget_after = env.budget().cpu_instruction_cost();
    println!("budget after : {:?}", budget_after);



}