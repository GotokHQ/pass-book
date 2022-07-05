mod utils;

use mpl_token_metadata::state::Metadata;
use nft_pass_book::{error::NFTPassError, find_pass_store_program_address, instruction};
use num_traits::FromPrimitive;
use solana_program::{
    borsh::try_from_slice_unchecked, instruction::InstructionError, program_pack::Pack,
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::TransactionError};
use utils::*;

async fn setup(
    user: &User,
    buyer: &User,
    edition: u64,
) -> (
    ProgramTestContext,
    TestMetadata,
    TestMasterEditionV2,
    TestEditionMarker,
) {
    let mut context = nft_pass_book_program_test().start_with_context().await;

    let test_metadata = TestMetadata::new();
    let test_master_edition = TestMasterEditionV2::new(&test_metadata);
    let test_edition_marker = TestEditionMarker::new(&test_metadata, &test_master_edition, edition);
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            Some(&user.owner),
            10,
            false,
            &user.pubkey()
        )
        .await
        .unwrap();

    test_master_edition
        .create(&mut context, Some(10))
        .await
        .unwrap();

    test_edition_marker
        .create(&mut context, &buyer.pubkey())
        .await
        .unwrap();

    (
        context,
        test_metadata,
        test_master_edition,
        test_edition_marker,
    )
}

#[tokio::test]
async fn success_buy_with_spl_token() {
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

    let buyer = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };

    let (mut context, test_metadata, test_master_edition_v2, test_edition_marker) =
        setup(&user, &buyer, 1).await;
    let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &user.pubkey());
    let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());
    let test_store = TestStore::new(&user.pubkey());
    let trade_history = TestTradeHistory::new(&test_master_pass.pubkey, &buyer.pubkey());
    let usdc_token = TestSplToken::new();
    usdc_token
        .create(
            &mut context,
            1_000_000_0000,
            &user.token_account,
            &user.pubkey(),
        )
        .await
        .unwrap();
    usdc_token
        .mint_to(
            &mut context,
            10_000_000,
            &buyer.token_account,
            &buyer.pubkey(),
        )
        .await
        .unwrap();
    let creator_payout = TestPayout::new(&user.pubkey(), &usdc_token.mint.pubkey());
    let creator_payout2 = TestPayout::new(&context.payer.pubkey(), &usdc_token.mint.pubkey());
    let market_place_payout =
        TestPayout::new(&market_place_user.pubkey(), &usdc_token.mint.pubkey());
    let referrer_payout = TestPayout::new(&referrer.pubkey(), &usdc_token.mint.pubkey());
    let market_place_user = Some(&market_place_user);
    let referrer = Some(&referrer);
    //Some(&market_authority),
    //Some(&referrer),
    test_master_pass
        .init(
            &mut context,
            &test_master_edition_v2,
            &test_metadata,
            &user,
            &store,
            &usdc_token.mint.pubkey(),
            market_place_user,
            referrer,
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: true,
                duration: Some(30), //30 mins duration per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(5),
                blur_hash: None,
                price: 10_000_000,
                has_referrer: referrer.is_some(), // Some(referrer.pubkey()),
                has_market_authority: market_place_user.is_some(),
                referral_end_date: None,
                pieces_in_one_wallet: Some(1),
            },
        )
        .await
        .unwrap();

    test_master_pass
        .buy(
            &mut context,
            &test_metadata,
            &test_edition_marker,
            &test_store,
            &buyer,
            market_place_user,
            &trade_history,
            instruction::BuyPassArgs {
                market_fee_basis_point: 250,
                referral_share: 50,
                referral_kick_back_share: 0,
            },
        )
        .await
        .unwrap();
    let master_pass = test_master_pass.get_data(&mut context).await;
    let master_metadata = test_metadata.get_data(&mut context).await;
    let new_metadata_account =
        get_account(&mut context, &test_edition_marker.new_metadata_pubkey).await;
    let new_metadata = try_from_slice_unchecked::<Metadata>(&new_metadata_account.data).unwrap();

    let creator_payout = creator_payout.get_data(&mut context).await;
    let creator_token_holder_account =
        get_account(&mut context, &creator_payout.treasury_holder).await;
    let creator_token_holder =
        spl_token::state::Account::unpack_unchecked(&creator_token_holder_account.data).unwrap();

    let creator_payout2 = creator_payout2.get_data(&mut context).await;
    let creator2_token_holder_account =
        get_account(&mut context, &creator_payout2.treasury_holder).await;
    let creator2_token_holder =
        spl_token::state::Account::unpack_unchecked(&creator2_token_holder_account.data).unwrap();

    let market_payout = market_place_payout.get_data(&mut context).await;
    let market_token_holder_account =
        get_account(&mut context, &market_payout.treasury_holder).await;
    let market_token_holder =
        spl_token::state::Account::unpack_unchecked(&market_token_holder_account.data).unwrap();

    let referrer_payout = referrer_payout.get_data(&mut context).await;
    let referrer_token_holder_account =
        get_account(&mut context, &referrer_payout.treasury_holder).await;
    let referrer_token_holder =
        spl_token::state::Account::unpack_unchecked(&referrer_token_holder_account.data).unwrap();

    let trade_history = trade_history.get_data(&mut context).await;
    assert_eq!(trade_history.already_bought, 1);

    assert_eq!(creator_payout.cash_in, 4875000);
    assert_eq!(creator_token_holder.amount, 4875000);

    assert_eq!(creator_payout2.cash_in, 4875000);
    assert_eq!(creator2_token_holder.amount, 4875000);

    assert_eq!(market_payout.cash_in, 125000);
    assert_eq!(market_token_holder.amount, 125000);

    assert_eq!(referrer_payout.cash_in, 125000);
    assert_eq!(referrer_token_holder.amount, 125000);

    assert_eq!(master_metadata.data.name, new_metadata.data.name);
    assert_eq!(master_pass.supply, 1);
}

