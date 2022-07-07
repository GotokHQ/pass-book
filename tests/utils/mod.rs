mod assert;
mod edition;
mod edition_marker;
mod master_edition_v2;
mod membership;
mod metadata;
mod pass_book;
mod payout;
mod store;
mod token;
mod trade_history;
mod user;

pub use assert::*;
pub use edition_marker::TestEditionMarker;
pub use master_edition_v2::TestMasterEditionV2;
pub use membership::TestMembership;
pub use metadata::TestMetadata;
pub use pass_book::TestPassBook;
pub use payout::TestPayout;
pub use store::TestStore;
pub use token::TestSplToken;
pub use trade_history::TestTradeHistory;
pub use user::*;

use solana_program_test::*;
use solana_sdk::{
    account::Account, program_pack::Pack, pubkey::Pubkey, signature::Signer,
    signer::keypair::Keypair, system_instruction, transaction::Transaction,
};
use nft_pass_book::{
    instruction,
};
use spl_token::state::Mint;

pub fn nft_pass_book_program_test<'a>() -> ProgramTest {
    let mut program = ProgramTest::new("nft_pass_book", nft_pass_book::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program
}

pub async fn is_empty_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> bool {
    match context.banks_client.get_account(*pubkey).await {
        Ok(account) => account.is_none(),
        Err(_) => false,
    }
}

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn get_mint(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Mint {
    let account = get_account(context, pubkey).await;
    Mint::unpack(&account.data).unwrap()
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    additional_signers: Option<Vec<&Keypair>>,
) -> Result<(), BanksClientError> {
    let mut signing_keypairs = vec![&context.payer];
    if let Some(signers) = additional_signers {
        signing_keypairs.extend(signers)
    }

    let tx = Transaction::new_signed_with_payer(
        &[
            spl_token::instruction::mint_to(&spl_token::id(), mint, account, owner, &[], amount)
                .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &signing_keypairs,
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn transfer_token(
    context: &mut ProgramTestContext,
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Keypair,
    amount: u64,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::transfer(
            &spl_token::id(),
            source,
            destination,
            &authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_account<S: Pack>(
    context: &mut ProgramTestContext,
    account: &Keypair,
    owner: &Pubkey,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &context.payer.pubkey(),
            &account.pubkey(),
            rent.minimum_balance(S::LEN),
            S::LEN as u64,
            owner,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    manager: &Pubkey,
    freeze_authority: Option<&Pubkey>,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                manager,
                freeze_authority,
                0,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn set_up_pass_book_data(
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
    (
        context,
        test_pass,
        test_store,
        trade_history,
        token,
        membership,
    )
}

pub fn setup_users() -> (User, User, User, User) {
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
    let buyer = User {
        owner: Keypair::new(),
        token_account: Keypair::new(),
    };
    return (user, referrer, market_authority, buyer);
}

pub async fn setup_pass_book(mutable: bool) -> (ProgramTestContext, TestPassBook, User) {
    let (user, referrer, market, buyer) = setup_users();
    let (mut context, test_pass, test_store, _, token, _) =
    set_up_pass_book_data(&user, &buyer, 10_000_000, false).await;
    let market_place_user = Some(&market);
    let referrer = Some(&referrer);

    let name = String::from("Pass Name");
    let uri = String::from("some link to storage");
    let description = String::from("Pack description");

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
                mutable: mutable,
                duration: Some(30), //30 mins duration per session
                access: Some(30),   //valid for 30 days
                max_supply: Some(5),
                blur_hash: None,
                price: 10_000_000,
                has_referrer: referrer.is_some(), // Some(referrer.pubkey()),
                has_market_authority: market_place_user.is_some(),
                referral_end_date: None,
            },
        )
        .await
        .unwrap();

    (context, test_pass, user)
}