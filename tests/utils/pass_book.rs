use crate::*;
use mpl_token_metadata::state::Metadata;
use nft_pass_book::{
    find_pass_book_program_address, find_payout_program_address,
    instruction::{self, EditPassBookArgs},
    state::{PassBook, PassStore, PayoutInfoArgs},
    utils::cmp_pubkeys,
};
use solana_program::{instruction::Instruction, program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;

use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
#[derive(Debug)]
pub struct TestPassBook {
    pub pubkey: Pubkey,
    pub token_account: Pubkey,
}

impl TestPassBook {
    #[allow(clippy::new_without_default)]
    pub fn new(mint: Pubkey) -> Self {
        let (pubkey, _) = find_pass_book_program_address(&nft_pass_book::id(), &mint);
        Self {
            pubkey,
            token_account: get_associated_token_address(&pubkey, &mint),
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
        mint: &Pubkey,
        new_master_edition_owner_token_acc: Option<&Pubkey>,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::delete_pass_book(
                &nft_pass_book::id(),
                &self.pubkey,
                &user.owner.pubkey(),
                refunder,
                &self.token_account,
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
        referrer: Option<&User>,
        args: instruction::InitPassBookArgs,
    ) -> Result<(), BanksClientError> {
        let metadata: Metadata = test_metadata.get_data(context).await;
        let mut instructions: Vec<Instruction> = vec![];
        let mut signers = vec![&context.payer, &user.owner];
        let is_native = cmp_pubkeys(price_mint, &spl_token::native_mint::id());
        let creator_payout = if let Some(creators) = metadata.data.creators {
            creators
                .iter()
                .map(|creator| {
                    let creator_payout = find_payout_program_address(
                        &nft_pass_book::id(),
                        &creator.address,
                        price_mint,
                    )
                    .0;
                    let token_account = if is_native {
                        creator_payout
                    } else {
                        instructions.push(create_associated_token_account(
                            &context.payer.pubkey(),
                            &creator_payout,
                            price_mint,
                        ));
                        get_associated_token_address(&creator_payout, price_mint)
                    };
                    PayoutInfoArgs {
                        authority: creator.address,
                        payout_account: creator_payout,
                        token_account: token_account,
                    }
                })
                .collect()
        } else {
            vec![]
        };
        let market_authority = if let Some(market_info) = market {
            signers.push(&market_info.owner);
            let payout = find_payout_program_address(
                &nft_pass_book::id(),
                &market_info.pubkey(),
                price_mint,
            )
            .0;
            let token_account = if is_native {
                payout
            } else {
                instructions.push(create_associated_token_account(
                    &context.payer.pubkey(),
                    &payout,
                    price_mint,
                ));
                get_associated_token_address(&payout, price_mint)
            };
            let market_auth = PayoutInfoArgs {
                authority: market_info.pubkey(),
                payout_account: payout,
                token_account: token_account,
            };
            Some(market_auth)
        } else {
            None
        };

        let referrer = if let Some(referrer_user) = referrer {
            let payout = find_payout_program_address(
                &nft_pass_book::id(),
                &referrer_user.pubkey(),
                price_mint,
            )
            .0;
            let token_account = if is_native {
                payout
            } else {
                instructions.push(create_associated_token_account(
                    &context.payer.pubkey(),
                    &payout,
                    price_mint,
                ));
                get_associated_token_address(&payout, price_mint)
            };
            let referrer = PayoutInfoArgs {
                authority: referrer_user.pubkey(),
                payout_account: payout,
                token_account: token_account,
            };
            Some(referrer)
        } else {
            None
        };
        instructions.push(create_associated_token_account(
            &context.payer.pubkey(),
            &self.pubkey,
            &test_metadata.mint.pubkey(),
        ));
        instructions.push(instruction::init_pass_book(
            &nft_pass_book::id(),
            &self.pubkey,
            &test_metadata.token.pubkey(),
            &self.token_account,
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
        ));
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&context.payer.pubkey()),
            &signers,
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await
    }

    pub async fn buy(
        &self,
        context: &mut ProgramTestContext,
        metadata: &TestMetadata,
        edition_marker: &TestEditionMarker,
        store: &TestStore,
        user: &User,
        market: Option<&User>,
        trade_history: &TestTradeHistory,
        args: instruction::BuyPassArgs,
    ) -> Result<(), BanksClientError> {
        let metadata: Metadata = metadata.get_data(context).await;
        let passbook: PassBook = self.get_data(context).await;
        let pass_store: PassStore = store.get_data(context).await;
        let mut signers = vec![&context.payer, &user.owner];
        let is_native = cmp_pubkeys(&passbook.price_mint, &spl_token::native_mint::id());
        let creator_payout = if let Some(creators) = metadata.data.creators {
            creators
                .iter()
                .map(|creator| {
                    let creator_payout = find_payout_program_address(
                        &nft_pass_book::id(),
                        &creator.address,
                        &passbook.price_mint,
                    )
                    .0;
                    let token_account = if is_native {
                        creator_payout
                    } else {
                        get_associated_token_address(&creator_payout, &passbook.price_mint)
                    };
                    PayoutInfoArgs {
                        authority: creator.address,
                        payout_account: creator_payout,
                        token_account: token_account,
                    }
                })
                .collect()
        } else {
            vec![]
        };
        let market_authority = if let Some(market_info) = market {
            signers.push(&market_info.owner);
            let payout = find_payout_program_address(
                &nft_pass_book::id(),
                &market_info.pubkey(),
                &passbook.price_mint,
            )
            .0;
            let token_account = if is_native {
                payout
            } else {
                get_associated_token_address(&payout, &passbook.price_mint)
            };
            let market_auth = PayoutInfoArgs {
                authority: market_info.pubkey(),
                payout_account: payout,
                token_account: token_account,
            };
            Some(market_auth)
        } else {
            None
        };

        let referrer = if let Some(referrer_user) = pass_store.referrer {
            let payout = find_payout_program_address(
                &nft_pass_book::id(),
                &referrer_user,
                &passbook.price_mint,
            )
            .0;
            let token_account = if is_native {
                payout
            } else {
                get_associated_token_address(&payout, &passbook.price_mint)
            };
            let referrer = PayoutInfoArgs {
                authority: referrer_user,
                payout_account: payout,
                token_account: token_account,
            };
            Some(referrer)
        } else {
            None
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction::buy_pass(
                &nft_pass_book::id(),
                &self.pubkey,
                &store.pubkey,
                &self.token_account,
                &user.owner.pubkey(),
                &user.token_account.pubkey(),
                &context.payer.pubkey(),
                &edition_marker.new_metadata_pubkey,
                &edition_marker.new_edition_pubkey,
                &edition_marker.mint.pubkey(),
                &edition_marker.metadata_pubkey,
                &edition_marker.master_edition_pubkey,
                &edition_marker.pubkey,
                &edition_marker.token.pubkey(),
                &trade_history.pubkey,
                market_authority.as_ref(),
                referrer.as_ref(),
                args.clone(),
                &creator_payout,
            )],
            Some(&context.payer.pubkey()),
            &signers,
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
