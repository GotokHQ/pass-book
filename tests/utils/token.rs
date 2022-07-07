use crate::*;

use solana_program::system_instruction;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, transaction::Transaction,
};
#[derive(Debug)]
pub struct TestSplToken {
    pub mint: Keypair,
    pub is_native: bool,
}

impl TestSplToken {
    #[allow(clippy::new_without_default)]
    pub fn new(is_native: bool) -> Self {
        let mint = Keypair::new();
        Self { mint,  is_native}
    }

    pub fn pubkey(&self) -> Pubkey {
        if self.is_native {
            spl_token::native_mint::id()
        } else {
            self.mint.pubkey()
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        context: &mut ProgramTestContext,
        amount: u64,
        token_account: &Keypair,
        token_owner: &Pubkey,
    ) -> Result<(), BanksClientError> {
        create_mint(context, &self.mint, &context.payer.pubkey(), None).await?;
        self.mint_to(context, amount, token_account, token_owner)
            .await
    }

    pub async fn mint_to(
        &self,
        context: &mut ProgramTestContext,
        amount: u64,
        token_account: &Keypair,
        token_owner: &Pubkey,
    ) -> Result<(), BanksClientError> {
        create_token_account(context, token_account, &self.mint.pubkey(), token_owner).await?;
        mint_tokens(
            context,
            &self.mint.pubkey(),
            &token_account.pubkey(),
            amount,
            &context.payer.pubkey(),
            None,
        )
        .await
    }

    pub async fn airdrop(
        &self,
        context: &mut ProgramTestContext,
        amount: u64,
        receiver: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::transfer(
                &context.payer.pubkey(),
                receiver,
                amount,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
