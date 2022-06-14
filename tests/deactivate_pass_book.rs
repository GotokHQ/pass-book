mod utils;
use nft_pass_book::{
    error::NFTPassError, find_pass_store_program_address, instruction, state::PassBookState,
};
use num_traits::FromPrimitive;
use solana_program::{instruction::InstructionError};
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::TransactionError
};
use utils::*;

async fn setup(
    activate: bool,
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
            &spl_token::native_mint::id(),
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: false,
                duration: Some(30), //30 mins duration per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(5),
                blur_hash: None,
                price: 0,
            },
        )
        .await
        .unwrap();

    if activate {
        test_master_pass
            .activate(&mut context, &user)
            .await
            .unwrap();
    }
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
    let (mut context, _test_metadata, _test_master_edition_v2, test_pass_book, user) =
        setup(true).await;

    test_pass_book
        .deactivate(&mut context, &user)
        .await
        .unwrap();
    let pass_book = test_pass_book.get_data(&mut context).await;

    assert_eq!(pass_book.state, PassBookState::Deactivated);
}

#[tokio::test]
async fn failure() {
    let (mut context, _test_metadata, _test_master_edition_v2, test_pass_book, user) =
        setup(true).await;

    test_pass_book
        .deactivate(&mut context, &user)
        .await
        .unwrap();
    context.warp_to_slot(3).unwrap();

    let result = test_pass_book.deactivate(&mut context, &user).await;
    assert_custom_error!(
        result.unwrap_err().unwrap(),
        NFTPassError::PassBookIsAlreadyDeactivated,
        0
    );
}

#[tokio::test]
async fn failure_invalid_state() {
    let (mut context, _test_metadata, _test_master_edition_v2, test_pass_book, user) =
        setup(false).await;

    let result = test_pass_book.deactivate(&mut context, &user).await;

    assert_custom_error!(result.unwrap_err().unwrap(), NFTPassError::PassNotActivated, 0);
}
