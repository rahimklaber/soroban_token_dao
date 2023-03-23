#![cfg(test)]

extern crate std;

use crate::proposal::{Proposal, ProposalInstr};
use crate::token::tokenclient;
use crate::{DaoContract, DaoContractClient};
use soroban_sdk::testutils::{Ledger, LedgerInfo, Address as _};
use soroban_sdk::{symbol, vec, Bytes, Env, IntoVal, Address};


#[test]
fn test() {
    let env: Env = Default::default();
    let token_contract_id = env.register_contract_wasm(None, tokenclient::WASM);
    let dao_contract_id = env.register_contract(None, DaoContract);

    let user_1 = Address::random(&env);
    let user_2 = Address::random(&env);

    let token_client = tokenclient::Client::new(&env, &token_contract_id.clone());

    token_client.initialize(
        &user_1.clone().into(),
        &7,
        &Bytes::from_array(&env, b"DAO TOKEN"),
        &Bytes::from_array(&env, b"DTOKEN"),
    );

    token_client.mint(
        &user_1,
        &user_2,
        &100,
    );

    assert_eq!(100, token_client.balance(&user_2.clone().into()));
    assert_eq!(0, token_client.power(&user_2.clone().into()));

    env.budget().reset();
    token_client.delegate(
        &user_2,
        &user_2,
        &10,
    );

    assert_eq!(10, token_client.power(&user_2.clone().into()));
    assert_eq!(10, token_client.power_at(&user_2.clone().into(), &0));
    assert_eq!(90, token_client.balance(&user_2.clone().into()));

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 1,
        protocol_version: 1,
        sequence_number: 1,
        base_reserve: 1,
        network_id: Default::default()
    });

    token_client.delegate(
        &user_2,
        &user_1.clone(),
        &10,
    );

    assert_eq!(10, token_client.power(&user_1.clone().into()));
    assert_eq!(0, token_client.power_at(&user_1.clone().into(), &0));
    assert_eq!(80, token_client.balance(&user_2.clone().into()));

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 1,
        protocol_version: 1,
        sequence_number: 2,
        base_reserve: 1,
        network_id: Default::default()
    });

    token_client.r_delegate(
        &user_2,
        &user_1,
        &10,
    );

    assert_eq!(0, token_client.power(&user_1));
    assert_eq!(90, token_client.balance(&user_2));
    assert_eq!(
        0,
        token_client.get_d_a(&user_2, &user_1)
    );
    assert_eq!(
        10,
        token_client.get_d_a(&user_2, &user_2)
    );

    let dao_client = DaoContractClient::new(&env, &dao_contract_id);

    token_client.set_admin(
        &user_1,
        &Address::from_contract_id(&env, &dao_contract_id),
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
                    Address::from_contract_id(&env, &dao_contract_id).into_val(&env),
                    user_2.into_val(&env),
                    (100i128.into_val(&env)),
                ],
            },
        ],
    };

    let prop_id = dao_client.c_prop(
        &user_2,
        &prop,
    );

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 11,
        protocol_version: 1,
        sequence_number: 10,
        base_reserve: 1,
        network_id: Default::default()
    });

    //todo shouldn't be able to vote after end time
    dao_client.vote_for(
        &user_2,
        &prop_id,
    );

    dao_client.execute(&prop_id);

    assert_eq!(190, token_client.balance(&user_2.clone().into()));

}
