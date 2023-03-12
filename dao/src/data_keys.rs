use soroban_sdk::{contracttype, panic_with_error, Address, Env};

use crate::{errors::ContractError, proposal::ProposalVoted};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // address of the DAO token
    DaoToken,
    Init,
    // -> Proposal struct
    Proposal(u32),
    //start ledger of proposal
    PropStart(u32),
    // the next proposal id to use
    ProposalId,
    // percentage of for votes + abstains needed to pass proposal
    // so a value of 75 would mean that 75% of voting power is needed to pass the propposal
    Quorum,
    //Minimum duration of proposal in seconds
    MinTime,
    // if this person voted for this proposal
    Voted(ProposalVoted),
    // abstain votes for this proposal
    AbstainV(u32),
    // fo votes
    ForVotes(u32),
    // against votes
    AgainstV(u32),
    Nonce(Address),
    // min power to propose
    MinPropP,
    //whether a proposal has been executedd
    Executed(u32),
}

pub fn check_init(env: &Env) {
    if is_init(&env) {
        panic_with_error!(&env, ContractError::NotInit)
    }
}

pub fn set_init(env: &Env) {
    env.storage().set(&DataKey::Init, &true)
}

pub fn is_init(env: &Env) -> bool {
    env.storage()
        .get(&DataKey::Init)
        .unwrap_or(Ok(false))
        .unwrap()
}
