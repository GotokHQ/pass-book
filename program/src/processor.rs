


use init_master_pass::init_master_pass;
use borsh::BorshDeserialize;
use crate::instruction::NFTPassInstruction;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub mod init_master_pass;

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
            NFTPassInstruction::InitMasterPass(args) => {
                msg!("Instruction: InitMasterPass");
                init_master_pass(program_id, accounts, args)
            }
        }
    }
}