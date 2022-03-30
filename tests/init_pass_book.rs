mod utils;

use nft_pass_book::{
    error::NFTPassError,
    find_pass_store_program_address, instruction,
    state::{AccountType, DurationType, PassBookState},
};
use num_traits::FromPrimitive;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::TransactionError, transport::TransportError,
};
use utils::*;

async fn setup(user: &User) -> (ProgramTestContext, TestMetadata, TestMasterEditionV2) {
    let mut context = gtk_packs_program_test().start_with_context().await;

    let test_metadata = TestMetadata::new();
    let test_master_edition = TestMasterEditionV2::new(&test_metadata);

    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            10,
            false,
            &user.token_account,
            &user.owner.pubkey(),
        )
        .await
        .unwrap();

    test_master_edition
        .create(&mut context, Some(10))
        .await
        .unwrap();

    (context, test_metadata, test_master_edition)
}

#[tokio::test]
async fn success() {
    //let mut context = gtk_packs_program_test().start_with_context().await;

    let name = String::from("Pass Name");
    let uri = String::from("some link to storage");
    let description = String::from("Pack description");
    let user = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };

    let (mut context, test_metadata, test_master_edition_v2) = setup(&user).await;
    let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &user.owner.pubkey());
    let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());

    test_master_pass
        .init(
            &mut context,
            &test_master_edition_v2,
            &test_metadata,
            &user,
            &store,
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: true,
                duration: 30, //valid for 30 days
                duration_type: DurationType::Days,
                max_supply: Some(5),
            },
        )
        .await
        .unwrap();

    let master_pass = test_master_pass.get_data(&mut context).await;
    assert_eq!(master_pass.name.trim_matches(char::from(0)), name);
    assert_eq!(master_pass.uri.trim_matches(char::from(0)), uri);
    assert_eq!(
        master_pass.description.trim_matches(char::from(0)),
        description
    );
    assert_eq!(master_pass.account_type, AccountType::PassBook);
    assert!(master_pass.mutable);
    assert_eq!(master_pass.pass_state, PassBookState::NotActivated);
    assert_eq!(master_pass.authority, user.owner.pubkey());
}

#[tokio::test]
async fn failure() {
    //let mut context = gtk_packs_program_test().start_with_context().await;

    let name = String::from("Pass Name");
    let uri = String::from("some link to storage");
    let description = String::from("Pack description");
    let admin = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
    let fake_admin = Keypair::new();
    let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &fake_admin.pubkey());
    let (mut context, test_metadata, test_master_edition_v2) = setup(&admin).await;
    let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());

    let result = test_master_pass
        .init(
            &mut context,
            &test_master_edition_v2,
            &test_metadata,
            &admin,
            &store,
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: true,
                duration: 30, //valid for 30 days
                duration_type: DurationType::Days,
                max_supply: Some(10),
            },
        )
        .await;
    assert_custom_error!(result.unwrap_err(), NFTPassError::InvalidStoreKey, 1);
}
