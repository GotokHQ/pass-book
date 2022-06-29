mod edition;
mod master_edition_v2;
mod pass_book;
mod metadata;
mod user;
mod assert;
mod token;

pub use master_edition_v2::TestMasterEditionV2;
pub use pass_book::TestPassBook;
pub use metadata::TestMetadata;
use solana_program::instruction::Instruction;
pub use token::TestSplToken;
pub use user::User;
pub use assert::*;

use solana_program_test::*;
use solana_sdk::{
    account::Account, program_pack::Pack, pubkey::Pubkey, signature::Signer,
    signer::keypair::Keypair, system_instruction, transaction::Transaction,
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
