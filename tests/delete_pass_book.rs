mod utils;

use nft_pass_book::{find_pass_store_program_address, instruction};
use solana_program::{system_program::ID as system_id};
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer,
};
use utils::*;

async fn setup(
    mutable: bool,
) -> (
    ProgramTestContext,
    TestMetadata,
    TestMasterEditionV2,
    TestPassBook,
    User,
) {
    let mut context = nft_pass_book_program_test().start_with_context().await;

    let test_metadata = TestMetadata::new();
    let test_master_edition = TestMasterEditionV2::new(&test_metadata);
    let user = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
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

    let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &user.owner.pubkey());
    let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());

    let name = String::from("Pass Name");
    let uri = String::from("Some link to storage");
    let description = String::from("Pass description");

    test_master_pass
        .init(
            &mut context,
            &test_master_edition,
            &test_metadata,
            &user,
            &store,
            &system_id,
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: mutable,
                duration: Some(30), //30 mins duration per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(5),
                blur_hash: None,
                price: 0,
            },
        )
        .await
        .unwrap();

    (
        context,
        test_metadata,
        test_master_edition,
        test_master_pass,
        user,
    )
}

#[tokio::test]
async fn success() {
    let (mut context, test_metadata, test_master_edition_v2, test_pass_book, user) =
        setup(true).await;

    test_pass_book
        .delete(
            &mut context,
            &user,
            &user.pubkey(),
            &test_pass_book.token_account.pubkey(),
            &test_metadata.mint.pubkey(),
            Some(&user.token_account.pubkey()),
        )
        .await
        .unwrap();

    assert!(is_empty_account(&mut context, &test_pass_book.pubkey).await);
}

#[tokio::test]
async fn success_delete_and_close() {
    let (mut context, test_metadata, _test_master_edition_v2, test_pass_book, user) =
        setup(true).await;

    test_pass_book
        .delete(
            &mut context,
            &user,
            &user.pubkey(),
            &test_pass_book.token_account.pubkey(),
            &test_metadata.mint.pubkey(),
            None,
        )
        .await
        .unwrap();

    assert!(is_empty_account(&mut context, &test_pass_book.token_account.pubkey()).await);
    assert!(is_empty_account(&mut context, &test_pass_book.pubkey).await)
}
