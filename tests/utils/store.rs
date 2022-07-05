use nft_pass_book::{find_pass_store_program_address, state::PassStore};
use solana_program::program_pack::Pack;
use solana_program_test::ProgramTestContext;
use solana_sdk::pubkey::Pubkey;

use super::get_account;

#[derive(Debug)]
pub struct TestStore {
    pub pubkey: Pubkey,
}

impl TestStore {
    pub fn new(authority: &Pubkey) -> Self {
        let (pubkey, _) = find_pass_store_program_address(&nft_pass_book::id(), authority);

        TestStore { pubkey }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> PassStore {
        let account = get_account(context, &self.pubkey).await;
        PassStore::unpack_unchecked(&account.data).unwrap()
    }
}
