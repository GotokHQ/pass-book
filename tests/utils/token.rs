use crate::*;

use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair
};

#[derive(Debug)]
pub struct TestSplToken {
    pub mint: Keypair,
}

impl TestSplToken {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mint = Keypair::new();
        Self {
            mint,
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
}
