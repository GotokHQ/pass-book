//! Init pass instruction processing

use crate::{
    error::NFTPassError,
    find_pass_store_program_address, find_payout_program_address,
    instruction::InitPassBookArgs,
    state::{InitPassBook, PassBook, Payout, Store, MAX_NAME_LENGTH, MAX_URI_LENGTH, PREFIX},
    utils::*,
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

use spl_associated_token_account::get_associated_token_address;

use std::slice::Iter;

use spl_token::state::Account;

/// Process InitPass instruction
pub fn init_pass_book(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitPassBookArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pass_book_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let clock = &Clock::from_account_info(clock_info)?;

    assert_signer(pass_book_info)?;
    assert_signer(creator_info)?;

    let is_native = cmp_pubkeys(mint_info.key, &spl_token::native_mint::id());

    if !is_native {
        assert_owned_by(mint_info, &spl_token::id())?;
    }

    let (store_key, store_bump_seed) =
        find_pass_store_program_address(program_id, creator_info.key);
    assert_account_key(store_info, &store_key, Some(NFTPassError::InvalidStoreKey))?;

    let store_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &creator_info.key.to_bytes(),
        Store::PREFIX.as_bytes(),
        &[store_bump_seed],
    ];

    let mut store: Store = get_pass_store_data(
        program_id,
        store_info,
        creator_info,
        payer_account_info,
        rent_info,
        system_account_info,
        store_signer_seeds,
    )?;

    assert_account_key(
        creator_info,
        &store.authority,
        Some(NFTPassError::InvalidCreatorKey),
    )?;

    let mut pass_book = get_or_create_passbook(
        program_id,
        pass_book_info,
        payer_account_info,
        rent_info,
        system_account_info,
    )?;

    // PassBook::unpack_unchecked(&pass_book_info.data.borrow_mut())?;

    if pass_book.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if args.name.len() > MAX_NAME_LENGTH {
        return Err(NFTPassError::NameTooLong.into());
    }

    if args.uri.len() > MAX_URI_LENGTH {
        return Err(NFTPassError::UriTooLong.into());
    }

    if let Some(max_uses) = args.max_uses {
        if max_uses == 0 {
            return Err(NFTPassError::WrongDuration.into());
        }
    }

    if let Some(validity) = args.access {
        if validity == 0 {
            return Err(NFTPassError::WrongValidityPeriod.into());
        }
    }

    if let Some(max_supply) = args.max_supply {
        if max_supply == 0 {
            return Err(NFTPassError::WrongMaxSupply.into());
        }
    }

    get_or_create_payout_account(
        program_id,
        &creator_info.key,
        account_info_iter,
        payer_account_info,
        rent_info,
        system_account_info,
        mint_info,
    )?;

    let market_authority = if args.has_market_authority {
        let market_authority_account = next_account_info(account_info_iter)?;
        assert_signer(market_authority_account)?;
        get_or_create_payout_account(
            program_id,
            &market_authority_account.key,
            account_info_iter,
            payer_account_info,
            rent_info,
            system_account_info,
            mint_info,
        )?;
        Some(*market_authority_account.key)
    } else {
        None
    };

    if args.has_referrer {
        let referrer_account = next_account_info(account_info_iter)?;
        get_or_create_payout_account(
            program_id,
            &referrer_account.key,
            account_info_iter,
            payer_account_info,
            rent_info,
            system_account_info,
            mint_info,
        )?;
        store.referrer = Some(*referrer_account.key);
        store.referral_end_date = args.referral_end_date;
    }

    pass_book.init(InitPassBook {
        name: args.name,
        uri: args.uri,
        description: args.description,
        authority: *creator_info.key,
        mutable: args.mutable,
        max_uses: args.max_uses,
        access: args.access,
        max_supply: args.max_supply,
        created_at: clock.unix_timestamp as u64,
        price: args.price,
        mint: *mint_info.key,
        market_authority: market_authority,
    });

    pass_book.puff_out_data_fields();

    PassBook::pack(pass_book, *pass_book_info.data.borrow_mut())?;
    store.increment_pass_book_count()?;
    Store::pack(store, *store_info.data.borrow_mut())?;
    Ok(())
}

