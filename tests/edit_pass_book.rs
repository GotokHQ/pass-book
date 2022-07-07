mod utils;

use nft_pass_book::{error::NFTPassError};
use num_traits::FromPrimitive;
use solana_program::{instruction::InstructionError};
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, transaction::TransactionError,
};
use utils::*;

#[tokio::test]
async fn success() {
    let (mut context, test_pass, user) = setup_pass_book(true).await;
    assert_eq!(
        test_pass
            .get_data(&mut context)
            .await
            .name
            .trim_matches(char::from(0)),
        String::from("Pass Name")
    );

    test_pass
        .edit(
            &mut context,
            &user,
            Some(false),
            Some(String::from("New Pass Name")),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    let pass_book = test_pass.get_data(&mut context).await;

    assert_eq!(
        pass_book.name.trim_matches(char::from(0)),
        String::from("New Pass Name")
    );
    assert_eq!(pass_book.mutable, false);
}

#[tokio::test]
async fn failure() {
    let name = String::from("Pass Name");
    let (mut context, test_pass, user) = setup_pass_book(true).await;
    let result = test_pass
        .edit(
            &mut context,
            &user,
            Some(false),
            Some(name),
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert_custom_error!(result.unwrap_err().unwrap(), NFTPassError::CantSetTheSameValue, 0);
}

#[tokio::test]
async fn fail_immutable() {
    let (mut context, test_pass, user) = setup_pass_book(false).await;

    let result = test_pass
        .edit(
            &mut context,
            &user,
            None,
            Some(String::from("New Pass Name")),
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert_custom_error!(result.unwrap_err().unwrap(), NFTPassError::ImmutablePassBook, 0);
}
