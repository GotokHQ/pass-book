//! Instruction types
#![allow(missing_docs)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::state::PayoutInfoArgs;

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
    pub max_uses: Option<u64>,
    /// The maximum number of passes that can be printed
    pub max_supply: Option<u64>,
    /// price
    pub price: u64,
    /// Indicates the presence of a referral account in the account list
    pub has_referrer: bool,
    /// Indicates the presence of a market place authority in the account list
    pub has_market_authority: bool,
    /// The date after which referral rewards expires
    pub referral_end_date: Option<u64>,
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
    /// Price
    pub price: Option<u64>,
    /// If true authority can make changes at deactivated phase
    pub mutable: Option<bool>,
}

/// Buy Pass arguments
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct BuyPassArgs {
    /// The fee in basis point for the market place owner
    pub market_fee_basis_point: u16,
    /// The percentage of the amount from market_fee_basis_point to reward to the referral account
    pub referral_share: u8,
    /// The percentage of the referral_split to reward back to the referred account
    pub referral_kick_back_share: u8,
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
    ///   4.  `[]` Mint account of the tok en   
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
    ///   3.  `[writer]` Store account with pda of ['pass', program id, authority, 'store']
    ///   4.  `[signer]` Authority of pass account
    ///   5.  `[signer]` payer
    ///   6.  `[]` Master Metadata mint
    ///   7.  `[]` Master record metadata account
    ///   8.  `[]` Master Record Edition V2 (pda of ['metadata', program id, master metadata mint id, 'edition'])
    ///   9.  `[]` Price mint
    ///   10. `[]` Clock info
    ///   11. `[]` Rent info
    ///   12. `[]` System program
    ///   13. `[]` Token program
    ///
    /// Parameters:
    /// - name	String
    /// - description String
    /// - URI String
    /// - mutable	bool
    /// - period    Period
    /// - max_num_uses    Option<u64>    InitPassBook()
    InitPassBook(InitPassBookArgs),
    /// Buy Pass
    ///
    /// Buy a pass from a Pass Book.
    ///
    /// Accounts:
    ///   0.   `[writable]` Pass book account with address as pda of (PDA ['pass', program id, master metadata mint id] )
    ///   1.   `[writable]` The pass store account with address as pda of (PDA ['pass', program id, authority, 'store'] )
    ///   2.   `[writable]` Pass book token account vaulted that holds the master edition
    ///   3.   `[signer]`   The wallet of the user making the purchase
    ///   4.   `[writable]` Token account owned by user wallet used for transfer
    ///   5.   `[signer]`   The fee payer 
    ///   6.   `[writable]` New metadata account   || Will be created by mpl_token_metadata
    ///   7.   `[writable]` New edition account    || Will be created by mpl_token_metadata
    ///   8.   `[writable]` New mint account 
    ///   9.   `[writable]` Master edition account 
    ///   10.  `[writable]` Master metadata account 
    ///   11.  `[writable]` Edition marker account  || Will be created by `mpl_token_metadata
    ///   12.  `[writable]` New token account that holds limited edition
    ///   12.  `[writable]` Trade history 
    ///   13.  `[writable]` Creator payout info account []
    ///   14.  `[writable]` Creator payout token account []
    ///   15.  `[writable]` Creator payout ticket account [] || Will be created by nft_  pass
    ///   16.  `[signer]`   Market place authority
    ///   17.  `[writable]` Market place payout info account
    ///   18.  `[writable]` Market place payout token account
    ///   19.  `[writable]` Market place payout ticker account || Will be created by nft_  pass
    ///   20.  `[]`         Referral user wallet
    ///   21.  `[writable]` Referral payout info account
    ///   22.  `[writable]` Referral payout token account
    ///   23.  `[writable]` Referral payout ticket account || Will be created by nft_  pass
    ///   24.  `[]` Mint account of the token   
    ///   25.  `[]` SPL Token Program
    ///   26.  `[writable]` New master edition owner
    BuyPass(BuyPassArgs),

}

