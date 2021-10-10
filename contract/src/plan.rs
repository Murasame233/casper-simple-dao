use alloc::{string::String, vec, vec::Vec};
use casper_contract::contract_api::{
    runtime::{self, get_caller, get_named_arg, revert},
    storage,
};
use casper_types::{account::AccountHash, Key};

use crate::{error::PlanError, gardian};

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
    if runtime::has_key("vote") {
        revert(PlanError::AlreadyHaveProposal);
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
fn plan_to_online() {}
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
