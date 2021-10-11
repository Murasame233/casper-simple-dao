#[cfg(test)]
mod tests {

    use casper_engine_test_support::{Code, SessionBuilder, TestContextBuilder};
    use casper_types::{
        account::AccountHash, runtime_args, AsymmetricType, ContractHash, PublicKey, RuntimeArgs,
        U256, U512,
    };

    const ACCOUNT_A: [u8; 32] = [3u8; 32];
    const ACCOUNT_B: [u8; 32] = [6u8; 32];
    const ACCOUNT_C: [u8; 32] = [9u8; 32];

    #[test]
    fn test() {
        // Prepare Account
        let pub_a = PublicKey::ed25519_from_bytes(&ACCOUNT_A).unwrap();
        let pub_b = PublicKey::ed25519_from_bytes(&ACCOUNT_B).unwrap();
        let pub_c = PublicKey::ed25519_from_bytes(&ACCOUNT_C).unwrap();

        let account_a = pub_a.to_account_hash();
        let account_b = pub_b.to_account_hash();
        let account_c = pub_c.to_account_hash();

        // Prepare test context
        let mut context = TestContextBuilder::new()
            .with_public_key(pub_a, U512::from(100_000_000_000_000u64))
            .with_public_key(pub_b, U512::from(100_000_000_000_000u64))
            .with_public_key(pub_c, U512::from(100_000_000_000_000u64))
            .build();

        println!("prepare finished");
        // Deploy contract
        let contract_code = Code::from("contract.wasm");
        let create_args = runtime_args! {
            "name" => String::from("Test DAO")
        };
        let create_session = SessionBuilder::new(contract_code, create_args)
            .with_address(account_a)
            .with_authorization_keys(&[account_a])
            .build();
        context.run(create_session);

        // get contract hash
        let hash: ContractHash = context
            .query(account_a, &["DAO_contract_hash".into()])
            .unwrap()
            .into_t()
            .unwrap();

        assert_eq!(
            context
                .query(account_a, &["status".into()])
                .unwrap()
                .into_t::<String>()
                .unwrap(),
            "join".to_string()
        );

        // join
        let join_code = Code::Hash(hash.value(), "join".into());
        let join_b = SessionBuilder::new(join_code, runtime_args! {})
            .with_address(account_b)
            .with_authorization_keys(&[account_b])
            .build();

        let join_code = Code::Hash(hash.value(), "join".into());
        let join_c = SessionBuilder::new(join_code, runtime_args! {})
            .with_address(account_c)
            .with_authorization_keys(&[account_c])
            .build();
        context.run(join_b);
        context.run(join_c);
        // get new hash
        let hash: ContractHash = context
            .query(account_a, &["DAO_contract_hash".into()])
            .unwrap()
            .into_t()
            .unwrap();

        let status: String = context
            .query(account_a, &["status".into()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(status, "plan".to_string());

        // proposal for plam
        let proposal_code = Code::Hash(hash.value(), "proposal".into());
        let proposal = SessionBuilder::new(
            proposal_code,
            runtime_args! {"plan" => String::from("100000000")},
        )
        .with_address(account_a)
        .with_authorization_keys(&[account_a])
        .build();
        context.run(proposal);

        let vote_code = Code::Hash(hash.value(), "vote".into());
        let vote = SessionBuilder::new(vote_code, runtime_args! {"vote" => true})
            .with_address(account_b)
            .with_authorization_keys(&[account_b])
            .build();
        context.run(vote);

        let status: String = context
            .query(account_a, &["status".into()])
            .unwrap()
            .into_t()
            .unwrap();
        assert_eq!(status, "online".to_string());

        // now it's online
        let new_hash: ContractHash = context
            .query(account_a, &["DAO_contract_hash".into()])
            .unwrap()
            .into_t()
            .unwrap();

        dbg!(context
            .query(account_a, &["accounting".into()])
            .unwrap()
            .into_t::<Vec<(AccountHash, U256)>>()
            .unwrap());

        let proposal_code = Code::Hash(new_hash.value(), "new_proposal".into());
        let proposal = SessionBuilder::new(
            proposal_code,
            runtime_args! {   "proposal"=> String::from("update reward 2"),
            "vote_limit"=> U256::from(20),
            "vote" => true,
            "amount"=> U256::from(1)},
        )
        .with_address(account_a)
        .with_authorization_keys(&[account_a])
        .build();
        context.run(proposal);
        dbg!(context
            .query(account_a, &["accounting".into()])
            .unwrap()
            .into_t::<Vec<(AccountHash, U256)>>()
            .unwrap());

        let vote_code = Code::Hash(new_hash.value(), "vote_by_pledges".into());
        let vote = SessionBuilder::new(
            vote_code,
            runtime_args! {
            "vote" => true,
            "amount"=> U256::from(20)},
        )
        .with_address(account_b)
        .with_authorization_keys(&[account_b])
        .build();
        context.run(vote);

        dbg!(context
            .query(account_a, &["accounting".into()])
            .unwrap()
            .into_t::<Vec<(AccountHash, U256)>>()
            .unwrap());
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
