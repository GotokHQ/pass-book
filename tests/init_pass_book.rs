// mod utils;

// use mpl_token_metadata::state::{MAX_METADATA_LEN, MAX_EDITION_LEN, MAX_MASTER_EDITION_LEN, MAX_EDITION_MARKER_SIZE};
// // use mpl_token_metadata::state::{MAX_METADATA_LEN, MAX_EDITION_LEN, MAX_MASTER_EDITION_LEN, MAX_EDITION_MARKER_SIZE};
// use nft_pass_book::{
//     error::NFTPassError,
//     find_pass_store_program_address, instruction,
//     state::{AccountType, PassBookState, MAX_PASS_BOOK_LEN, MAX_PASS_STORE_LEN, MAX_PAYOUT_LEN}, // MAX_PASS_BOOK_LEN, MAX_PASS_STORE_LEN, MAX_PAYOUT_LEN},
// };
// use num_traits::FromPrimitive;
// use solana_program::{
//     instruction::InstructionError
// };
// use solana_program_test::*;
// use solana_sdk::{
//     signature::Keypair, signer::Signer, transaction::TransactionError
// };
// use utils::*;

// async fn setup(user: &User) -> (ProgramTestContext, TestMetadata, TestMasterEditionV2) {
//     let mut context = nft_pass_book_program_test().start_with_context().await;

//     let test_metadata = TestMetadata::new();
//     let test_master_edition = TestMasterEditionV2::new(&test_metadata);
//     println!("PASSBOOK LENGTH: {}", MAX_PASS_BOOK_LEN);
//     println!("STORE LENGTH: {}", MAX_PASS_STORE_LEN);
//     println!("PAYOUT LENGTH: {}", MAX_PAYOUT_LEN);
//     println!("MAX METADATA LENGTH: {}", MAX_METADATA_LEN);
//     println!("MAX EDITION LENGTH: {}", MAX_EDITION_LEN);
//     println!("MAX_MASTER_EDITION_LEN LENGTH: {}", MAX_MASTER_EDITION_LEN);
//     println!("MAX_EDITION_MARKER_SIZE LENGTH: {}", MAX_EDITION_MARKER_SIZE);
    

//     test_metadata
//         .create(
//             &mut context,
//             "Test".to_string(),
//             "TST".to_string(),
//             "uri".to_string(),
//             Some(&user.owner),
//             10,
//             false,
//         )
//         .await
//         .unwrap();

//     test_master_edition
//         .create(&mut context, Some(10))
//         .await
//         .unwrap();

//     (context, test_metadata, test_master_edition)
// }

// #[tokio::test]
// async fn success() {
//     //let mut context = gtk_packs_program_test().start_with_context().await;

//     let name = String::from("Pass Name");
//     let uri = String::from("some link to storage");
//     let description = String::from("Pack description");
//     let user = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };
//     let referrer = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };
//     let market_authority = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };

//     let (mut context, test_metadata, test_master_edition_v2) = setup(&user).await;
//     let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &user.owner.pubkey());
//     let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());

//     test_master_pass
//         .init(
//             &mut context,
//             &test_master_edition_v2,
//             &test_metadata,
//             &user,
//             &store,
//             &spl_token::native_mint::id(),
//             Some(&market_authority),
//             Some(&referrer),
//             instruction::InitPassBookArgs {
//                 name: name.clone(),
//                 uri: uri.clone(),
//                 description: description.clone(),
//                 mutable: true,
//                 duration: Some(30), //30 mins duration per session
//                 access: Some(30), //valid for 30 days
//                 max_supply: Some(5),
//                 blur_hash: None,
//                 price: 0,
//                 has_referrer: true,
//                 has_market_authority: true,
//                 referral_end_date: None,
//                 pieces_in_one_wallet: None,
//             },
//         )
//         .await
//         .unwrap();

