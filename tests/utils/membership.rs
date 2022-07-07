use nft_pass_book::{state::Membership, find_membership_program_address};
use solana_program::program_pack::Pack;
use solana_program_test::ProgramTestContext;
use solana_sdk::pubkey::Pubkey;

use super::get_account;

#[derive(Debug)]
pub struct TestMembership {
    pub pubkey: Pubkey,
}

impl TestMembership {
    pub fn new(store: &Pubkey, wallet: &Pubkey) -> Self {
        let (pubkey, _) =
            find_membership_program_address(&nft_pass_book::id(), store, wallet);

        TestMembership { pubkey }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Membership {
        let account = get_account(context, &self.pubkey).await;
        Membership::unpack_unchecked(&account.data).unwrap()
    }
}
