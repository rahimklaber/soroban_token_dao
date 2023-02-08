use soroban_auth::{Signature, Identifier};
use soroban_sdk::{Env, panic_with_error, unwrap::UnwrapOptimized};

use crate::{data_keys::DataKey, errors::ContractError};
// from token contract
pub fn verify_and_consume_nonce(e: &Env, auth: &Signature, expected_nonce: i128) {
    match auth {
        Signature::Invoker => {
            if expected_nonce != 0 {
                panic_with_error!(e, ContractError::InvalidNonce)
            }
            return;
        }
        _ => {}
    }

    let id = auth.identifier(e);
    let key = DataKey::Nonce(id.clone());
    let nonce = read_nonce(e, &id);

    if nonce != expected_nonce {
        panic_with_error!(e, ContractError::InvalidNonce)
    }
    e.storage().set(key, &nonce + 1);
}

pub fn read_nonce(e: &Env, id: &Identifier) -> i128 {
    let key = DataKey::Nonce(id.clone());
    e.storage().get(key).unwrap_or(Ok(0)).unwrap_optimized()
}