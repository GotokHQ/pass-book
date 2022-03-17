


use init_pass_book::init_pass_book;
use borsh::BorshDeserialize;
use crate::instruction::NFTPassInstruction;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub mod init_pass_book;

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
            NFTPassInstruction::InitPassBook(args) => {
                msg!("Instruction: InitPassBook");
                init_pass_book(program_id, accounts, args)
            }
        }
    }
}