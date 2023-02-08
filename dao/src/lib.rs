#![no_std]

mod data_keys;
mod errors;
mod proposal;
mod settings;
mod token;
mod nonce;

use data_keys::{check_init, set_init};
use nonce::{verify_and_consume_nonce, read_nonce};
use proposal::{add_proposal, Proposal, check_min_duration, get_prop_start_ledger, set_voted, add_for_votes, check_voted, add_against_votes, add_abstain_votes, get_for_votes, get_against_votes, get_abstain_votes, set_prop_start_ledger, get_min_proposal_power, get_proposal, set_min_proposal_power, check_min_prop_power, VotesCount, votes_counts};
use settings::{set_min_prop_duration, set_quorum, get_min_prop_duration, get_quorum};
use soroban_auth::{Signature, verify, Identifier};
use soroban_sdk::{contractimpl, BytesN, Env, symbol, contracttype, Symbol, assert_with_error};
use token::{get_dao_token_client, store_dao_token};

use crate::errors::ContractError;



#[contracttype]
#[derive(Clone)]
pub struct ProposalExtra{
    pub proposal: Proposal,
    pub start_seq: u32
}
pub trait DaoTrait {
    fn init(env: Env, dao_token_id: BytesN<32>, min_prop_duration: u32, min_quorum_percent: u32, min_prop_power: i128);
    //create proposal and return its id
    fn c_prop(env: Env, from: Signature, nonce: i128, proposal: Proposal) -> u32;

    //try to execute prop
    fn execute(env: Env, prop_id: u32);

    fn proposal(env: Env, prop_id: u32) -> ProposalExtra;

    //allow a member to vote on a proposal]
    fn vote_for(env: Env, from: Signature, nonce: i128, prop_id: u32);
    fn v_against(env: Env, from: Signature, nonce: i128, prop_id: u32);
    fn v_abstain(env: Env, from: Signature, nonce: i128, prop_id: u32);

    fn votes(env: Env, prop_id: u32) -> VotesCount;

    //min power to propose
    fn min_prop_p(env: Env) -> i128;
    // get minimum duration of proposal
    fn min_dur(env: Env) -> u32;
    //minimum percentage to for proposal to pass.
    // so for (votes + abstain / total_power) * 100 must be > quorum 
    fn quorum(env: Env) -> u32;
    fn nonce(env: Env, of: Identifier) -> i128;
}

pub struct DaoContract;

#[contractimpl]
impl DaoTrait for DaoContract {
    fn init(env: Env, dao_token_id: BytesN<32>, min_prop_duration: u32, min_quorum_percent: u32, min_prop_power: i128) {
        check_init(&env);
        // we need to be the dao token admin.
        // just assume that we are, and if not that people check.
        // can't see a way to check that I am an admin.
        // todo, maybe i should make this contract the token admin in this function
        store_dao_token(&env, dao_token_id);
        set_init(&env);
        set_min_prop_duration(&env, min_prop_duration);
        set_min_proposal_power(&env,min_prop_power);
        set_quorum(&env, min_quorum_percent);
    }

    fn c_prop(env: Env, signature: Signature, nonce: i128, proposal: Proposal) -> u32 {
        let identifier = signature.identifier(&env);

        check_min_duration(&env, &proposal);
        check_min_prop_power(&env, get_dao_token_client(&env).power(&identifier));        

        add_proposal(&env, proposal)
    }

    //try to execute prop
    fn execute(env: Env, prop_id: u32) {
        let proposal = get_proposal(&env, prop_id);

        assert_with_error!(&env, proposal.end_time >= env.ledger().timestamp(), ContractError::TooEarlyToExecute);
        
    }

    fn proposal(env: Env, prop_id: u32) -> ProposalExtra{
         ProposalExtra {
             proposal: get_proposal(&env, prop_id),
             start_seq: get_prop_start_ledger(&env, prop_id)
         }
    }

    //allow a member to vote on a proposal]
    fn vote_for(env: Env, from: Signature, nonce: i128, prop_id: u32){
        add_for_votes(&env, prop_id, vote_helper(&env, from, nonce, prop_id, symbol!("vote_for")));
    }
    
    fn v_against(env: Env, from: Signature, nonce: i128, prop_id: u32){
        add_against_votes(&env, prop_id, vote_helper(&env, from, nonce, prop_id, symbol!("v_against")))
    }

    fn v_abstain(env: Env, from: Signature, nonce: i128, prop_id: u32){
        add_abstain_votes(&env, prop_id, vote_helper(&env, from, nonce, prop_id, symbol!("v_abstain")))
    }

    fn votes(env: Env, prop_id: u32) -> VotesCount{
        votes_counts(&env, prop_id)
    }

    fn min_dur(env: Env) -> u32{
        get_min_prop_duration(&env)
    }

    fn quorum(env: Env) -> u32{
        get_quorum(&env)
    }

    fn nonce(env: Env, of: Identifier) -> i128{
        read_nonce(&env, &of)
    }
    
    fn min_prop_p(env: Env) -> i128{
        get_min_proposal_power(&env)
    }
}

// function to avoid code duplication in the vote functions

fn vote_helper(env: &Env, from: Signature, nonce: i128, prop_id: u32, symbol: Symbol)-> i128{
    let client = get_dao_token_client(&env);
    let start_ledger = get_prop_start_ledger(&env, prop_id);
    
    let from_id = from.identifier(&env);
    // check if person allready voted
    check_voted(&env, prop_id, from_id.clone());

    let power_at_start = client.power_at(&from_id, &start_ledger);

    verify(&env, &from, symbol, (&from_id, &nonce, &prop_id));
    verify_and_consume_nonce(&env, &from, nonce);

    set_voted(&env, prop_id, from_id);

    power_at_start
}