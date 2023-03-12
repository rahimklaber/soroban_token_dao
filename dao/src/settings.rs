use soroban_sdk::{unwrap::UnwrapOptimized, Env};

use crate::data_keys::DataKey;

// `percent` -> percent of quorum needed to pass proposal.
// from 0 to 100
pub fn set_quorum(env: &Env, percent: u32) {
    env.storage().set(&DataKey::Quorum, &percent)
}

pub fn get_quorum(env: &Env) -> u32 {
    env.storage()
        .get(&DataKey::Quorum)
        .unwrap_optimized()
        .unwrap_optimized()
}

// set min duration of proposal
pub fn set_min_prop_duration(env: &Env, min_time_seconds: u32) {
    env.storage().set(&DataKey::MinTime, &min_time_seconds)
}

pub fn get_min_prop_duration(env: &Env) -> u32 {
    env.storage()
        .get(&DataKey::MinTime)
        .unwrap_optimized()
        .unwrap_optimized()
}
