use crate::*;
use mpl_token_metadata::{
    id, instruction,
    state::{Creator, Data, PREFIX},
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, transaction::Transaction,
};

#[derive(Debug)]
pub struct TestMetadata {
    pub mint: Keypair,
    pub pubkey: Pubkey,
    pub token: Keypair,
}

impl TestMetadata {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();
        let program_id = id();

        let metadata_seeds = &[PREFIX.as_bytes(), program_id.as_ref(), mint_pubkey.as_ref()];
        let (pubkey, _) = Pubkey::find_program_address(metadata_seeds, &id());

        Self {
            mint,
            pubkey,
            token: Keypair::new(),
        }
    }

    pub async fn get_data(
        &self,
        context: &mut ProgramTestContext,
    ) -> mpl_token_metadata::state::Metadata {
        let account = get_account(context, &self.pubkey).await;
        try_from_slice_unchecked(&account.data).unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        context: &mut ProgramTestContext,
        name: String,
        symbol: String,
        uri: String,
        authority: Option<&Keypair>,
        seller_fee_basis_points: u16,
        is_mutable: bool,
        token_authority: &Pubkey
    ) -> Result<(), BanksClientError> {
        create_mint(context, &self.mint, &context.payer.pubkey(), None).await?;
        create_token_account(context, &self.token, &self.mint.pubkey(), token_authority).await?;
        mint_tokens(
            context,
            &self.mint.pubkey(),
            &self.token.pubkey(),
            1,
            &context.payer.pubkey(),
            None,
        )
        .await?;
        let creators = if let Some(user) = authority {
            vec![
                Creator {
                    address: user.pubkey(),
                    verified: false,
                    share: 50,
                },
                Creator {
                    address: context.payer.pubkey(),
                    verified: false,
                    share: 50,
                },
            ]
        } else {
            vec![
                Creator {
                    address: context.payer.pubkey(),
                    verified: false,
                    share: 100,
                },
            ]
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction::create_metadata_accounts(
                id(),
                self.pubkey,
                self.mint.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                name,
                symbol,
                uri,
                Some(creators),
                seller_fee_basis_points,
                false,
                is_mutable,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        Ok(context.banks_client.process_transaction(tx).await?)
    }

    pub async fn update_primary_sale_happened_via_token(
        &self,
        context: &mut ProgramTestContext,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_primary_sale_happened_via_token(
                id(),
                self.pubkey,
                context.payer.pubkey(),
                self.token.pubkey(),
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        Ok(context.banks_client.process_transaction(tx).await?)
    }

    pub async fn update(
        &self,
        context: &mut ProgramTestContext,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_metadata_accounts(
                id(),
                self.pubkey,
                context.payer.pubkey(),
                None,
                Some(Data {
                    name,
                    symbol,
                    uri,
                    creators,
                    seller_fee_basis_points,
                }),
                None,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        Ok(context.banks_client.process_transaction(tx).await?)
    }
}