//     let master_pass = test_master_pass.get_data(&mut context).await;
//     assert_eq!(master_pass.name.trim_matches(char::from(0)), name);
//     assert_eq!(master_pass.uri.trim_matches(char::from(0)), uri);
//     assert_eq!(
//         master_pass.description.trim_matches(char::from(0)),
//         description
//     );
//     assert_eq!(master_pass.account_type, AccountType::PassBook);
//     assert!(master_pass.mutable);
//     assert_eq!(master_pass.state, PassBookState::NotActivated);
//     assert_eq!(master_pass.authority, user.owner.pubkey());
// }

// #[tokio::test]
// async fn failure() {
//     //let mut context = gtk_packs_program_test().start_with_context().await;

//     let name = String::from("Pass Name");
//     let uri = String::from("some link to storage");
//     let description = String::from("Pack description");
//     let admin = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };
//     let referrer = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };
//     let market_authority = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };
//     let fake_admin = Keypair::new();
//     let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &fake_admin.pubkey());
//     let (mut context, test_metadata, test_master_edition_v2) = setup(&admin).await;
//     let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());

//     let result = test_master_pass
//         .init(
//             &mut context,
//             &test_master_edition_v2,
//             &test_metadata,
//             &admin,
//             &store,
//             &spl_token::native_mint::id(),
//             Some(&market_authority),
//             Some(&referrer),
//             instruction::InitPassBookArgs {
//                 name: name.clone(),
//                 uri: uri.clone(),
//                 description: description.clone(),
//                 mutable: true,
//                 duration: Some(30), //30 mins duration per session
//                 access: Some(30), //valid for 30 days
//                 max_supply: Some(10),
//                 blur_hash: None,
//                 price: 0,
//                 has_referrer: true,
//                 has_market_authority: true,
//                 referral_end_date: None,
//                 pieces_in_one_wallet: None,
//             },
//         )
//         .await;
//     assert_custom_error!(result.unwrap_err().unwrap(), NFTPassError::InvalidStoreKey, 1);
// }

// #[tokio::test]
// async fn success_spl_token() {
//     //let mut context = gtk_packs_program_test().start_with_context().await;

//     let name = String::from("Pass Name");
//     let uri = String::from("some link to storage");
//     let description = String::from("Pack description");
//     let user = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };
//     let referrer = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };
//     let market_place_user = User {
//         owner: Keypair::new(),
//         token_account: Keypair::new(),
//     };


//     let (mut context, test_metadata, test_master_edition_v2) = setup(&user).await;
//     let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &user.owner.pubkey());
//     let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());

//     let usdc_token = TestSplToken::new();
//     _ = usdc_token.create(&mut context, 1000, &user.token_account, &user.pubkey()).await;
//     let market_place_user = Some(&market_place_user);
//     let referrer = Some(&referrer);
//     //Some(&market_authority),
//     //Some(&referrer),
//     test_master_pass
//         .init(
//             &mut context,
//             &test_master_edition_v2,
//             &test_metadata,
//             &user,
//             &store,
//             &usdc_token.mint.pubkey(),
//             market_place_user,
//             referrer,
//             instruction::InitPassBookArgs {
//                 name: name.clone(),
//                 uri: uri.clone(),
//                 description: description.clone(),
//                 mutable: true,
//                 duration: Some(30), //30 mins duration per session
//                 access: Some(30), //valid for 30 days
//                 max_supply: Some(5),
//                 blur_hash: None,
//                 price: 0,
//                 has_referrer: referrer.is_some(), // Some(referrer.pubkey()),
//                 has_market_authority: market_place_user.is_some(),
//                 referral_end_date: None,
//                 pieces_in_one_wallet: None,
//             },
//         )
//         .await
//         .unwrap();

//     let master_pass = test_master_pass.get_data(&mut context).await;
//     assert_eq!(master_pass.name.trim_matches(char::from(0)), name);
//     assert_eq!(master_pass.uri.trim_matches(char::from(0)), uri);
//     assert_eq!(
//         master_pass.description.trim_matches(char::from(0)),
//         description
//     );
//     assert_eq!(master_pass.account_type, AccountType::PassBook);
//     assert!(master_pass.mutable);
//     assert_eq!(master_pass.state, PassBookState::NotActivated);
//     assert_eq!(master_pass.authority, user.owner.pubkey());
// }