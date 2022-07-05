//! Program utils

use crate::error::NFTPassError;

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_memory::sol_memcmp,
    program_pack::IsInitialized,
    pubkey::{Pubkey, PUBKEY_BYTES},
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use spl_associated_token_account::{instruction::create_associated_token_account};

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
pub fn assert_account_key(
    account_info: &AccountInfo,
    key: &Pubkey,
    error: Option<NFTPassError>,
) -> ProgramResult {
    if *account_info.key != *key {
        match error {
            Some(e) => Err(e.into()),
            _ => Err(ProgramError::InvalidArgument),
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

/// Native instruction.
pub fn native_transfer<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    amount: u64,
) -> Result<(), ProgramError> {
    invoke(
        // for native SOL transfer user_wallet key == user_token_account key
        &system_instruction::transfer(&source.key, &destination.key, amount),
        &[source, destination],
    )
}

/// SPL transfer instruction.
pub fn sign_metadata<'a>(
    creator: &AccountInfo<'a>,
    metadata_account: &AccountInfo<'a>,
    signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let ix = mpl_token_metadata::instruction::sign_metadata(
        mpl_token_metadata::id(),
        *metadata_account.key,
        *creator.key,
    );
    invoke_signed(
        &ix,
        &[creator.clone(), metadata_account.clone()],
        signers_seeds,
    )
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

/// transfer all the SOL from source to receiver
pub fn empty_account_balance(
    source: &AccountInfo,
    receiver: &AccountInfo,
) -> Result<(), ProgramError> {
    let mut from = source.try_borrow_mut_lamports()?;
    let mut to = receiver.try_borrow_mut_lamports()?;
    **to += **from;
    **from = 0;
    Ok(())
}

/// Checks two pubkeys for equality in a computationally cheap way using
/// `sol_memcmp`
pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

/// Wrapper of `mint_new_edition_from_master_edition_via_token` instruction from `mpl_token_metadata` program
#[inline(always)]
pub fn mpl_mint_new_edition_from_master_edition_via_token<'a>(
    new_metadata: &AccountInfo<'a>,
    new_edition: &AccountInfo<'a>,
    new_mint: &AccountInfo<'a>,
    new_mint_authority: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    user_wallet: &AccountInfo<'a>,
    token_account_owner: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    master_metadata: &AccountInfo<'a>,
    master_edition: &AccountInfo<'a>,
    metadata_mint: &Pubkey,
    edition_marker: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    edition: u64,
    signers_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let tx = mpl_token_metadata::instruction::mint_new_edition_from_master_edition_via_token(
        mpl_token_metadata::id(),
        *new_metadata.key,
        *new_edition.key,
        *master_edition.key,
        *new_mint.key,
        *new_mint_authority.key,
        *payer.key,
        *token_account_owner.key,
        *token_account.key,
        *user_wallet.key,
        *master_metadata.key,
        *metadata_mint,
        edition,
    );

    invoke_signed(
        &tx,
        &[
            new_metadata.clone(),
            new_edition.clone(),
            master_edition.clone(),
            new_mint.clone(),
            edition_marker.clone(),
            new_mint_authority.clone(),
            user_wallet.clone(),
            token_account_owner.clone(),
            token_account.clone(),
            user_wallet.clone(),
            master_metadata.clone(),
            token_program.clone(),
            system_program.clone(),
            rent.clone(),
        ],
        &[&signers_seeds],
    )
}

/// Wrapper of `update_primary_sale_happened_via_token` instruction from `mpl_token_metadata` program
#[inline(always)]
pub fn mpl_update_primary_sale_happened_via_token<'a>(
    metadata: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    token: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    let tx = mpl_token_metadata::instruction::update_primary_sale_happened_via_token(
        mpl_token_metadata::id(),
        *metadata.key,
        *owner.key,
        *token.key,
    );
    invoke(
        &tx,
        &[metadata.clone(), owner.clone(), token.clone()],
    )
}

pub fn calculate_shares(total_amount: u64, shares: u64) -> Result<u64, ProgramError> {
    Ok(total_amount
        .checked_mul(shares)
        .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?
        .checked_div(100)
        .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?)
}

pub fn calculate_user_shares_by_points(
    total_amount: u64,
    seller_fee_basis_points: u64,
    shares: u64,
) -> Result<u64, ProgramError> {
    Ok((total_amount
        .checked_mul(seller_fee_basis_points)
        .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?
        .checked_div(10000)
        .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?)
    .checked_mul(shares)
    .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?
    .checked_div(100)
    .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?)
}

pub fn calculate_amount_for_points(total_amount: u64, seller_fee_basis_points: u64) -> Result<u64, ProgramError> {
    Ok(total_amount
        .checked_mul(seller_fee_basis_points)
        .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?
        .checked_div(10000)
        .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?)
}

pub fn calculate_shares_less_points(
    total_amount: u64,
    seller_fee_basis_points: u64,
) -> Result<u64, ProgramError> {
    Ok(total_amount
        .checked_sub(
            total_amount
                .checked_mul(seller_fee_basis_points)
                .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?
                .checked_div(10000)
                .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?,
        )
        .ok_or::<ProgramError>(NFTPassError::MathOverflow.into())?)
}

pub fn create_associated_token_account_raw<'a>(
    payer_info: &AccountInfo<'a>,
    wallet_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    token_info: &AccountInfo<'a>,
    spl_token_program_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    signers_seeds: &[&[u8]],
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