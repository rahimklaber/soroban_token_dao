use core::ops::Add;

use soroban_sdk::{
    contracterror, contractimpl, contracttype, panic_with_error, symbol, unwrap::UnwrapOptimized,
    Address, Env, Vec,
};

use crate::balance::{receive_balance, spend_balance};
#[derive(Clone)]
#[contracttype]
pub struct PowerAtArgs {
    block: u32,
    ident: Address,
}
#[derive(Clone)]
#[contracttype]
pub struct DelegateAmountArgs {
    from: Address,
    to: Address,
}

#[contracttype]
pub enum DaoDataKey {
    // blocks where their voting power changed
    // Vec<u64>
    PChanges(Address),
    // power at block
    //u128
    PowerAt(PowerAtArgs),
    //u128
    //current power
    Power(Address),
    // amount delegated from, to to
    // i128
    DelegateTo(DelegateAmountArgs),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DaoError {
    NotEnoughToken = 0,
    CannotDelegateNegative = 1,
    PowerCannotBeNegative = 2,
    CannotAddNegativePower = 3,
    CannotRemoveNegativePower = 4,
    IncorrectNonce = 5,
}

pub trait DaoExtensionTrait {
    // Get voting power of a Identifier
    // We explicitly use Identifier instead of Address to allow for threshold signature schemes like FROST
    fn power(env: Env, of: Address) -> i128;
    fn power_at(env: Env, of: Address, at_block: u32) -> i128;
    // delegate power `from` to `to`
    fn delegate(env: Env, from: Address, to: Address, amount: i128);
    // remove delegation
    // amount is the amount we want to remove
    // from is the person who delegated
    fn r_delegate(env: Env, from: Address, to: Address, amount: i128);
    //get amount that `from` has delegated to `to`
    fn get_d_a(env: Env, from: Address, to: Address) -> i128;
}

struct DaoExtension;

#[contractimpl]
impl DaoExtensionTrait for DaoExtension {
    fn power(env: Env, of: Address) -> i128 {
        return get_power(&env, of);
    }

    fn power_at(env: Env, of: Address, at_block: u32) -> i128 {
        get_power_at_or_before(&env, of, at_block)
    }

    fn delegate(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        add_delgation(&env, from, to, amount)
    }

    fn r_delegate(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        remove_delegation(&env, from, to, amount)
    }

