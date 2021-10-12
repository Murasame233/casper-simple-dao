use core::ops::Div;

use crate::{
    accounting::{add, get_accounting, reduce},
    error::OnlineError,
};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::contract_api::{
    runtime::{self, get_caller, revert},
    storage,
};
use casper_types::{
    account::AccountHash, CLType, EntryPoint, EntryPoints, Parameter, PublicKey, U256,
};

// ============================
// The var in the storage used
// - supply: U256
// - reward: U256
// - accounting: Vec<(AccountHash, U256)>
// - pledges: Option<Vec<(AccountHash, U256)>>
// - pool: Option<(U256,U256)> (upvote,downvote)
// - vote_limit: Option<U256>
// - proposal: Option<String>

fn mint_to(account: AccountHash, amount: U256) {
    add(account, amount);
    let mut supply: U256 = storage::read(runtime::get_key("supply").unwrap().into_uref().unwrap())
        .unwrap()
        .unwrap();
    supply -= amount;
    storage::write(
        runtime::get_key("supply").unwrap().into_uref().unwrap(),
        supply,
    )
}

// pledge token for vote
fn pledges(account: AccountHash, amount: U256, vote: bool) {
    reduce(account, amount);
    if let Some(mut pledges) = read_key::<Option<Vec<(AccountHash, U256, bool)>>>("pledges") {
        match pledges
            .iter_mut()
            .find(|(acc, _, _bool)| acc == &account && _bool == &vote)
        {
            Some((_account, _amount, _)) => *_amount += amount,
            None => pledges.push((account, amount, vote)),
        }
        update_key("pledges", Some(pledges))
    };
}
fn pledges_back(result: bool) {
    let pledges: Vec<(AccountHash, U256, bool)> = read_key::<Option<_>>("pledges").unwrap();
    pledges.into_iter().map(|(account, amount, vote)| {
        add(account, amount);
        if vote == result {
            let reward: U256 =
                storage::read(runtime::get_key("reward").unwrap().into_uref().unwrap())
                    .unwrap()
                    .unwrap();

            mint_to(account, reward);
        }
    }).count();
}
fn execute() {
    let order: String = read_key::<Option<_>>("proposal").unwrap();
    let mut s = order.split_ascii_whitespace();
    let method = s.next().unwrap().to_string();
    if &method == "mint" {
        let amount = s.next().unwrap().to_string().parse::<U256>().unwrap();
        let account = AccountHash::from_formatted_str(s.next().unwrap()).unwrap();
        mint_to(account, amount);
        drop(account);
    } else if &method == "update" {
        if "reward" == s.next().unwrap() {
            let reward = U256::from(s.next().unwrap().to_string().parse::<usize>().unwrap());
            storage::write(
                runtime::get_key("reward").unwrap().into_uref().unwrap(),
                reward,
            );
        }
        // and so on
    }
    // and so on
}

#[no_mangle]
pub extern "C" fn transfer() {
    // ============
    // args:
    // amount: U256
    // recipient: AccountHash
    // ============
    let from: AccountHash = runtime::get_caller();
    let to: AccountHash = runtime::get_named_arg::<PublicKey>("recipient").to_account_hash();
    let amount: U256 = runtime::get_named_arg("amount");
    reduce(from, amount);
    add(to, amount);
}

