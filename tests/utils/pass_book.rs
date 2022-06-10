use crate::*;
use mpl_token_metadata::state::Metadata;
use nft_pass_book::{
    find_pass_book_program_address, find_payout_program_address,
    instruction::{self, EditPassBookArgs},
    state::PassBook,
};
use solana_program::{
    program_pack::Pack, pubkey::Pubkey, system_instruction, system_program::ID as system_id,
};
use solana_program_test::*;

use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_token::state::Account;

#[derive(Debug)]
pub struct TestPassBook {
    pub pubkey: Pubkey,
    pub token_account: Keypair,
}

impl TestPassBook {
    #[allow(clippy::new_without_default)]
    pub fn new(mint: Pubkey) -> Self {
        let (pubkey, _) = find_pass_book_program_address(&nft_pass_book::id(), &mint);
        Self {
            pubkey,
            token_account: Keypair::new(),
        }
    }

    pub async fn activate(
        &self,
        context: &mut ProgramTestContext,
        user: &User,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::activate_pass_book(
                &nft_pass_book::id(),
                &self.pubkey,
                &user.owner.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&user.owner, &context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn deactivate(
        &self,
        context: &mut ProgramTestContext,
        user: &User,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::deactivate_pass_book(
                &nft_pass_book::id(),
                &self.pubkey,
                &user.owner.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&user.owner, &context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> PassBook {
        let account = get_account(context, &self.pubkey).await;
        PassBook::unpack_unchecked(&account.data).unwrap()
    }

    pub async fn edit(
        &self,
        context: &mut ProgramTestContext,
        user: &User,
        mutable: Option<bool>,
        name: Option<String>,
        description: Option<String>,
        uri: Option<String>,
        price: Option<u64>,
        blur_hash: Option<String>,
        price_mint: Option<&Pubkey>,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::edit_pass_book(
                &nft_pass_book::id(),
                &self.pubkey,
                &user.owner.pubkey(),
                price_mint,
                EditPassBookArgs {
                    name,
                    description,
                    uri,
                    blur_hash,
                    price,
                    mutable,
                },
            )],
            Some(&context.payer.pubkey()),
            &[&user.owner, &context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn delete(
        &self,
        context: &mut ProgramTestContext,
        user: &User,
        refunder: &Pubkey,
        token_acc: &Pubkey,
        mint: &Pubkey,
        new_master_edition_owner_token_acc: Option<&Pubkey>,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::delete_pass_book(
                &nft_pass_book::id(),
                &self.pubkey,
                &&user.owner.pubkey(),
                refunder,
                token_acc,
                mint,
                new_master_edition_owner_token_acc,
            )],
            Some(&context.payer.pubkey()),
            &[&user.owner, &context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn init(
        &self,
        context: &mut ProgramTestContext,
        test_master_edition: &TestMasterEditionV2,
        test_metadata: &TestMetadata,
        user: &User,
        store: &Pubkey,
        price_mint: &Pubkey,
        args: instruction::InitPassBookArgs,
    ) -> Result<(), BanksClientError> {
        let rent = context.banks_client.get_rent().await.unwrap();
        let metadata: Metadata = test_metadata.get_data(context).await;

        let payout = if let Some(creators) = metadata.data.creators {
            creators
                .iter()
                .map(|creator| {
                    find_payout_program_address(&nft_pass_book::id(), &creator.address, price_mint)
                        .0
                })
                .collect()
        } else {
            vec![]
        };

        let is_native = *price_mint == system_id;
        let (token_accounts, token_accounts_pub_key) = if is_native {
            (vec![], payout.clone())
        } else {
            let accounts: Vec<Keypair> = payout.iter().map(|_| Keypair::new()).collect();
            let account_keys: Vec<Pubkey> =
                accounts.iter().map(|account| account.pubkey()).collect();
            (accounts, account_keys)
        };
        let mut signers = vec![&context.payer, &self.token_account, &user.owner];
        for account in &token_accounts {
            signers.push(account);
        }
        let tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &self.token_account.pubkey(),
                    rent.minimum_balance(Account::LEN),
                    Account::LEN as u64,
                    &spl_token::id(),
                ),
                instruction::init_pass_book(
                    &nft_pass_book::id(),
                    &self.pubkey,
                    &user.token_account.pubkey(),
                    &self.token_account.pubkey(),
                    store,
                    &user.owner.pubkey(),
                    &context.payer.pubkey(),
                    &test_master_edition.mint_pubkey,
                    &test_metadata.pubkey,
                    &test_master_edition.pubkey,
                    price_mint,
                    None,
                    args.clone(),
                    &payout,
                    &token_accounts_pub_key,
                ),
            ],
            Some(&context.payer.pubkey()),
            &signers,
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
