#![cfg(test)]

extern crate std;
use std::println;

use crate::proposal::{Proposal, ProposalInstr};
use crate::token::tokenclient;
use crate::{token, DaoContract, DaoContractClient};
use ed25519_dalek::Keypair;
use rand::thread_rng;
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{symbol, vec, Bytes, Env, IntoVal, Vec, __bytes_lit_bytes, bytes};

fn generate_keypair() -> Keypair {
    Keypair::generate(&mut thread_rng())
}

#[test]
fn test() {
    let env: Env = Default::default();
    let token_contract_id = env.register_contract_wasm(None, tokenclient::WASM);
    let dao_contract_id = env.register_contract(None, DaoContract);

    let user_1 = env.accounts().generate();
    let user_2 = env.accounts().generate();

    let token_client = tokenclient::Client::new(&env, &token_contract_id.clone());

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

    env.ledger().set(LedgerInfo {
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

    env.ledger().set(LedgerInfo {
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
    assert_eq!(
        0,
        token_client.get_d_a(&user_2.clone().into(), &user_1.clone().into())
    );
    assert_eq!(
        10,
        token_client.get_d_a(&user_2.clone().into(), &user_2.clone().into())
    );

    let dao_client = DaoContractClient::new(&env, dao_contract_id.clone());

    token_client.with_source_account(&user_1).set_admin(
        &soroban_auth::Signature::Invoker,
        &0,
        &soroban_auth::Identifier::Contract(dao_contract_id.clone()),
    );

    dao_client.init(&token_contract_id, &1, &0, &10);

    let prop = Proposal {
        end_time: env.ledger().timestamp() + 10,
        instr: vec![
            &env,
            ProposalInstr {
                c_id: token_contract_id.clone(),
                fun_name: symbol!("mint"),
                args: vec![
                    &env,
                    soroban_auth::Signature::Invoker.into_val(&env),
                    (0i128.into_val(&env)),
                    Identifier::Account(user_2.clone()).into_val(&env),
                    (100i128.into_val(&env)),
                ],
            },
        ],
    };

    let prop_id = dao_client.with_source_account(&user_2.clone()).c_prop(
        &soroban_auth::Signature::Invoker,
        &0,
        &prop,
    );

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 11,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 1,
    });

    //todo shouldn't be able to vote after end time
    dao_client.with_source_account(&user_2.clone()).vote_for(
        &soroban_auth::Signature::Invoker,
        &0,
        &prop_id,
    );

    dao_client.execute(&prop_id);

    assert_eq!(190, token_client.balance(&user_2.clone().into()));

    let bytes = Bytes::from_array(&env, b"hi");
}
