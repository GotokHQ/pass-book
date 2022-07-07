mod utils;
use nft_pass_book::{
    error::NFTPassError, state::PassBookState,
};
use num_traits::FromPrimitive;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, transaction::TransactionError};
use utils::*;

#[tokio::test]
async fn success() {
    let (mut context, test_pass, user) = setup_pass_book(true).await;

    test_pass.activate(&mut context, &user).await.unwrap();
    let pass_book = test_pass.get_data(&mut context).await;

    assert_eq!(pass_book.state, PassBookState::Activated);
}

#[tokio::test]
async fn failure() {
    let (mut context, test_pass, user) = setup_pass_book(true).await;

    test_pass.activate(&mut context, &user).await.unwrap();
    context.warp_to_slot(1500).unwrap();

    let result = test_pass.activate(&mut context, &user).await;
    assert_custom_error!(
        result.unwrap_err().unwrap(),
        NFTPassError::PassBookIsAlreadyActivated,
        0
    );
}
