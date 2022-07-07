mod utils;

use nft_pass_book::{error::NFTPassError, instruction};
use num_traits::FromPrimitive;
use solana_program::{clock::Clock, instruction::InstructionError, program_pack::Pack};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::TransactionError};
use utils::*;

async fn setup(
    user: &User,
    buyer: &User,
    amount: u64,
    is_native: bool,
) -> (
    ProgramTestContext,
    TestPassBook,
    TestStore,
    TestTradeHistory,
    TestSplToken,
    TestMembership,
) {
    let mut context = nft_pass_book_program_test().start_with_context().await;
    let test_pass = TestPassBook::new();
    let test_store = TestStore::new(&user.pubkey());
    let trade_history = TestTradeHistory::new(&test_pass.account.pubkey(), &buyer.pubkey());
    let membership = TestMembership::new(&test_store.pubkey, &buyer.pubkey());
    let token = TestSplToken::new(is_native);
    if is_native {
        token
            .airdrop(&mut context, amount, &buyer.pubkey())
            .await
            .unwrap()
    } else {
        token
            .create(
                &mut context,
                1_000_000_000_000,
                &user.token_account,
                &&user.pubkey(),
            )
            .await
            .unwrap();
        token
            .mint_to(&mut context, amount, &buyer.token_account, &buyer.pubkey())
            .await
            .unwrap();
    }
    (context, test_pass, test_store, trade_history, token, membership)
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

    let (mut context, test_pass, test_store, trade_history, token, membership) =
        setup(&user, &buyer, 10_000_000, false).await;
    let test_creator_payout = TestPayout::new(&user.pubkey(), &token.pubkey());
    let test_market_place_payout = TestPayout::new(&market_place_user.pubkey(), &token.pubkey());
    let test_referrer_payout = TestPayout::new(&referrer.pubkey(), &token.pubkey());
    let market_place_user = Some(&market_place_user);
    let referrer = Some(&referrer);
    //Some(&market_authority),
    //Some(&referrer),
    test_pass
        .init(
            &mut context,
            &user,
            &test_store.pubkey,
            &token.pubkey(),
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

    test_pass
        .buy(
            &mut context,
            &test_store,
            &buyer,
            &membership,
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
    let master_pass = test_pass.get_data(&mut context).await;
    let test_creator_payout = test_creator_payout.get_data(&mut context).await;
    let creator_token_holder_account =
        get_account(&mut context, &test_creator_payout.treasury_holder).await;
    let creator_token_holder =
        spl_token::state::Account::unpack_unchecked(&creator_token_holder_account.data).unwrap();

    let market_payout = test_market_place_payout.get_data(&mut context).await;
    let market_token_holder_account =
        get_account(&mut context, &market_payout.treasury_holder).await;
    let market_token_holder =
        spl_token::state::Account::unpack_unchecked(&market_token_holder_account.data).unwrap();

    let test_referrer_payout = test_referrer_payout.get_data(&mut context).await;
    let referrer_token_holder_account =
        get_account(&mut context, &test_referrer_payout.treasury_holder).await;
    let referrer_token_holder =
        spl_token::state::Account::unpack_unchecked(&referrer_token_holder_account.data).unwrap();

    let trade_history = trade_history.get_data(&mut context).await;
    assert_eq!(trade_history.already_bought, 1);

    assert_eq!(test_creator_payout.cash_in, 9750000);
    assert_eq!(creator_token_holder.amount, 9750000);

    assert_eq!(market_payout.cash_in, 125000);
    assert_eq!(market_token_holder.amount, 125000);

    assert_eq!(test_referrer_payout.cash_in, 125000);
    assert_eq!(referrer_token_holder.amount, 125000);
    assert_eq!(master_pass.supply, 1);
}

#[tokio::test]
async fn success_buy_with_native_token() {
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

    let (mut context, test_pass, test_store, trade_history, token, membership) =
        setup(&user, &buyer, 10_000_000, true).await;
    let test_creator_payout = TestPayout::new(&user.pubkey(), &token.pubkey());
    let test_market_place_payout = TestPayout::new(&market_place_user.pubkey(), &token.pubkey());
    let test_referrer_payout = TestPayout::new(&referrer.pubkey(), &token.pubkey());
    let market_place_user = Some(&market_place_user);
    let referrer = Some(&referrer);
    //Some(&market_authority),
    //Some(&referrer),
    test_pass
        .init(
            &mut context,
            &user,
            &test_store.pubkey,
            &token.pubkey(),
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

    let mut creator_payout = test_creator_payout.get_data(&mut context).await;

    let mut creator_token_holder_account =
        get_account(&mut context, &creator_payout.treasury_holder).await;
    let creator_initial_lamport = creator_token_holder_account.lamports;

    let mut market_payout = test_market_place_payout.get_data(&mut context).await;
    let mut market_token_holder_account =
        get_account(&mut context, &market_payout.treasury_holder).await;
    let market_place_initial_lamport = market_token_holder_account.lamports;

    let mut referrer_payout = test_referrer_payout.get_data(&mut context).await;
    let mut referrer_token_holder_account =
        get_account(&mut context, &referrer_payout.treasury_holder).await;
    let referrer_initial_lamport = referrer_token_holder_account.lamports;

    test_pass
        .buy(
            &mut context,
            &test_store,
            &buyer,
            &membership,
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
    let master_pass = test_pass.get_data(&mut context).await;
    creator_payout = test_creator_payout.get_data(&mut context).await;
    creator_token_holder_account = get_account(&mut context, &creator_payout.treasury_holder).await;

    market_payout = test_market_place_payout.get_data(&mut context).await;
    market_token_holder_account = get_account(&mut context, &market_payout.treasury_holder).await;

    referrer_payout = test_referrer_payout.get_data(&mut context).await;
    referrer_token_holder_account =
        get_account(&mut context, &referrer_payout.treasury_holder).await;

    let trade_history = trade_history.get_data(&mut context).await;
    assert_eq!(trade_history.already_bought, 1);

    assert_eq!(creator_payout.cash_in, 9750000);
    assert_eq!(
        creator_token_holder_account
            .lamports
            .checked_sub(creator_initial_lamport)
            .unwrap(),
        9750000
    );

    assert_eq!(market_payout.cash_in, 125000);
    assert_eq!(
        market_token_holder_account
            .lamports
            .checked_sub(market_place_initial_lamport)
            .unwrap(),
        125000
    );

    assert_eq!(referrer_payout.cash_in, 125000);
    assert_eq!(
        referrer_token_holder_account
            .lamports
            .checked_sub(referrer_initial_lamport)
            .unwrap(),
        125000
    );
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

    let (mut context, test_pass, test_store, trade_history, token, membership) =
        setup(&user, &buyer, 50_000_000, false).await;
    let market_place_user = Some(&market_place_user);
    let referrer = Some(&referrer);
    //Some(&market_authority),
    //Some(&referrer),
    test_pass
        .init(
            &mut context,
            &user,
            &test_store.pubkey,
            &token.pubkey(),
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

    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 1500).unwrap();

    test_pass
        .buy(
            &mut context,
            &test_store,
            &buyer,
            &membership,
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
    let clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    context.warp_to_slot(clock.slot + 1500).unwrap();

    let result = test_pass
        .buy(
            &mut context,
            &test_store,
            &buyer,
            &membership,
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
        NFTPassError::UserHasActiveMembership,
        0
    );
}
