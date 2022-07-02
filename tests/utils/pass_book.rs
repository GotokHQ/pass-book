use crate::*;
use mpl_token_metadata::state::Metadata;
use nft_pass_book::{
    find_pass_book_program_address, find_payout_program_address,
    instruction::{self, EditPassBookArgs},
    state::{PassBook, PayoutInfoArgs},
    utils::cmp_pubkeys
};
use solana_program::{
    program_pack::Pack, pubkey::Pubkey, system_instruction, instruction::Instruction,
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
        market: Option<&User>,
        referrer:Option<&User>, 
        args: instruction::InitPassBookArgs,
    ) -> Result<(), BanksClientError> {
        let rent = context.banks_client.get_rent().await.unwrap();
        let metadata: Metadata = test_metadata.get_data(context).await;

        let mut instructions: Vec<Instruction> = vec![];
        let mut all_signers = vec![&context.payer, &self.token_account, &user.owner];
        let mut new_signers:Vec<Keypair> = vec![];
        let is_native =  cmp_pubkeys(price_mint, &spl_token::native_mint::id());
        let creator_payout = if let Some(creators) = metadata.data.creators {
            creators
                .iter()
                .map(|creator| {
                    let creator_payout = find_payout_program_address(&nft_pass_book::id(), &creator.address, price_mint)
                        .0;
                    let token_account = if is_native {
                        creator_payout
                    } else {
                        let account = Keypair::new();
                        instructions.push(
                            system_instruction::create_account(
                                &context.payer.pubkey(),
                                &account.pubkey(),
                                rent.minimum_balance(Account::LEN),
                                Account::LEN as u64,
                                &spl_token::id(),
                            ),
                        );
                        let pub_key = account.pubkey();
                        new_signers.push(account);
                        pub_key
                    };
                    PayoutInfoArgs{
                        authority: creator.address,
                        payout_account: creator_payout,
                        token_account: token_account,
                        share: creator.share
                    }
                })
                .collect()
        } else {
            vec![]
        };
        for signer in new_signers.iter() {
            all_signers.push(signer)
        }
        let market_authority = if let Some(market_info) = market {
            all_signers.push(&market_info.owner);
            let payout = find_payout_program_address(&nft_pass_book::id(), &market_info.pubkey(), price_mint)
            .0;
            let token_account = if is_native {
                payout
            } else {
                instructions.push(
                    system_instruction::create_account(
                        &context.payer.pubkey(),
                        &market_info.token_account.pubkey(),
                        rent.minimum_balance(Account::LEN),
                        Account::LEN as u64,
                        &spl_token::id(),
                    ),
                );
                all_signers.push(&market_info.token_account);
                market_info.token_account.pubkey()
            };
            let market_auth = PayoutInfoArgs{
                authority: market_info.pubkey(),
                payout_account: payout,
                token_account: token_account,
                share: 100
            };
            Some(market_auth)
        } else {
            None
        };
    
        let referrer = if let Some(referrer_user) = referrer {
            let payout = find_payout_program_address(&nft_pass_book::id(), &referrer_user.pubkey(), price_mint)
            .0;
            let token_account = if is_native {
                payout
            } else {
                instructions.push(
                    system_instruction::create_account(
                        &context.payer.pubkey(),
                        &referrer_user.token_account.pubkey(),
                        rent.minimum_balance(Account::LEN),
                        Account::LEN as u64,
                        &spl_token::id(),
                    ),
                );
                all_signers.push(&referrer_user.token_account);
                referrer_user.token_account.pubkey()
            };
            let referrer = PayoutInfoArgs{
                authority: referrer_user.pubkey(),
                payout_account: payout,
                token_account: token_account,
                share: 40
            };
            Some(referrer)
        } else {
            None
        };

        instructions.push(
            system_instruction::create_account(
                &context.payer.pubkey(),
                &self.token_account.pubkey(),
                rent.minimum_balance(Account::LEN),
                Account::LEN as u64,
                &spl_token::id(),
            ),
        );
        instructions.push(
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
                market_authority.as_ref(),
                referrer.as_ref(),
                args.clone(),
                &creator_payout,
            ),
        );
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&context.payer.pubkey()),
            &all_signers,
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
