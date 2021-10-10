use alloc::vec::Vec;
use casper_contract::contract_api::{
    runtime::{self, revert},
    storage,
};
use casper_types::{account::AccountHash, U256};

use crate::error::OnlineError;

// ==========
// helper functions
pub fn reduce(account: AccountHash, amount: U256) {
    let mut accounting = get_accounting();
    match accounting
        .iter_mut()
        .find(|(_account, _)| _account == &account)
    {
        Some((_, _amount)) => {
            if *_amount < amount {
                revert(OnlineError::UserHaveNoEnoughToken)
            } else {
                *_amount -= amount
            }
        }
        None => revert(OnlineError::UserHaveNoEnoughToken),
    }
    save_accounting(accounting)
}

pub fn add(account: AccountHash, amount: U256) {
    let mut accounting = get_accounting();
    match accounting
        .iter_mut()
        .find(|(_account, _)| _account == &account)
    {
        Some((_, _amount)) => *_amount += amount,
        None => accounting.push((account, amount)),
    }
    save_accounting(accounting);
}

pub fn get_accounting() -> Vec<(AccountHash, U256)> {
    storage::read(runtime::get_key("accounting").unwrap().into_uref().unwrap())
        .unwrap()
        .unwrap()
}

pub fn save_accounting(accounting: Vec<(AccountHash, U256)>) {
    storage::write(
        runtime::get_key("accounting").unwrap().into_uref().unwrap(),
        accounting,
    )
}
