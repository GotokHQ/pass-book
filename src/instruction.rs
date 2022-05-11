//! Instruction types
#![allow(missing_docs)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Initialize a PackSet arguments
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Initialize a PackSet params
pub struct InitPassBookArgs {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// URI
    pub uri: String,
    /// If true authority can make changes at deactivated phase
    pub mutable: bool,
    /// The no of days this pass can be used to access the service
    pub access: Option<u64>,
    /// The no of minutes consumed for each use of this pass
    pub duration: Option<u64>,
    /// The maximum number of passes that can be printed
    pub max_supply: Option<u64>,
    /// blur hash of image
    pub blur_hash: Option<String>,
    /// price
    pub price: u64
}

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum NFTPassInstruction {
    /// InitPassBook
    ///
    /// Initialize created account.
    ///
    /// Accounts:
    ///   0.  `[writable]` Uninitialized master pass account with address as pda of (PDA ['pass', program id, master metadata mint id] )
    ///   1.  `[writer]` Source token account that holds MasterEdition token
    ///   2.  `[writer]` token_account (program account to hold MasterEdition token)
    ///   3.  `[writer]` PassStore account with pda of ['pass', program id, authority, 'store']
    ///   4.  `[signer]` Authority of pass account
    ///   5.  `[signer]` payer
    ///   6.  `[]` Master Metadata mint
    ///   7.  `[]` Master record metadata account
    ///   8.  `[]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   9.  `[]` System program
    ///   10. `[]` Rent info
    ///
    /// Parameters:
    /// - name	[u8; 32]
    /// - description String
    /// - URI String
    /// - mutable	bool
    /// - period    Period
    /// - max_num_uses    Option<u64>    InitPassBook()
    InitPassBook(InitPassBookArgs),
}

/// Create `InitPassBook` instruction
pub fn init_pass_book(
    program_id: &Pubkey,
    pass_book: &Pubkey,
    source: &Pubkey,
    token_account: &Pubkey,
    store: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    mint: &Pubkey,
    master_metadata: &Pubkey,
    master_edition: &Pubkey,
    price_mint: &Pubkey,
    gate_keeper: Option<&Pubkey>,
    args: InitPassBookArgs,
    payout_accounts: &[Pubkey],
    payout_token_accounts: &[Pubkey]
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*pass_book, false),
        AccountMeta::new(*source, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*store, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*payer, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(*master_metadata, false),
        AccountMeta::new_readonly(*master_edition, false),
        AccountMeta::new_readonly(*price_mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    for (i, payout) in payout_accounts.iter().enumerate() {
        accounts.push(AccountMeta::new(*payout, false));
        accounts.push(AccountMeta::new(payout_token_accounts[i], false))
    }

    if let Some(g_keeper) = gate_keeper {
        accounts.push(AccountMeta::new_readonly(*g_keeper, true))
    }

    Instruction::new_with_borsh(
        *program_id,
        &NFTPassInstruction::InitPassBook(args),
        accounts,
    )
}
