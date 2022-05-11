//! Program utils


use crate::{
    error::NFTPassError,
    find_program_authority,
    find_pass_collection_mint,
    state::{PREFIX, PassStore},
};
 
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use spl_associated_token_account::{create_associated_token_account};

/// Assert uninitialized
pub fn assert_uninitialized<T: IsInitialized>(account: &T) -> ProgramResult {
    if account.is_initialized() {
        Err(ProgramError::AccountAlreadyInitialized)
    } else {
        Ok(())
    }
}

/// Assert signer
pub fn assert_signer(account: &AccountInfo) -> ProgramResult {
    if account.is_signer {
        return Ok(());
    }

    Err(ProgramError::MissingRequiredSignature)
}

/// Assert owned by
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        Err(ProgramError::IllegalOwner)
    } else {
        Ok(())
    }
}

/// Assert account key
pub fn assert_account_key(account_info: &AccountInfo, key: &Pubkey, error: Option<NFTPassError>) -> ProgramResult {
    if *account_info.key != *key {
        match error {
            Some(e) => Err(e.into()),
            _ => Err(ProgramError::InvalidArgument)
        }
    } else {
        Ok(())
    }
}

/// Assert account rent exempt
pub fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        Err(ProgramError::AccountNotRentExempt)
    } else {
        Ok(())
    }
}

// pub fn assert_pass_authority_derivation(
//     program_id: &Pubkey,
//     pass_info: &AccountInfo,
//     authority: &AccountInfo,
//     name: &str,
// ) -> Result<u8, ProgramError> {
//     let pass_seeds = [PREFIX.as_bytes(), program_id.as_ref(), name.as_bytes()];
//     let bump = assert_derivation(&program_id, pass_info, &pass_seeds)?;
//     let pass = Pass::unpack_from_slice(&pass_info.data.borrow())?;
//     if *authority.key != pass.authority {
//         return Err(ProgramError::InvalidArgument);
//     }
//     Ok(bump)
// }

/// Initialize SPL account instruction.
pub fn spl_initialize_account<'a>(
    account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
) -> ProgramResult {
    let ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        account.key,
        mint.key,
        owner.key,
    )?;

    invoke(
        &ix,
        &[account.clone(), mint.clone(), owner.clone(), rent.clone()],
    )
}

/// Initialize SPL mint instruction
pub fn spl_initialize_mint<'a>(
    mint: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    decimals: u8,
) -> ProgramResult {
    let ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        mint.key,
        mint_authority.key,
        None,
        decimals,
    )?;

    invoke(&ix, &[mint.clone(), rent.clone()])
}

/// SPL transfer instruction.
pub fn spl_token_transfer<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[source, destination, authority], signers_seeds)
}

/// SPL mint token
pub fn spl_mint<'a>(
    mint: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint.key,
        token_account.key,
        owner.key,
        &[],
        amount,
    )?;

    invoke_signed(
        &ix,
        &[mint.clone(), token_account.clone(), owner.clone()],
        &[&signer_seeds],
    )
}

/// Create account almost from scratch, lifted from
/// https://github.com/solana-labs/solana-program-library/tree/master/associated-token-account/program/src/processor.rs#L51-L98
#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let accounts = &[new_account_info.clone(), system_program_info.clone()];

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
        &[&signer_seeds],
    )?;

    Ok(())
}


