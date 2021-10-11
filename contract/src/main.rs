#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;
mod accounting;
mod error;
mod join;
mod online;
mod plan;

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use casper_contract::contract_api::{
    runtime::{self, revert},
    storage,
};
use casper_types::{
    account::AccountHash, contracts::NamedKeys, CLType, ContractHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, Parameter, U256,
};

use crate::online::online_entries;

#[no_mangle]
pub extern "C" fn call() {
    // Variable on the storage:
    // - name: String (DAO name)
    // - originals: Vec<AccountHash> (the three people who create)
    // - status: String ( join | plan | online )
    // - DAO_contract_hash: ContractHash
    // - plan: String (format "{supply}")

    // Parse DAO name
    let name: String = runtime::get_named_arg("name");
    let name_uref = storage::new_uref(name);
    runtime::put_key("name", Key::URef(name_uref));

    // Parse creator to originals
    let creator = runtime::get_caller();
    let originals = vec![creator];
    let originals_uref = storage::new_uref(originals).into_read_write();
    runtime::put_key("originals", Key::URef(originals_uref));

    // status
    let name_uref = storage::new_uref("create".to_string());
    runtime::put_key("status", Key::URef(name_uref));

    // hash placeholder
    let c_hash = ContractHash::new([8u8; 32]);
    let c_hash_uref = storage::new_uref(c_hash).into_read_write();
    runtime::put_key("DAO_contract_hash", Key::URef(c_hash_uref));

    // plan
    let name_uref = storage::new_uref("".to_string());
    runtime::put_key("plan", Key::URef(name_uref));

    // Accounting
    let accounting: Vec<(AccountHash, U256)> = vec![];
    let accounting_uref = storage::new_uref(accounting);
    runtime::put_key("accounting", Key::URef(accounting_uref));

    // update contract
    let mut keys = NamedKeys::new();
    keys.insert("name".into(), runtime::get_key("name").unwrap());
    keys.insert("originals".into(), runtime::get_key("originals").unwrap());
    keys.insert("status".into(), runtime::get_key("status").unwrap());
    keys.insert("plan".into(), runtime::get_key("plan").unwrap());
    keys.insert("accounting".into(), runtime::get_key("accounting").unwrap());
    keys.insert(
        "DAO_contract_hash".into(),
        runtime::get_key("DAO_contract_hash").unwrap(),
    );

    let mut entries = online_entries();
    add_join_entry(&mut entries);
    add_plan_entry(&mut entries);
    let (package_hash, _) = storage::create_contract_package_at_hash();
    let (hash, _) = storage::add_contract_version(package_hash, entries, keys);

    // update hash
    storage::write(
        runtime::get_key("DAO_contract_hash")
            .unwrap()
            .into_uref()
            .unwrap(),
        hash,
    );

    // update status
    storage::write(
        runtime::get_key("status").unwrap().into_uref().unwrap(),
        "join".to_string(),
    );
}

fn add_join_entry(entries: &mut EntryPoints) {
    entries.add_entry_point(EntryPoint::new(
        "join",
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        casper_types::EntryPointType::Contract,
    ));
}

pub fn add_plan_entry(entries: &mut EntryPoints) {
    // for originals create proposal
    entries.add_entry_point(EntryPoint::new(
        "proposal",
        vec![Parameter::new("plan", CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    // for vote proposal
    entries.add_entry_point(EntryPoint::new(
        "vote",
        vec![Parameter::new("vote", CLType::Bool)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
}

fn gardian(accept: String) {
    if accept
        != storage::read::<String>(runtime::get_key("status").unwrap().into_uref().unwrap())
            .unwrap()
            .unwrap()
    {
        revert(error::Error::UnOpenEntry)
    }
}
