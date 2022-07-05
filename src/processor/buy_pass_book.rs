//! Init pass instruction processing

use crate::{
    error::NFTPassError,
    find_pass_book_program_address, find_pass_store_program_address,
    find_trade_history_program_address, id,
    instruction::BuyPassArgs,
    state::{PassBook, PassStore, Payout, TradeHistory, PREFIX},
    utils::*,
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

use mpl_token_metadata::{
    state::Metadata,
    utils::{assert_initialized, get_supply_off_master_edition},
};

use std::slice::Iter;

use spl_token::state::Account;

/// Process InitPass instruction
pub fn buy<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: BuyPassArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pass_book_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let vault_token_account_info = next_account_info(account_info_iter)?;
    let user_wallet_info = next_account_info(account_info_iter)?;
    let user_token_account_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let new_metadata_info = next_account_info(account_info_iter)?;
    let new_edition_info = next_account_info(account_info_iter)?;
    let new_mint_info = next_account_info(account_info_iter)?;
    let master_metadata_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;
    let edition_marker_info = next_account_info(account_info_iter)?;
    let new_token_account_info = next_account_info(account_info_iter)?;

    let trade_history_info = next_account_info(account_info_iter)?;

    let clock_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let clock = &Clock::from_account_info(clock_info)?;

    assert_owned_by(pass_book_info, &id())?;
    assert_owned_by(store_info, &id())?;
    assert_owned_by(vault_token_account_info, &spl_token::id())?;

    assert_signer(user_wallet_info)?;

    assert_owned_by(new_mint_info, &spl_token::id())?;
    assert_owned_by(master_metadata_info, &mpl_token_metadata::id())?;
    assert_owned_by(master_edition_info, &mpl_token_metadata::id())?;

    assert_owned_by(new_token_account_info, &spl_token::id())?;

    let mut passbook = PassBook::unpack(&pass_book_info.data.borrow_mut())?;

    let mut pass_store = PassStore::unpack(&store_info.data.borrow_mut())?;

    let (store_key, _) = find_pass_store_program_address(program_id, &passbook.authority);
    assert_account_key(store_info, &store_key, Some(NFTPassError::InvalidStoreKey))?;
    assert_account_key(
        vault_token_account_info,
        &passbook.token,
        Some(NFTPassError::InvalidVaultToken),
    )?;
    let master_metadata = Metadata::from_account_info(master_metadata_info)?;
    if master_metadata.mint != passbook.mint {
        return Err(NFTPassError::InvalidMintKey.into());
    }

    let is_native = cmp_pubkeys(&passbook.price_mint, &spl_token::native_mint::id());

    if is_native {
        assert_account_key(
            user_wallet_info,
            &user_token_account_info.key,
            Some(NFTPassError::UserWalletMustMatchUserTokenAccount),
        )?;
    } else {
        let user_token_account: Account = assert_initialized(user_token_account_info)?;
        if user_token_account.mint != passbook.price_mint {
            return Err(NFTPassError::PriceTokenMismatch.into());
        }
        if user_token_account.owner != *user_wallet_info.key {
            msg!("USER TOKEN OWNER DOES NOT MATCH WALLET OWNER ----------------->");
            return Err(ProgramError::IllegalOwner);
        }
    }

    let edition = get_supply_off_master_edition(&master_edition_info)?
        .checked_add(1)
        .ok_or(NFTPassError::MathOverflow)?;

    let (_, pass_book_bump_seed) = find_pass_book_program_address(program_id, &passbook.mint);

    let pass_book_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &passbook.mint.to_bytes(),
        &[pass_book_bump_seed],
    ];

    mpl_mint_new_edition_from_master_edition_via_token(
        &new_metadata_info,
        &new_edition_info,
        &new_mint_info,
        &payer_account_info,
        &payer_account_info,
        user_wallet_info,
        &pass_book_info,
        &vault_token_account_info,
        &master_metadata_info,
        &master_edition_info,
        &master_metadata.mint,
        &edition_marker_info,
        &token_program_info,
        &system_account_info,
        &rent_info,
        edition,
        pass_book_signer_seeds,
    )?;
    let new_token_account: Account = assert_initialized(new_token_account_info)?;
    if new_token_account.owner != *user_wallet_info.key {
        return Err(ProgramError::IllegalOwner);
    }

    mpl_update_primary_sale_happened_via_token(
        &new_metadata_info,
        &user_wallet_info,
        &new_token_account_info,
    )?;

    let (trade_history_key, trade_history_bump_seed) =
        find_trade_history_program_address(program_id, pass_book_info.key, user_wallet_info.key);
    assert_account_key(
        trade_history_info,
        &trade_history_key,
        Some(NFTPassError::InvalidTradeHistoryKey),
    )?;

    let trade_history_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &pass_book_info.key.to_bytes(),
        &user_wallet_info.key.to_bytes(),
        TradeHistory::PREFIX.as_bytes(),
        &[trade_history_bump_seed],
    ];

    let mut trade_history = get_or_create_trade_history(
        program_id,
        trade_history_info,
        pass_book_info,
        user_wallet_info,
        payer_account_info,
        rent_info,
        system_account_info,
        trade_history_signer_seeds,
    )?;

    // Check, that user not reach buy limit
    // todo, add to passbook
    if let Some(pieces_in_one_wallet) = passbook.pieces_in_one_wallet {
        if trade_history.already_bought == pieces_in_one_wallet {
            return Err(NFTPassError::UserReachBuyLimit.into());
        }
    }

    distribute_payout(
        args.market_fee_basis_point as u64,
        args.referral_share as u64,
        args.referral_kick_back_share as u64,
        &master_metadata,
        &passbook,
        &pass_store,
        user_wallet_info.clone(),
        user_token_account_info.clone(),
        clock,
        account_info_iter,
    )?;

    pass_store.increment_pass_count()?;
    trade_history.increment_already_bought()?;
    passbook.increment_supply()?;
    PassBook::pack(passbook, *pass_book_info.data.borrow_mut())?;
    PassStore::pack(pass_store, *store_info.data.borrow_mut())?;
    TradeHistory::pack(trade_history, *trade_history_info.data.borrow_mut())?;
    Ok(())
}

