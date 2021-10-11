use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::contract_api::{
    runtime::{self, get_caller, get_named_arg, revert},
    storage::{self, new_contract},
};
use casper_types::{account::AccountHash, contracts::NamedKeys, Key, U256};

use crate::{error::PlanError, gardian, online::online_entries};

#[no_mangle]
pub extern "C" fn proposal() {
    gardian("plan".into());
    // args
    // - plan: String (format: "{supply}")
    judge_original();
    if runtime::has_key("vote") {
        revert(PlanError::AlreadyHaveProposal);
    };
    let plan: String = get_named_arg("plan");
    storage::write(runtime::get_key("plan").unwrap().into_uref().unwrap(), plan);
    let i = get_original_index();
    // vote is a variable only can exist on plan.
    // format: (upvote, downvote)
    let mut vote = vec![0, 0, 0];
    vote[i] = 1;
    runtime::put_key("vote", Key::URef(storage::new_uref(vote)));
}

#[no_mangle]
pub extern "C" fn vote() {
    gardian("plan".into());
    judge_original();
    if !runtime::has_key("vote") {
        revert(PlanError::NoProposal);
    };
    let judge: bool = runtime::get_named_arg("vote");
    let mut vote =
        storage::read::<Vec<i32>>(runtime::get_key("vote").unwrap().into_uref().unwrap())
            .unwrap()
            .unwrap();
    let i = get_original_index();
    if judge {
        vote[i] = 1;
        if vote.iter().filter(|v| v == &&1).count() == 2 {
            runtime::remove_key("vote");
            plan_to_online();
        } else {
            storage::write(runtime::get_key("vote").unwrap().into_uref().unwrap(), vote)
        };
    } else {
        vote[i] = -1;
        if vote.iter().filter(|v| v == &&-1).count() == 2 {
            runtime::remove_key("vote")
        };
    }
}
fn plan_to_online() {
    storage::write(
        runtime::get_key("status").unwrap().into_uref().unwrap(),
        "online".to_string(),
    );
    let mut keys = NamedKeys::new();

    let plan: String = storage::read(runtime::get_key("plan").unwrap().into_uref().unwrap())
        .unwrap()
        .unwrap();
    let sup = U256::from(plan.parse::<usize>().unwrap());
    let sup_u = storage::new_uref(sup / 100 * 70);
    keys.insert("supply".into(), Key::URef(sup_u));
    keys.insert("reward".into(), Key::URef(storage::new_uref(U256::from(1))));
    let originals: Vec<AccountHash> = {
        let uref = runtime::get_key("originals").unwrap().into_uref().unwrap();
        storage::read::<Vec<AccountHash>>(uref).unwrap().unwrap()
    };
    let v: Vec<(AccountHash, U256)> = originals.into_iter().map(|f| (f, sup / 100 * 30)).collect();
    storage::write(runtime::get_key("accounting").unwrap().into_uref().unwrap(), v);
    keys.insert("accounting".into(), runtime::get_key("accounting").unwrap());
    keys.insert(
        "pledges".into(),
        Key::URef(storage::new_uref::<Option<Vec<(AccountHash, U256, bool)>>>(None)),
    );
    keys.insert(
        "pool".into(),
        Key::URef(storage::new_uref::<Option<(U256, U256)>>(None)),
    );
    keys.insert(
        "vote_limit".into(),
        Key::URef(storage::new_uref::<Option<U256>>(None)),
    );
    keys.insert(
        "proposal".into(),
        Key::URef(storage::new_uref::<Option<String>>(None)),
    );
    let (hash, _) = new_contract(online_entries(), Some(keys), None, None);
    storage::write(
        runtime::get_key("DAO_contract_hash")
            .unwrap()
            .into_uref()
            .unwrap(),
        hash,
    );
    runtime::remove_key("name")
}

// judge the caller is one of the originals
fn judge_original() {
    let caller = runtime::get_caller();
    let originals: Vec<AccountHash> = {
        let uref = runtime::get_key("originals").unwrap().into_uref().unwrap();
        storage::read::<Vec<AccountHash>>(uref).unwrap().unwrap()
    };
    if !originals.iter().any(|already| *already == caller) {
        revert(PlanError::NotOriginal)
    }
}

fn get_original_index() -> usize {
    let originals: Vec<AccountHash> = {
        let uref = runtime::get_key("originals").unwrap().into_uref().unwrap();
        storage::read::<Vec<AccountHash>>(uref).unwrap().unwrap()
    };
    let caller = get_caller();
    originals
        .iter()
        .enumerate()
        .find(|a| a.1 == &caller)
        .unwrap()
        .0
}
