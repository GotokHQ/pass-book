//! Instruction types
#![allow(missing_docs)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
    system_program, sysvar,
};

use crate::state::Period;


/// Initialize a PackSet arguments
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Initialize a PackSet params
pub struct InitMasterPassArgs {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// URI
    pub uri: String,
    /// If true authority can make changes at deactivated phase
    pub mutable: bool,  
    /// Period type
    pub period: Period,
}

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum NFTPassInstruction {
    /// InitMasterPass
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
    ///   9.  `[]` System program
    ///   10. `[]` Rent info
    ///
    /// Parameters:
    /// - name	[u8; 32]
    /// - description String
    /// - URI String
    /// - mutable	bool
    /// - period    Period
    /// - max_num_uses    Option<u64>    InitMasterPass()
    InitMasterPass(InitMasterPassArgs),
}

/// Create `InitMasterPass` instruction
pub fn init_master_pass(
    program_id: &Pubkey,
    master_pass: &Pubkey,
    source: &Pubkey,
    token_account: &Pubkey,
    store: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    mint: &Pubkey,
    master_metadata: &Pubkey,
    master_edition: &Pubkey,
    args: InitMasterPassArgs,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*master_pass, false),
        AccountMeta::new(*source, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*store, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*payer, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(*master_metadata, false),
        AccountMeta::new_readonly(*master_edition, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction::new_with_borsh(*program_id, &NFTPassInstruction::InitMasterPass(args), accounts)
}