pub fn transfer<'a>(
    is_native: bool,
    source_account_info: &AccountInfo<'a>,
    destination_account_info: &AccountInfo<'a>,
    owner_account_info: &AccountInfo<'a>,
    amount: u64,
) -> Result<(), ProgramError> {
    if is_native {
        native_transfer(
            source_account_info.clone(),
            destination_account_info.clone(),
            amount,
        )
    } else {
        // Transfer from source to token account
        spl_token_transfer(
            source_account_info.clone(),
            destination_account_info.clone(),
            owner_account_info.clone(),
            amount, // transfer master edition
            &[],
        )
    }
}

pub fn pay_account<'a>(
    amount: u64,
    authority: &Pubkey,
    passbook: &PassBook,
    user_wallet: &AccountInfo<'a>,
    user_token_account: &AccountInfo<'a>,
    payout_account: &AccountInfo<'a>,
    payout_token_account: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    let mut payout = Payout::unpack(&payout_account.data.borrow_mut())?;
    if *authority != payout.authority && passbook.price_mint != payout.mint {
        return Err(NFTPassError::InvalidPayoutKey.into());
    }
    let is_native = cmp_pubkeys(&passbook.price_mint, &spl_token::native_mint::id());
    if is_native {
        if payout_token_account.key != payout_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
    } else {
        assert_owned_by(payout_token_account, &spl_token::id())?;
        let token_account: Account = assert_initialized(payout_token_account)?;
        if token_account.mint != passbook.price_mint {
            return Err(NFTPassError::PriceTokenMismatch.into());
        }
        if token_account.owner != *payout_account.key {
            msg!("PAYOUT TOKEN OWNER DOES NOT MATCH PAYOUT ACCOUNT ----------------->");
            return Err(ProgramError::IllegalOwner);
        }
    }
    transfer(
        is_native,
        user_token_account,
        payout_token_account,
        user_wallet,
        amount,
    )?;
    payout.cash_in = payout
        .cash_in
        .checked_add(amount)
        .ok_or(NFTPassError::MathOverflow)?;
    Payout::pack(payout, *payout_account.data.borrow_mut())?;
    Ok(())
}

