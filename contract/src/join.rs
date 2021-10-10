use alloc::{string::ToString, vec::Vec};
use casper_contract::contract_api::{
    runtime::{self, revert},
    storage,
};
use casper_types::account::AccountHash;

use crate::{error::CreateError, gardian};

#[no_mangle]
pub extern "C" fn join() {
    gardian("join".into());
    let caller = runtime::get_caller();
    let mut originals: Vec<AccountHash> = {
        let uref = runtime::get_key("originals").unwrap().into_uref().unwrap();
        storage::read::<Vec<AccountHash>>(uref).unwrap().unwrap()
    };
    if originals.iter().any(|already| *already == caller) {
        revert(CreateError::AlreadyJoin)
    };
    originals.push(caller);
    let len = originals.len();
    storage::write(
        runtime::get_key("originals")
            .unwrap()
            .into_uref()
            .unwrap()
            .into_write(),
        originals.clone(),
    );
    if len == 3 {
        join_to_plan();
    }
}

fn join_to_plan() {
    storage::write(
        runtime::get_key("status").unwrap().into_uref().unwrap(),
        "plan".to_string(),
    );
}