// vote
#[no_mangle]
pub extern "C" fn vote_by_pledges() {
    // ============
    // args:
    // amount: U256
    // vote: bool
    // ============
    let account = runtime::get_caller();
    let amount: U256 = runtime::get_named_arg("amount");
    if amount < U256::from(1) {
        revert(OnlineError::NoZero)
    }
    let vote: bool = runtime::get_named_arg("vote");
    let vote_limit: U256 = read_key::<Option<_>>("vote_limit").unwrap();

    pledges(account, amount, vote);
    let mut pool: (U256, U256) = read_key::<Option<_>>("pool").unwrap();

    if vote {
        pool = (pool.0 + amount, pool.1);
    } else {
        pool = (pool.0, pool.1 + amount);
    }
    if pool.0 + pool.1 >= vote_limit {
        update_key::<Option<(U256, U256)>>("pool", None);
        if pool.0 > pool.1 {
            execute()
        }
        pledges_back(pool.0 > pool.1);
        update_key::<Option<Vec<(AccountHash, U256, bool)>>>("pledges", None);
        update_key::<Option<String>>("proposal",None);
        update_key::<Option<U256>>("vote_limit",None);
    } else {
        storage::write(runtime::get_key("pool").unwrap().into_uref().unwrap(), pool);
    }
}

// vote
#[no_mangle]
pub extern "C" fn new_proposal() {
    // ============
    // args:
    // proposal: String
    // vote_limit: U256
    // vote: bool
    // amount: U256
    // ============

    // Valid there is no active proposal
    if let Some(_) = read_key::<Option<(U256, U256)>>("pool") {
        revert(OnlineError::HaveUnFinishProposal)
    };
    // Valid caller
    let caller = get_caller();
    let accounting = get_accounting();
    if !accounting
        .iter()
        .any(|f| f.0 == caller && f.1 > U256::from(1))
    {
        revert(OnlineError::NoPermission)
    }

    let proposal: String = runtime::get_named_arg("proposal");
    let amount: U256 = runtime::get_named_arg("amount");
    let vote: bool = runtime::get_named_arg("vote");
    let vote_limit: U256 = runtime::get_named_arg("vote_limit");

    let mut s = proposal.split_ascii_whitespace();
    let first = s.next().unwrap();
    if !(first == "mint" || first == "update") {
        revert(OnlineError::InValidProposal)
    }

    if vote_limit < U256::from(20) {
        revert(OnlineError::TooSmall)
    }

    if amount > vote_limit.div(U256::from(2)) {
        revert(OnlineError::AmountTooBig)
    }

    update_key("proposal", Some(proposal));
    update_key("vote_limit", Some(vote_limit));
    update_key("pool", Some((U256::from(0), U256::from(0))));
    let pledge: Vec<(AccountHash, U256, bool)> = vec![];
    update_key("pledges", Some(pledge));

    // Save creator's vote
    if let Some(mut pool) = read_key::<Option<(U256, U256)>>("pool") {
        if amount > U256::from(0) {
            pledges(caller, amount, vote);
            if vote {
                pool = (pool.0 + amount, pool.1);
            } else {
                pool = (pool.0, pool.1 + amount);
            }
        }
        update_key("pool", Some(pool));
    }
}

fn update_key<T>(name: &str, value: T)
where
    T: casper_types::CLTyped + casper_types::bytesrepr::ToBytes,
{
    storage::write(runtime::get_key(name).unwrap().into_uref().unwrap(), value);
}

fn read_key<T>(name: &str) -> T
where
    T: casper_types::CLTyped + casper_types::bytesrepr::FromBytes,
{
    storage::read(runtime::get_key(name).unwrap().into_uref().unwrap())
        .unwrap()
        .unwrap()
}

pub fn online_entries() -> EntryPoints {
    let mut entries = EntryPoints::new();
    entries.add_entry_point(EntryPoint::new(
        "new_proposal",
        vec![
            Parameter::new("proposal", CLType::String),
            Parameter::new("vote_limit", CLType::U256),
            Parameter::new("vote", CLType::Bool),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Contract,
    ));
    entries.add_entry_point(EntryPoint::new(
        "vote_by_pledges",
        vec![
            Parameter::new("vote", CLType::Bool),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Contract,
    ));
    entries.add_entry_point(EntryPoint::new(
        "transfer",
        vec![
            Parameter::new("amount", CLType::U256),
            Parameter::new("recipient", CLType::PublicKey),
        ],
        CLType::Unit,
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Contract,
    ));
    entries
}