#[tokio::test]
async fn failure_multiple_buy() {
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

    let buyer = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };

    let (mut context, test_metadata, test_master_edition, test_edition_marker1) =
        setup(&user, &buyer, 1).await;
    let (store, _) = find_pass_store_program_address(&nft_pass_book::id(), &user.pubkey());
    let test_master_pass = TestPassBook::new(test_metadata.mint.pubkey());
    let test_store = TestStore::new(&user.pubkey());
    let trade_history = TestTradeHistory::new(&test_master_pass.pubkey, &buyer.pubkey());
    let usdc_token = TestSplToken::new();
    usdc_token
        .create(
            &mut context,
            1_000_000_0000,
            &user.token_account,
            &user.pubkey(),
        )
        .await
        .unwrap();
    usdc_token
        .mint_to(
            &mut context,
            50_000_000,
            &buyer.token_account,
            &buyer.pubkey(),
        )
        .await
        .unwrap();
    let market_place_user = Some(&market_place_user);
    let referrer = Some(&referrer);
    //Some(&market_authority),
    //Some(&referrer),
    test_master_pass
        .init(
            &mut context,
            &test_master_edition,
            &test_metadata,
            &user,
            &store,
            &usdc_token.mint.pubkey(),
            market_place_user,
            referrer,
            instruction::InitPassBookArgs {
                name: name.clone(),
                uri: uri.clone(),
                description: description.clone(),
                mutable: true,
                duration: Some(30), //30 mins duration per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(5),
                blur_hash: None,
                price: 10_000_000,
                has_referrer: referrer.is_some(), // Some(referrer.pubkey()),
                has_market_authority: market_place_user.is_some(),
                referral_end_date: None,
                pieces_in_one_wallet: Some(1),
            },
        )
        .await
        .unwrap();
    test_master_pass
        .buy(
            &mut context,
            &test_metadata,
            &test_edition_marker1,
            &test_store,
            &buyer,
            market_place_user,
            &trade_history,
            instruction::BuyPassArgs {
                market_fee_basis_point: 250,
                referral_share: 50,
                referral_kick_back_share: 0,
            },
        )
        .await
        .unwrap();

    let test_edition_marker2 = TestEditionMarker::new(&test_metadata, &test_master_edition, 2);
    test_edition_marker2
        .create(&mut context, &buyer.pubkey())
        .await
        .unwrap();
    let result = test_master_pass
        .buy(
            &mut context,
            &test_metadata,
            &test_edition_marker2,
            &test_store,
            &buyer,
            market_place_user,
            &trade_history,
            instruction::BuyPassArgs {
                market_fee_basis_point: 250,
                referral_share: 50,
                referral_kick_back_share: 0,
            },
        )
        .await;

    assert_custom_error!(
        result.unwrap_err().unwrap(),
        NFTPassError::UserReachBuyLimit,
        0
    );
}
