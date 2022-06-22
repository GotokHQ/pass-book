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
    pub price: u64,
}

/// Edit a PassBook arguments
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct EditPassBookArgs {
    /// Name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// URI
    pub uri: Option<String>,
    /// Blurhash
    pub blur_hash: Option<String>,
    /// Price
    pub price: Option<u64>,
    /// If true authority can make changes at deactivated phase
    pub mutable: Option<bool>,
}

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum NFTPassInstruction {
    /// ActivatePassBook
    ///
    /// Set pack state to "Activated"
    ///
    /// Accounts:
    ///   0.  `[writable]` Pass book account with address as pda of (PDA ['pass', program id, master metadata mint id] )
    ///   1.  `[signer]` Authority of pass book account
    ActivatePassBook,
    /// DeletePass Book
    ///
    /// Transfer all the SOL from pass book account to refunder account and thus remove it.
    ///
    /// Accounts:
    ///   0.  `[writable]` Pass book account with address as pda of (PDA ['pass', program id, master metadata mint id] )
    ///   1.  `[signer]` Authority of pass book account
    ///   2.  `[writable]` Refunder
    ///   3.  `[writable]` Token account owned by pass book that holds the master edition
    ///   4.  `[]` Mint account of the token   
    ///   5.  `[]` SPL Token Program
    ///   6.  `[writable]` New master edition owner
    DeletePassBook,
    /// DeactivatePassBook
    ///
    /// Set pack state to "Deactivated"
    ///
    /// Accounts:
    ///   0.  `[writable]` Pass book account with address as pda of (PDA ['pass', program id, master metadata mint id] )
    ///   1.  `[signer]` Authority of pass book account
    DeactivatePassBook,
    /// EditPassBook
    ///
    /// Edit pass book.
    ///
    /// Accounts:
    ///   0.  `[writable]` Pass book account with address as pda of (PDA ['pass', program id, master metadata mint id] )
    ///   1.  `[signer]` Authority of pass book account
    ///
    /// Parameters:
    /// - name Option<String>
    /// - description Option<String>
    /// - URI Option<String>
    /// - mutable	Option<bool> (only can be changed from true to false)
    EditPassBook(EditPassBookArgs),
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
    /// - name	String
    /// - description String
    /// - URI String
    /// - mutable	bool
    /// - period    Period
    /// - max_num_uses    Option<u64>    InitPassBook()
    InitPassBook(InitPassBookArgs),
}

/// Create `ActivatePassBook` instruction
pub fn activate_pass_book(
    program_id: &Pubkey,
    pass_book: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*pass_book, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    Instruction::new_with_borsh(*program_id, &NFTPassInstruction::ActivatePassBook, accounts)
}

/// Create `DeactivatePassBook` instruction
pub fn deactivate_pass_book(
    program_id: &Pubkey,
    pass_book: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*pass_book, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    Instruction::new_with_borsh(*program_id, &NFTPassInstruction::DeactivatePassBook, accounts)
}

/// Create `DeletePassBook` instruction
pub fn delete_pass_book(
    program_id: &Pubkey,
    pass_book: &Pubkey,
    authority: &Pubkey,
    refunder: &Pubkey,
    token_account: &Pubkey,
    mint_account: &Pubkey,
    new_master_edition_owner: Option<&Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*pass_book, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*refunder, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*mint_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    if let Some(new_master_edition) = new_master_edition_owner {
        accounts.push(AccountMeta::new(*new_master_edition, false))
    }
    Instruction::new_with_borsh(*program_id, &NFTPassInstruction::DeletePassBook, accounts)
}

/// Create `EditPassBook` instruction
pub fn edit_pass_book(
    program_id: &Pubkey,
    pass_book: &Pubkey,
    authority: &Pubkey,
    price_mint: Option<&Pubkey>,
    args: EditPassBookArgs,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*pass_book, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    if let Some(new_price_mint) = price_mint {
        accounts.push(AccountMeta::new_readonly(*new_price_mint, false))
    }

    Instruction::new_with_borsh(
        *program_id,
        &NFTPassInstruction::EditPassBook(args),
        accounts,
    )
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
    payout_token_accounts: &[Pubkey],
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*pass_book, false),
        AccountMeta::new(*source, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*store, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*payer, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(*master_metadata, false),
        AccountMeta::new_readonly(*master_edition, false),
        AccountMeta::new_readonly(*price_mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(mpl_token_metadata::id(), false),
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