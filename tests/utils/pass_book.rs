use crate::*;
use nft_pass_book::{
    find_payout_program_address,
    instruction::{self, EditPassBookArgs},
    state::{PassBook, PayoutInfoArgs, Store},
    utils::cmp_pubkeys,
};
use solana_program::{
    instruction::Instruction, program_pack::Pack, pubkey::Pubkey,
};
use solana_program_test::*;

use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
#[derive(Debug)]
pub struct TestPassBook {
    pub account: Keypair,
}

impl TestPassBook {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            account: Keypair::new(),
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
                &self.account.pubkey(),
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
                &self.account.pubkey(),
                &user.owner.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&user.owner, &context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> PassBook {
        let account = get_account(context, &self.account.pubkey()).await;
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
        mint: Option<&Pubkey>,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::edit_pass_book(
                &nft_pass_book::id(),
                &self.account.pubkey(),
                &user.owner.pubkey(),
                mint,
                EditPassBookArgs {
                    name,
                    description,
                    uri,
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
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::delete_pass_book(
                &nft_pass_book::id(),
                &self.account.pubkey(),
                &user.pubkey(),
                &refunder,
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
        user: &User,
        store: &Pubkey,
        mint: &Pubkey,
        market: Option<&User>,
        referrer: Option<&User>,
        args: instruction::InitPassBookArgs,
    ) -> Result<(), BanksClientError> {
        let mut instructions: Vec<Instruction> = vec![];
        let mut signers = vec![&context.payer, &user.owner, &self.account];
        let is_native = cmp_pubkeys(mint, &spl_token::native_mint::id());
        let creator_payout_key =
            find_payout_program_address(&nft_pass_book::id(), &user.pubkey(), mint).0;
        let creator_token_account = if is_native {
            creator_payout_key
        } else {
            instructions.push(create_associated_token_account(
                &context.payer.pubkey(),
                &creator_payout_key,
                mint,
            ));
            get_associated_token_address(&creator_payout_key, mint)
        };
        let creator_payout = PayoutInfoArgs {
            authority: user.pubkey(),
            payout_account: creator_payout_key,
            token_account: creator_token_account,
        };
        let market_authority = if let Some(market_info) = market {
            signers.push(&market_info.owner);
            let payout =
                find_payout_program_address(&nft_pass_book::id(), &market_info.pubkey(), mint).0;
            let token_account = if is_native {
                payout
            } else {
                instructions.push(create_associated_token_account(
                    &context.payer.pubkey(),
                    &payout,
                    mint,
                ));
                get_associated_token_address(&payout, mint)
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
            let payout =
                find_payout_program_address(&nft_pass_book::id(), &referrer_user.pubkey(), mint).0;
            let token_account = if is_native {
                payout
            } else {
                instructions.push(create_associated_token_account(
                    &context.payer.pubkey(),
                    &payout,
                    mint,
                ));
                get_associated_token_address(&payout, mint)
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
        // let rent = context.banks_client.get_rent().await?;
        // instructions.push(system_instruction::create_account(
        //     &context.payer.pubkey(),
        //     &self.account.pubkey(),
        //     rent.minimum_balance(PassBook::LEN),
        //     PassBook::LEN as u64,
        //     &nft_pass_book::id(),
        // ));
        instructions.push(instruction::init_pass_book(
            &nft_pass_book::id(),
            &self.account.pubkey(),
            store,
            &user.pubkey(),
            &context.payer.pubkey(),
            mint,
            &creator_payout,
            market_authority.as_ref(),
            referrer.as_ref(),
            args.clone(),
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
        store: &TestStore,
        buyer: &User,
        membership: &TestMembership,
        market: Option<&User>,
        trade_history: &TestTradeHistory,
        args: instruction::BuyPassArgs,
    ) -> Result<(), BanksClientError> {
        let passbook: PassBook = self.get_data(context).await;
        let pass_store: Store = store.get_data(context).await;
        let mut signers = vec![&context.payer, &buyer.owner];
        let is_native = cmp_pubkeys(&passbook.mint, &spl_token::native_mint::id());
        let creator_payout_key =
            find_payout_program_address(&nft_pass_book::id(), &passbook.authority, &passbook.mint)
                .0;
        let token_account = if is_native {
            creator_payout_key
        } else {
            get_associated_token_address(&creator_payout_key, &passbook.mint)
        };
        let creator_payout = PayoutInfoArgs {
            authority: passbook.authority,
            payout_account: creator_payout_key,
            token_account: token_account,
        };
        let market_authority = if let Some(market_info) = market {
            signers.push(&market_info.owner);
            let payout = find_payout_program_address(
                &nft_pass_book::id(),
                &market_info.pubkey(),
                &passbook.mint,
            )
            .0;
            let token_account = if is_native {
                payout
            } else {
                get_associated_token_address(&payout, &passbook.mint)
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
            let payout =
                find_payout_program_address(&nft_pass_book::id(), &referrer_user, &passbook.mint).0;
            let token_account = if is_native {
                payout
            } else {
                get_associated_token_address(&payout, &passbook.mint)
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

        let buyer_token = if is_native {
            buyer.pubkey()
        } else {
            buyer.token_account.pubkey()
        };
        println!("START PROCESSING BUY INSTRUCTION");
        let tx = Transaction::new_signed_with_payer(
            &[instruction::buy_pass(
                &nft_pass_book::id(),
                &self.account.pubkey(),
                &store.pubkey,
                &buyer.pubkey(),
                &buyer_token,
                &context.payer.pubkey(),
                &trade_history.pubkey,
                &membership.pubkey,
                market_authority.as_ref(),
                referrer.as_ref(),
                &creator_payout,
                args.clone(),
            )],
            Some(&context.payer.pubkey()),
            &signers,
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
