


use init_pass_book::init_pass_book;
use edit_pass_book::edit_pass_book;
use delete_pass_book::delete_pass_book;
use activate_pass_book::activate_pass_book;
use deactivate_pass_book::deactivate_pass_book;
use buy_pass_book::buy;

use borsh::BorshDeserialize;
use crate::instruction::NFTPassInstruction;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub mod init_pass_book;
pub mod edit_pass_book;
pub mod delete_pass_book;
pub mod activate_pass_book;
pub mod deactivate_pass_book;
pub mod buy_pass_book;

pub struct Processor {}

impl Processor {
    /// Processes an instruction
    pub fn process_instruction<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        input: &[u8],
    ) -> ProgramResult {
        msg!("START PROCESSING INSTRUCTION");
        let instruction = NFTPassInstruction::try_from_slice(input)?;
        match instruction {
            NFTPassInstruction::ActivatePassBook => {
                msg!("Instruction: ActivatePassBook");
                activate_pass_book(program_id, accounts)
            },
            NFTPassInstruction::DeactivatePassBook => {
                msg!("Instruction: DeactivatePassBook");
                deactivate_pass_book(program_id, accounts)
            },
            NFTPassInstruction::DeletePassBook => {
                msg!("Instruction: DeletePassBook");
                delete_pass_book(program_id, accounts)
            },
            NFTPassInstruction::EditPassBook(args) => {
                msg!("Instruction: EditPassBook");
                edit_pass_book(program_id, accounts, args)
            },
            NFTPassInstruction::InitPassBook(args) => {
                msg!("Instruction: InitPassBook");
                init_pass_book(program_id, accounts, args)
            },
            NFTPassInstruction::BuyPass(args) => {
                msg!("Instruction: BuyPass");
                buy(program_id, accounts, args)
            },
            _ => Ok(())
        }
    }
}