use nft_pass_book::{find_trade_history_program_address, state::TradeHistory};
use solana_program::program_pack::Pack;
use solana_program_test::ProgramTestContext;
use solana_sdk::pubkey::Pubkey;

use super::get_account;

#[derive(Debug)]
pub struct TestTradeHistory {
    pub pubkey: Pubkey,
}

impl TestTradeHistory {
    pub fn new(passbook: &Pubkey, wallet: &Pubkey) -> Self {
        let (pubkey, _) =
            find_trade_history_program_address(&nft_pass_book::id(), passbook, wallet);

        TestTradeHistory { pubkey }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> TradeHistory {
        let account = get_account(context, &self.pubkey).await;
        TradeHistory::unpack_unchecked(&account.data).unwrap()
    }
}