pub fn get_pass_store_data<'a>(
    program_id: &Pubkey,
    store_info: &AccountInfo<'a>,
    creator_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    signers_seeds: &[&[u8]],
) -> Result<Store, ProgramError> {
    // set up pass store account

    let unpack = Store::unpack(&store_info.data.borrow_mut());

    let proving_process = match unpack {
        Ok(data) => Ok(data),
        Err(_) => {
            // create pass store account
            create_or_allocate_account_raw(
                *program_id,
                store_info,
                rent_sysvar_info,
                system_program_info,
                payer_info,
                Store::LEN,
                signers_seeds,
            )?;

            msg!("New pass store account was created");

            let mut data = Store::unpack_unchecked(&store_info.data.borrow_mut())?;

            data.init(*creator_info.key);
            Ok(data)
        }
    };

    proving_process
}

pub fn get_or_create_passbook<'a>(
    program_id: &Pubkey,
    passbook_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> Result<PassBook, ProgramError> {
    // set up pass store account

    let unpack = PassBook::unpack(&passbook_info.data.borrow_mut());

    let proving_process = match unpack {
        Ok(data) => Ok(data),
        Err(_) => {
            // create pass store account
            create_or_new_account_raw(
                *program_id,
                passbook_info,
                rent_sysvar_info,
                system_program_info,
                payer_info,
                PassBook::LEN,
            )?;

            msg!("New passbook account was created");
            Ok(PassBook::unpack_unchecked(
                &passbook_info.data.borrow_mut(),
            )?)
        }
    };

    proving_process
}

pub fn get_or_create_payout_account<'a>(
    program_id: &Pubkey,
    authority: &Pubkey,
    remaining_accounts: &mut Iter<AccountInfo<'a>>,
    payer_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    // set up pass store account
    let payout_info = next_account_info(remaining_accounts)?;
    let treasury_holder_info = next_account_info(remaining_accounts)?;
    let (payout_key, payout_bump_seed) =
        find_payout_program_address(program_id, authority, mint_info.key);

    msg!("PAYOUT INFO KEY -----------------> {}", payout_info.key);
    msg!("PAYOUT DERIVED KEY -----------------> {}", payout_key);
    assert_account_key(
        payout_info,
        &payout_key,
        Some(NFTPassError::InvalidPayoutKey),
    )?;
    let payout_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &authority.to_bytes(),
        &mint_info.key.to_bytes(),
        Payout::PREFIX.as_bytes(),
        &[payout_bump_seed],
    ];
    let unpack = Payout::unpack(&payout_info.data.borrow_mut());

    match unpack {
        Ok(_) => Ok(()),
        Err(_) => {
            let is_native = cmp_pubkeys(mint_info.key, &spl_token::native_mint::id());
            if is_native {
                if treasury_holder_info.key != payout_info.key {
                    return Err(ProgramError::InvalidAccountData);
                }
            } else {
                let associated_token_account =
                    get_associated_token_address(&payout_key, &mint_info.key);

                // Check, that provided destination is associated token account
                if associated_token_account != *treasury_holder_info.key {
                    return Err(NFTPassError::InvalidPayerTokenAccount.into());
                }

                let _: Account = assert_initialized(treasury_holder_info)?;
                msg!("Token initialized");
            }
            // create payout account
            create_or_allocate_account_raw(
                *program_id,
                payout_info,
                rent_sysvar_info,
                system_program_info,
                payer_info,
                Payout::LEN,
                payout_signer_seeds,
            )?;

            msg!("New payout account was created");

            let mut data = Payout::unpack_unchecked(&payout_info.data.borrow_mut())?;

            data.init(*authority, *mint_info.key, *treasury_holder_info.key);
            Payout::pack(data, *payout_info.data.borrow_mut())?;
            Ok(())
        }
    }
}
