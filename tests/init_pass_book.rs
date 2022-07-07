mod utils;

use nft_pass_book::{
    error::NFTPassError,
    instruction,
    state::{
        AccountType, PassBookState, MAX_MEMBERSHIP_LEN, MAX_PASS_BOOK_LEN, MAX_PAYOUT_LEN,
        MAX_STORE_LEN, MAX_TRADE_HISTORY_LEN, STORE_AUTHORITY_LENGTH, USES_LENGTH,
        USE_AUTHORITY_LENGTH,
    }, // MAX_PASS_BOOK_LEN, MAX_STORE_LEN, MAX&_PAYOUT_LEN},
};
use num_traits::FromPrimitive;
use solana_program::{instruction::InstructionError};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::TransactionError};
use utils::*;

async fn setup(user: &User) -> (ProgramTestContext, TestPassBook, TestStore) {
    println!("MAX_PASS_BOOK_LEN: {}", MAX_PASS_BOOK_LEN);
    println!("MAX_STORE_LEN: {}", MAX_STORE_LEN);
    println!("MAX_PAYOUT_LEN: {}", MAX_PAYOUT_LEN);
    println!("MAX_MEMBERSHIP_LEN: {}", MAX_MEMBERSHIP_LEN);
    println!("MAX_TRADE_HISTORY_LEN: {}", MAX_TRADE_HISTORY_LEN);
    println!("USE_AUTHORITY_LENGTH: {}", USE_AUTHORITY_LENGTH);
    println!("STORE_AUTHORITY_LENGTH: {}", STORE_AUTHORITY_LENGTH);
    println!("USES_LENGTH: {}", USES_LENGTH);

    let context = nft_pass_book_program_test().start_with_context().await;
    let test_pass = TestPassBook::new();
    let test_store = TestStore::new(&user.pubkey());
    (context, test_pass, test_store)
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
    let referrer = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
    let market_authority = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };

    let (mut context, test_pass, test_store) = setup(&user).await;

    test_pass
        .init(
            &mut context,
            &user,
            &test_store.pubkey,
            &spl_token::native_mint::id(),
            Some(&market_authority),
            Some(&referrer),
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: true,
                max_uses: Some(30), //30 mins max_uses per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(5),
                price: 0,
                has_referrer: true,
                has_market_authority: true,
                referral_end_date: None,
            },
        )
        .await
        .unwrap();

    let master_pass = test_pass.get_data(&mut context).await;
    assert_eq!(master_pass.name.trim_matches(char::from(0)), name);
    assert_eq!(master_pass.uri.trim_matches(char::from(0)), uri);
    assert_eq!(
        master_pass.description.trim_matches(char::from(0)),
        description
    );
    assert_eq!(master_pass.account_type, AccountType::PassBook);
    assert!(master_pass.mutable);
    assert_eq!(master_pass.state, PassBookState::NotActivated);
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
    let referrer = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
    let market_authority = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
    let fake_admin = Keypair::new();
    let (mut context, test_pass, _) = setup(&admin).await;
    let fake_test_store = TestStore::new(&fake_admin.pubkey());
    let result = test_pass
        .init(
            &mut context,
            &admin,
            &fake_test_store.pubkey,
            &spl_token::native_mint::id(),
            Some(&market_authority),
            Some(&referrer),
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: true,
                max_uses: Some(30), //30 mins max_uses per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(10),
                price: 0,
                has_referrer: true,
                has_market_authority: true,
                referral_end_date: None,
            },
        )
        .await;
    assert_custom_error!(
        result.unwrap_err().unwrap(),
        NFTPassError::InvalidStoreKey,
        0
    );
}

#[tokio::test]
async fn success_spl_token() {
    //let mut context = gtk_packs_program_test().start_with_context().await;

    let name = String::from("Pass Name");
    let uri = String::from("some link to storage");
    let description = String::from("Pack description");
    let user = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
    let referrer = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
    let market_place_user = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };

    let (mut context, test_pass, test_store) = setup(&user).await;

    let usdc_token = TestSplToken::new(false);
    _ = usdc_token
        .create(&mut context, 1000, &user.token_account, &user.pubkey())
        .await;
    let market_place_user = Some(&market_place_user);
    let referrer = Some(&referrer);
    //Some(&market_authority),
    //Some(&referrer),
    test_pass
        .init(
            &mut context,
            &user,
            &test_store.pubkey,
            &usdc_token.pubkey(),
            market_place_user,
            referrer,
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: true,
                max_uses: Some(30), //30 mins max_uses per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(5),
                price: 10_000_000,
                has_referrer: referrer.is_some(), // Some(referrer.pubkey()),
                has_market_authority: market_place_user.is_some(),
                referral_end_date: None,
            },
        )
        .await
        .unwrap();

    let test_pass = test_pass.get_data(&mut context).await;
    assert_eq!(test_pass.name.trim_matches(char::from(0)), name);
    assert_eq!(test_pass.uri.trim_matches(char::from(0)), uri);
    assert_eq!(
        test_pass.description.trim_matches(char::from(0)),
        description
    );
    assert_eq!(test_pass.account_type, AccountType::PassBook);
    assert!(test_pass.mutable);
    assert_eq!(test_pass.state, PassBookState::NotActivated);
    assert_eq!(test_pass.mint, usdc_token.mint.pubkey());
    assert_eq!(test_pass.authority, user.owner.pubkey());
}
