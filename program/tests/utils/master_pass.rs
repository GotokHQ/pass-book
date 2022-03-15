use crate::*;
use gtk_nft_pass::{
    instruction::{self},
    state::MasterPass,
    find_master_pass_program_address,
};
use solana_program::{
    program_pack::Pack, pubkey::Pubkey, system_instruction,
};
use solana_program_test::*;

use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction},
    transport,
};
use spl_token::state::Account;

#[derive(Debug)]
pub struct TestMasterPass {
    pub pubkey: Pubkey,
    pub token_account: Keypair,
    pub store: Pubkey,
}

impl TestMasterPass {
    #[allow(clippy::new_without_default)]
    pub fn new(store: Pubkey, mint: Pubkey) -> Self {
        let (pubkey, _) = find_master_pass_program_address(&gtk_nft_pass::id(), &mint);
        Self {
            pubkey,
            token_account: Keypair::new(),
            store,
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> MasterPass {
        let account = get_account(context, &self.pubkey).await;
        MasterPass::unpack_unchecked(&account.data).unwrap()
    }


    pub async fn init(
        &self,
        context: &mut ProgramTestContext,
        test_master_edition: &TestMasterEditionV2,
        test_metadata: &TestMetadata,
        user: &User,
        args: instruction::InitMasterPassArgs,
    ) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &self.token_account.pubkey(),
                    rent.minimum_balance(Account::LEN),
                    Account::LEN as u64,
                    &spl_token::id(),
                ),
                instruction::init_master_pass(
                    &gtk_nft_pass::id(),
                    &self.pubkey,
                    &user.token_account.pubkey(),
                    &self.token_account.pubkey(),
                    &self.store,
                    &user.owner.pubkey(),
                    &context.payer.pubkey(),
                    &test_master_edition.mint_pubkey,
                    &test_metadata.pubkey,
                    &test_master_edition.pubkey,
                    args.clone()
                ),
            ],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &self.token_account,
                &user.owner,
            ],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}