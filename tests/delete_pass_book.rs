mod utils;

use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use utils::*;

#[tokio::test]
async fn success() {
    let (mut context, test_pass, user) = setup_pass_book(true).await;

    test_pass
        .delete(
            &mut context,
            &user,
            &user.pubkey(),
        )
        .await
        .unwrap();

    assert!(is_empty_account(&mut context, &test_pass.account.pubkey()).await);
}