pub fn create_pass_collection<'a>(
    program_id: Pubkey,
    pass_authority_info: &AccountInfo<'a>,
    collection_mint_info: &AccountInfo<'a>,
    collection_metadata_info: &AccountInfo<'a>,
    collection_token_info: &AccountInfo<'a>,
    collection_edition_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    spl_token_program_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> Result<PassStore, ProgramError> {
    // set up pass collection account

    let unpack = PassStore::unpack(&pass_authority_info.data.borrow_mut());

    let proving_process = match unpack {
        Ok(data) => Ok(data),
        Err(_) => {
            let (pass_authority_key, pass_authority_bump_seed) = find_program_authority(&program_id);
            let pass_authority_signer_seeds = &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                &[pass_authority_bump_seed],
            ];
            assert_account_key(pass_authority_info, &pass_authority_key, Some(NFTPassError::InvalidAuthorityKey))?;
            let (pass_collection_key, pass_collection_bump_seed) = find_pass_collection_mint(&program_id);
            let pass_collection_signer_seeds = &[
                PREFIX.as_bytes(),
                program_id.as_ref(),
                &collection_mint_info.key.to_bytes(),
                &[pass_collection_bump_seed],
            ];
            if collection_mint_info.key != &pass_collection_key {
                return Err(NFTPassError::InvalidPassKey.into());
            }
        
            // create pass collection account
            create_or_allocate_account_raw(
                program_id,
                pass_authority_info,
                rent_sysvar_info,
                system_program_info,
                payer_info,
                PassStore::LEN,
                pass_authority_signer_seeds,
            )?;
        
            // create and mint account
            create_or_allocate_account_raw(
                spl_token::id(),
                collection_mint_info,
                rent_sysvar_info,
                system_program_info,
                payer_info,
                spl_token::state::Mint::LEN,
                pass_collection_signer_seeds,
            )?;
        
            spl_initialize_mint(
                collection_mint_info,
                pass_authority_info,
                rent_sysvar_info,
                0,
            )?;
        
            // create associated token account from pass account and pass collection mint to hold collection token
            create_associated_token_account_raw(
                payer_info,
                pass_authority_info,
                collection_mint_info,
                collection_token_info,
                spl_token_program_info,
                rent_sysvar_info,
                system_program_info,
            )?;
        
            // initialize token account to hold mint tokens
            spl_initialize_account(
                collection_token_info,
                collection_mint_info,
                pass_authority_info,
                rent_sysvar_info,
            )?;
        
            // mint token
            spl_mint(
                collection_mint_info,
                collection_token_info,
                pass_authority_info,
                1,
                pass_authority_signer_seeds,
            )?;
        
            create_metadata(
                String::from(""),
                String::from(""),
                String::from(""),
                0,
                None,
                false,
                collection_mint_info,
                collection_metadata_info,
                payer_info,
                pass_authority_info,
                pass_authority_info,
                rent_sysvar_info,
                system_program_info,
                pass_authority_signer_seeds,
            )?;
        
            create_master_edition(
                collection_edition_info,
                collection_mint_info,
                collection_metadata_info,
                payer_info,
                pass_authority_info,
                pass_authority_info,
                Some(1),
                spl_token_program_info,
                rent_sysvar_info,
                system_program_info,
                pass_authority_signer_seeds,
            )?;

            msg!("New pass store account was created");

            let mut data = PassStore::unpack_unchecked(&pass_authority_info.data.borrow_mut())?;

            data.init(*collection_mint_info.key);
            Ok(data)
        }
    };

    proving_process
}


pub fn create_metadata<'a>(
    name: String,
    uri: String,
    symbol: String,
    seller_fee_basis_points: u16,
    creators: Option<Vec<mpl_token_metadata::state::Creator>>,
    is_mutable: bool,
    mint_account: &AccountInfo<'a>,
    metadata_account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &mpl_token_metadata::instruction::create_metadata_accounts(
            mpl_token_metadata::id(),
            *metadata_account.key,
            *mint_account.key,
            *mint_authority.key,
            *payer.key,
            *update_authority.key,
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            true,
            is_mutable,
        ),
        &[
            metadata_account.clone(),
            mint_account.clone(),
            mint_authority.clone(),
            payer.clone(),
            update_authority.clone(),
            system_program_info.clone(),
            rent_sysvar_info.clone(),
        ],
        &[&signer_seeds],
    )?;
    Ok(())
}

pub fn create_master_edition<'a>(
    edition_account: &AccountInfo<'a>,
    mint_account: &AccountInfo<'a>,
    metadata_account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    update_authority: &AccountInfo<'a>,
    max_supply: Option<u64>,
    spl_token_program_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &mpl_token_metadata::instruction::create_master_edition_v3(
            mpl_token_metadata::id(),
            *edition_account.key,
            *mint_account.key,
            *update_authority.key,
            *mint_authority.key,
            *metadata_account.key,
            *payer.key,
            max_supply,
        ),
        &[
            edition_account.clone(),
            mint_account.clone(),
            update_authority.clone(),
            mint_authority.clone(),
            payer.clone(),
            metadata_account.clone(),
            spl_token_program_info.clone(),
            system_program_info.clone(),
            rent_sysvar_info.clone(),
        ],
        &[&signer_seeds],
    )?;
    Ok(())
}

pub fn create_associated_token_account_raw<'a>(
    payer_info: &AccountInfo<'a>,
    wallet_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    token_info: &AccountInfo<'a>,
    spl_token_program_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    invoke(
        &create_associated_token_account(payer_info.key, wallet_info.key, mint_info.key),
        &[
            payer_info.clone(),
            token_info.clone(),
            wallet_info.clone(),
            mint_info.clone(),
            system_program_info.clone(),
            spl_token_program_info.clone(),
            rent_sysvar_info.clone(),
        ],
    )
}
