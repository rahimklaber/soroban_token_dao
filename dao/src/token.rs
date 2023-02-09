use soroban_sdk::{BytesN, Env};

use crate::data_keys::DataKey;

pub mod tokenclient {
    soroban_sdk::contractimport!(file = "../target/wasm32-unknown-unknown/release/dao_token.wasm");
}

pub fn get_dao_token_client(env: &Env) -> tokenclient::Client {
    let token_id: BytesN<32> = env
        .storage()
        .get(DataKey::DaoToken)
        .unwrap() // we don't handle error here. If this doesn't work, then we are screwed anyways.
        .unwrap();

    tokenclient::Client::new(&env, token_id)
}

pub fn store_dao_token(env: &Env, token_id: BytesN<32>) {
    env.storage().set(DataKey::DaoToken, token_id)
}
