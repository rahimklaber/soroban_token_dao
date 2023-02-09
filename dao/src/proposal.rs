use soroban_auth::Identifier;
use soroban_sdk::{
    contracttype, panic_with_error, unwrap::UnwrapOptimized, BytesN, Env, RawVal, Symbol, Vec,
};

use crate::{data_keys::DataKey, errors::ContractError, settings::get_min_prop_duration};

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalVoted {
    pub voter: Identifier,
    pub prop_id: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalInstr {
    //contract id
    pub c_id: BytesN<32>,
    pub fun_name: Symbol,
    pub args: Vec<RawVal>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub end_time: u64,
    // instrunctions will be executed in sequence
    pub instr: Vec<ProposalInstr>,
}

#[contracttype]
#[derive(Clone)]
pub struct VotesCount {
    pub v_for: i128,
    pub v_against: i128,
    pub v_abstain: i128,
}
// add prop and return its id
pub fn add_proposal(env: &Env, proposal: Proposal) -> u32 {
    let prop_id = get_and_inc_prop_id(env);

    env.storage().set(DataKey::Proposal(prop_id), proposal);
    set_prop_start_ledger(env, prop_id, env.ledger().sequence());

    prop_id
}

pub fn get_proposal(env: &Env, prop_id: u32) -> Proposal {
    env.storage()
        .get(DataKey::Proposal(prop_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidProposalId))
        .unwrap_optimized()
}

fn get_and_inc_prop_id(env: &Env) -> u32 {
    let prev = env
        .storage()
        .get(DataKey::ProposalId)
        .unwrap_or(Ok(0u32))
        .unwrap_optimized();

    env.storage().set(DataKey::ProposalId, prev + 1);
    prev
}

pub fn check_min_duration(env: &Env, proposal: &Proposal) {
    let min_duration = get_min_prop_duration(env);
    if proposal.end_time - env.ledger().timestamp() < (min_duration as u64) {
        panic_with_error!(env, ContractError::MinDurationNotSatisfied)
    }
}

pub fn set_voted(env: &Env, prop_id: u32, voter: Identifier) {
    env.storage()
        .set(DataKey::Voted(ProposalVoted { voter, prop_id }), true)
}

pub fn get_voted(env: &Env, prop_id: u32, voter: Identifier) -> bool {
    env.storage()
        .get(DataKey::Voted(ProposalVoted { voter, prop_id }))
        .unwrap_or(Ok(false))
        .unwrap_optimized()
}

pub fn check_voted(env: &Env, prop_id: u32, voter: Identifier) {
    if get_voted(env, prop_id, voter) {
        panic_with_error!(env, ContractError::AlreadyVoted)
    }
}

pub fn set_prop_start_ledger(env: &Env, prop_id: u32, start_ledger: u32) {
    env.storage().set(DataKey::PropStart(prop_id), start_ledger)
}

pub fn get_prop_start_ledger(env: &Env, prop_id: u32) -> u32 {
    env.storage()
        .get(DataKey::PropStart(prop_id))
        .unwrap_optimized()
        .unwrap_optimized()
}

pub fn get_for_votes(env: &Env, prop_id: u32) -> i128 {
    env.storage()
        .get(DataKey::ForVotes(prop_id))
        .unwrap_or(Ok(0))
        .unwrap_optimized()
}

fn set_for_votes(env: &Env, prop_id: u32, amount: i128) {
    env.storage().set(DataKey::ForVotes(prop_id), amount)
}

pub fn add_for_votes(env: &Env, prop_id: u32, amount: i128) {
    if amount < 0 {
        panic_with_error!(env, ContractError::CannotAddNegativeVote)
    }

    let curr_votes = get_for_votes(env, prop_id);
    set_for_votes(env, prop_id, curr_votes + amount)
}

pub fn get_against_votes(env: &Env, prop_id: u32) -> i128 {
    env.storage()
        .get(DataKey::AgainstV(prop_id))
        .unwrap_or(Ok(0))
        .unwrap_optimized()
}

fn set_against_votes(env: &Env, prop_id: u32, amount: i128) {
    env.storage().set(DataKey::AgainstV(prop_id), amount)
}

pub fn add_against_votes(env: &Env, prop_id: u32, amount: i128) {
    if amount < 0 {
        panic_with_error!(env, ContractError::CannotAddNegativeVote)
    }

    let curr_votes = get_against_votes(env, prop_id);
    set_against_votes(env, prop_id, curr_votes + amount)
}

pub fn get_abstain_votes(env: &Env, prop_id: u32) -> i128 {
    env.storage()
        .get(DataKey::AbstainV(prop_id))
        .unwrap_or(Ok(0))
        .unwrap_optimized()
}

fn set_abstain_votes(env: &Env, prop_id: u32, amount: i128) {
    env.storage().set(DataKey::AbstainV(prop_id), amount)
}

pub fn add_abstain_votes(env: &Env, prop_id: u32, amount: i128) {
    if amount < 0 {
        panic_with_error!(env, ContractError::CannotAddNegativeVote)
    }

    let curr_votes = get_abstain_votes(env, prop_id);
    set_abstain_votes(env, prop_id, curr_votes + amount)
}

pub fn set_min_proposal_power(env: &Env, min_power: i128) {
    env.storage().set(DataKey::MinPropP, min_power)
}

pub fn get_min_proposal_power(env: &Env) -> i128 {
    env.storage()
        .get(DataKey::MinPropP)
        .unwrap_optimized()
        .unwrap_optimized()
}

pub fn check_min_prop_power(env: &Env, power: i128) {
    if get_min_proposal_power(env) > power {
        panic_with_error!(env, ContractError::NotEnoughPower)
    }
}

pub fn votes_counts(env: &Env, prop_id: u32) -> VotesCount {
    let for_votes = get_for_votes(&env, prop_id);
    let against_votes = get_against_votes(&env, prop_id);
    let abstain_votes = get_abstain_votes(&env, prop_id);

    VotesCount {
        v_for: for_votes,
        v_against: against_votes,
        v_abstain: abstain_votes,
    }
}
