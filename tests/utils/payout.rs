use nft_pass_book::{find_payout_program_address, state::Payout};
use solana_program::program_pack::Pack;
use solana_program_test::ProgramTestContext;
use solana_sdk::pubkey::Pubkey;

use super::get_account;

#[derive(Debug)]
pub struct TestPayout {
    pub pubkey: Pubkey,
}

impl TestPayout {
    pub fn new(authority: &Pubkey, mint: &Pubkey) -> Self {
        let (pubkey, _) = find_payout_program_address(&nft_pass_book::id(), authority, mint);

        TestPayout { pubkey }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Payout {
        let account = get_account(context, &self.pubkey).await;
        Payout::unpack_unchecked(&account.data).unwrap()
    }
}