    fn get_d_a(env: Env, from: Address, to: Address) -> i128 {
        get_delagate_amount_from_to(&env, from, to)
    }
}

fn get_power(env: &Env, of: Address) -> i128 {
    env.storage()
        .get(&DaoDataKey::Power(of))
        .unwrap_or(Ok(0))
        .unwrap_optimized()
}

fn set_power(env: &Env, of: Address, power: i128) {
    env.storage().set(&DaoDataKey::Power(of), &power)
}

fn add_power(env: &Env, of: Address, amount: i128) {
    if amount < 0 {
        panic_with_error!(env, DaoError::CannotAddNegativePower);
    }
    let power = get_power(env, of.clone());
    let new_power = power + amount;

    set_power(env, of.clone(), new_power);
    add_power_change(env, of.clone(), new_power);
}

fn remove_power(env: &Env, of: Address, amount: i128) {
    if amount < 0 {
        panic_with_error!(env, DaoError::CannotRemoveNegativePower);
    }
    let power = get_power(env, of.clone());
    let new_power = power - amount;

    set_power(env, of.clone(), new_power);
    add_power_change(env, of.clone(), new_power)
}

// store that the power changed at this block
fn add_power_change(env: &Env, of: Address, power: i128) {
    let mut current_changes = get_power_changes(env, of.clone());

    // Todo: is this actually necesary?
    // check whether we haven't allready added this sequence to the vec. Might happen if two transaction in the same block calls this fun?
    // so don't add a duplicate sequence nr.
    if !(current_changes.len() > 0
        && current_changes.last().unwrap_optimized().unwrap_optimized() == env.ledger().sequence())
    {
        current_changes.push_back(env.ledger().sequence())
    }

    env.storage()
        .set(&DaoDataKey::PChanges(of.clone()), &current_changes);

    env.storage().set(
        &DaoDataKey::PowerAt(PowerAtArgs {
            block: env.ledger().sequence(),
            ident: of.clone(),
        }),
        &power,
    )
}

// get the blocks at which this identifier's power changed
fn get_power_changes(env: &Env, of: Address) -> Vec<u32> {
    env.storage()
        .get(&DaoDataKey::PChanges(of))
        .unwrap_or(Ok(Vec::new(env)))
        .unwrap_optimized()
}

fn get_power_at_or_before(env: &Env, of: Address, at_or_before: u32) -> i128 {
    let changes = get_power_changes(env, of.clone());
    if changes.len() == 0 {
        return 0;
    }
    let res = changes.binary_search(at_or_before);

    let latest_seq_at_or_before = if let Ok(_) = res {
        // `at_or_before` sequence is in list
        at_or_before
    } else {
        // index is the index where the new element should be inserted in the vec, so that it stays sorted.
        // This mean that to actually get the value we want we neec to do vec[index-1]
        let index = unsafe { res.unwrap_err_unchecked() };
        if index == 0 {
            return 0;
        }
        changes.get(index - 1).unwrap_optimized().unwrap_optimized()
    };

    env.storage()
        .get(&DaoDataKey::PowerAt(PowerAtArgs {
            block: latest_seq_at_or_before,
            ident: of.clone(),
        }))
        .unwrap_optimized()
        .unwrap_optimized()
}

fn add_delgation(env: &Env, from: Address, to: Address, amount: i128) {
    let current_delegate_amount = get_delagate_amount_from_to(env, from.clone(), to.clone());

    // remove_power(env, from.clone(), amount);
    add_power(env, to.clone(), amount);

    set_delgate_amount_from_to(
        env,
        from.clone(),
        to.clone(),
        amount + current_delegate_amount,
    );
    spend_balance(env, from.clone(), amount);
}

// `from` -> the person that delegated originally and wants to remove their delegation.
// `amount` -> amount that we want to remove. Should be positive and not negative.
fn remove_delegation(env: &Env, from: Address, to: Address, amount: i128) {
    let current_delegate_amount = get_delagate_amount_from_to(env, from.clone(), to.clone());

    // if we want to remove more than is delegated
    if amount > current_delegate_amount {
        panic_with_error!(env, DaoError::NotEnoughToken)
    }

    remove_power(env, to.clone(), amount);
    add_power(env, from.clone(), amount);

    set_delgate_amount_from_to(
        env,
        from.clone(),
        to.clone(),
        current_delegate_amount - amount,
    );
    receive_balance(env, from.clone(), amount);
}

fn get_delagate_amount_from_to(env: &Env, from: Address, to: Address) -> i128 {
    env.storage()
        .get(&DaoDataKey::DelegateTo(DelegateAmountArgs { from, to }))
        .unwrap_or(Ok(0))
        .unwrap_optimized()
}

fn set_delgate_amount_from_to(env: &Env, from: Address, to: Address, amount: i128) {
    env.storage().set(
        &DaoDataKey::DelegateTo(DelegateAmountArgs { from, to }),
        &amount,
    )
}

#[cfg(test)]
mod dao_test {
    extern crate std;
    use std::println;

    use crate::contract::TokenClient;
    use crate::dao::{DaoExtension, DaoExtensionClient};
    use soroban_sdk::{unwrap::UnwrapOptimized, vec, Bytes, Env};

    #[test]
    fn test() {
        // let env: Env = Default::default();
        // let token_contract_id = env.register_contract(None, crate::contract::Token);
        // env.register_contract(&token_contract_id,DaoExtension);
        // let user_1 = env.accounts().generate();
        // let user_2 = env.accounts().generate();

        // let token_client = TokenClient::new(&env, token_contract_id.clone());
        // let dao_ext_client = DaoExtensionClient::new(&env, token_contract_id.clone());
        // token_client.initialize(&user_1.clone().into(), &7, &Bytes::from_array(&env, b"DAO TOKEN"), &Bytes::from_array(&env, b"DTOKEN") );

        // token_client
        //     .with_source_account(&user_1)
        //     .mint(&soroban_auth::Signature::Invoker, &0, &user_2.clone().into(), &1000000000);

        // assert_eq!(1000000000, token_client.balance(&user_2.clone().into()));
        // // assert_eq!(0, dao_ext_client.power(&user_2.clone().into()));

        // let power_res = dao_ext_client.power(&user_2.clone().into());

        // println!("{:?}", power_res);

        // // let res = dao_ext_client
        // //     .with_source_account(&user_2)
        // //     .delegate(&soroban_auth::Signature::Invoker, &0, &user_2.clone().into(), &10);
    }
}