pub fn distribute_payout<'a>(
    market_fee_basis_point: u64,
    referral_share: u64,
    referral_kick_back: u64,
    metadata: &Metadata,
    passbook: &PassBook,
    store: &PassStore,
    user_wallet: AccountInfo<'a>,
    user_token_account: AccountInfo<'a>,
    clock: &Clock,
    remaining_accounts: &mut Iter<'a, AccountInfo<'a>>,
) -> Result<(), ProgramError> {
    if referral_share > 100 {
        return Err(NFTPassError::WrongReferralShare.into());
    }
    if referral_kick_back > 100 {
        return Err(NFTPassError::WrongReferralShare.into());
    }
    let amount_for_creators = calculate_shares_less_points(passbook.price, market_fee_basis_point)?;
    let mut creator_payout: Vec<PayoutInfo> = vec![];

    if let Some(creators) = &metadata.data.creators {
        for creator in creators {
            let payout_info = next_account_info(remaining_accounts)?;
            let payout_token_info = next_account_info(remaining_accounts)?;
            creator_payout.push(PayoutInfo {
                authority: creator.address,
                payout_account: payout_info,
                token_account: payout_token_info,
                share: creator.share,
            })
        }
    }

    distribute_payout_for_creators(
        amount_for_creators,
        passbook,
        metadata,
        &user_wallet,
        &user_token_account,
        &creator_payout,
    )?;

    let amount_for_market_place =
        calculate_amount_for_points(passbook.price, market_fee_basis_point)?;
    if let Some(market_authority) = passbook.market_authority {
        let market_authority_account_info = next_account_info(remaining_accounts)?;
        assert_signer(market_authority_account_info)?;
        assert_account_key(
            market_authority_account_info,
            &market_authority,
            Some(NFTPassError::InvalidMarketAuthority),
        )?;
        let market_payout_info = next_account_info(remaining_accounts)?;
        let market_payout_token_info = next_account_info(remaining_accounts)?;
        let market_amount = calculate_shares(amount_for_market_place, 100 - referral_share)?;
        pay_account(
            market_amount,
            &market_authority,
            passbook,
            &user_wallet,
            &user_token_account,
            market_payout_info,
            market_payout_token_info,
        )?;
    }

    if let Some(referrer) = store.referrer {
        if let Some(referral_end_date) = store.referral_end_date {
            if clock.unix_timestamp as u64 > referral_end_date {
                return Ok(());
            }
        }
        let amount_for_referrer = calculate_shares(amount_for_market_place, referral_share)?;
        let referrer_account_info = next_account_info(remaining_accounts)?;
        assert_account_key(
            referrer_account_info,
            &referrer,
            Some(NFTPassError::InvalidMarketAuthority),
        )?;
        let referrer_payout_info = next_account_info(remaining_accounts)?;
        let referrer_payout_token_info = next_account_info(remaining_accounts)?;
        let referrer_kick_back_amount = calculate_shares(amount_for_referrer, referral_kick_back)?;
        let referrer_amount = calculate_shares(amount_for_referrer, 100 - referral_kick_back)?;
        pay_account(
            referrer_amount,
            &referrer,
            passbook,
            &user_wallet,
            &user_token_account,
            referrer_payout_info,
            referrer_payout_token_info,
        )?;
        distribute_referral_payout_for_creators(
            referrer_kick_back_amount,
            passbook,
            &user_wallet,
            &user_token_account,
            &creator_payout,
        )?;
    }
    Ok(())
}

pub fn distribute_payout_for_creators<'a>(
    amount: u64,
    passbook: &PassBook,
    metadata: &Metadata,
    user_wallet: &AccountInfo<'a>,
    user_token_account: &AccountInfo<'a>,
    payout_accounts: &[PayoutInfo<'a>],
) -> Result<(), ProgramError> {
    if amount == 0 {
        return Ok(());
    }
    for creator in payout_accounts {
        let creator_amount = if metadata.primary_sale_happened {
            calculate_user_shares_by_points(
                amount,
                metadata.data.seller_fee_basis_points as u64,
                creator.share as u64,
            )?
        } else {
            calculate_shares(amount, creator.share as u64)?
        };
        pay_account(
            creator_amount,
            &creator.authority,
            passbook,
            user_wallet,
            user_token_account,
            creator.payout_account,
            creator.token_account,
        )?;
    }
    Ok(())
}

pub fn distribute_referral_payout_for_creators<'a>(
    amount: u64,
    passbook: &PassBook,
    user_wallet: &AccountInfo<'a>,
    user_token_account: &AccountInfo<'a>,
    payout_accounts: &[PayoutInfo<'a>],
) -> Result<(), ProgramError> {
    if amount == 0 {
        return Ok(());
    }
    for creator in payout_accounts {
        let creator_amount = calculate_shares(amount, creator.share as u64)?;
        pay_account(
            creator_amount,
            &creator.authority,
            passbook,
            user_wallet,
            user_token_account,
            creator.payout_account,
            creator.token_account,
        )?;
    }
    Ok(())
}

pub fn get_or_create_trade_history<'a>(
    program_id: &Pubkey,
    history_info: &AccountInfo<'a>,
    passbook_info: &AccountInfo<'a>,
    user_wallet_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    signers_seeds: &[&[u8]],
) -> Result<TradeHistory, ProgramError> {
    // set up pass store account

    let unpack = TradeHistory::unpack(&history_info.data.borrow_mut());

    let proving_process = match unpack {
        Ok(data) => Ok(data),
        Err(_) => {
            // create pass store account
            create_or_allocate_account_raw(
                *program_id,
                history_info,
                rent_sysvar_info,
                system_program_info,
                payer_info,
                TradeHistory::LEN,
                signers_seeds,
            )?;

            msg!("New trade history account was created");

            let mut data = TradeHistory::unpack_unchecked(&history_info.data.borrow_mut())?;

            data.init(*passbook_info.key, *user_wallet_info.key);
            Ok(data)
        }
    };

    proving_process
}

pub struct PayoutInfo<'a> {
    pub authority: Pubkey,
    pub payout_account: &'a AccountInfo<'a>,
    pub token_account: &'a AccountInfo<'a>,
    pub share: u8,
}