/// Create `ActivatePassBook` instruction
pub fn activate_pass_book(
    program_id: &Pubkey,
    passbook: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*passbook, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    Instruction::new_with_borsh(*program_id, &NFTPassInstruction::ActivatePassBook, accounts)
}

/// Create `DeactivatePassBook` instruction
pub fn deactivate_pass_book(
    program_id: &Pubkey,
    passbook: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*passbook, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    Instruction::new_with_borsh(*program_id, &NFTPassInstruction::DeactivatePassBook, accounts)
}

/// Create `DeletePassBook` instruction
pub fn delete_pass_book(
    program_id: &Pubkey,
    passbook: &Pubkey,
    authority: &Pubkey,
    refunder: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*passbook, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*refunder, false),
    ];
    Instruction::new_with_borsh(*program_id, &NFTPassInstruction::DeletePassBook, accounts)
}

/// Create `EditPassBook` instruction
pub fn edit_pass_book(
    program_id: &Pubkey,
    passbook: &Pubkey,
    authority: &Pubkey,
    mint: Option<&Pubkey>,
    args: EditPassBookArgs,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*passbook, false),
        AccountMeta::new_readonly(*authority, true),
    ];

    if let Some(new_price_mint) = mint {
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
    passbook: &Pubkey,
    store: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    mint: &Pubkey,
    creator_payout: &PayoutInfoArgs,
    market_payout: Option<&PayoutInfoArgs>,
    referral_payout: Option<&PayoutInfoArgs>,
    args: InitPassBookArgs,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*passbook, true),
        AccountMeta::new(*store, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*payer, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new(creator_payout.payout_account, false),
        AccountMeta::new(creator_payout.token_account, false),
    ];

    if let Some(market_place) = market_payout {
        accounts.push(AccountMeta::new_readonly(market_place.authority, true));
        accounts.push(AccountMeta::new(market_place.payout_account, false));
        accounts.push(AccountMeta::new(market_place.token_account, false))
    }

    if let Some(referral) = referral_payout {
        accounts.push(AccountMeta::new_readonly(referral.authority, false));
        accounts.push(AccountMeta::new(referral.payout_account, false));
        accounts.push(AccountMeta::new(referral.token_account, false))
    }
    Instruction::new_with_borsh(
        *program_id,
        &NFTPassInstruction::InitPassBook(args),
        accounts,
    )
}


/// Create `InitPassBook` instruction
pub fn buy_pass(
    program_id: &Pubkey,
    passbook: &Pubkey,
    store: &Pubkey,
    user_wallet: &Pubkey,
    user_token_account: &Pubkey,
    payer: &Pubkey,
    trade_history: &Pubkey,
    membership: &Pubkey,
    market_authority: Option<&PayoutInfoArgs>,
    referral_authority: Option<&PayoutInfoArgs>,
    creator_payout: &PayoutInfoArgs,
    args: BuyPassArgs,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*passbook, false),
        AccountMeta::new(*store, false),
        AccountMeta::new(*user_wallet, true),
        AccountMeta::new(*user_token_account, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(*trade_history, false),
        AccountMeta::new(*membership, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),    
        AccountMeta::new(creator_payout.payout_account, false),  
        AccountMeta::new(creator_payout.token_account, false),
    ];

    if let Some(market_place) = market_authority {
        accounts.push(AccountMeta::new_readonly(market_place.authority, true));
        accounts.push(AccountMeta::new(market_place.payout_account, false));
        accounts.push(AccountMeta::new(market_place.token_account, false))
    }

    if let Some(referral) = referral_authority {
        accounts.push(AccountMeta::new_readonly(referral.authority, false));
        accounts.push(AccountMeta::new(referral.payout_account, false));
        accounts.push(AccountMeta::new(referral.token_account, false))
    }

    accounts.push(AccountMeta::new_readonly(spl_token::id(), false));
    Instruction::new_with_borsh(
        *program_id,
        &NFTPassInstruction::BuyPass(args),
        accounts,
    )
}