use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar,
};
use spl_token::{
    instruction::transfer_checked,
    state::{Account as TokenAccount, Mint},
};

use crate::{
    error::MesonError,
    state::{self, owner_of_pool, ConstantValue, PostedSwap},
    utils::Utils,
};

pub fn post_swap<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    user_account: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    token_program_info: &'a AccountInfo<'b>,
    save_token_list_account: &'a AccountInfo<'b>,
    save_ps_account_input: &'a AccountInfo<'b>,
    ta_user_input: &'a AccountInfo<'b>,
    ta_program_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
    pool_index: u64,
) -> ProgramResult {
    Utils::for_initial_chain(encoded_swap)?;
    state::match_token_address(
        save_token_list_account,
        token_mint_account,
        Utils::in_coin_index_from(encoded_swap),
    )?;

    let amount = Utils::amount_from(encoded_swap);
    Utils::amount_within_max(amount)?;

    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp.to_le() as u64;
    let delta = Utils::expire_ts_from(encoded_swap) - now_timestamp;
    if delta < Utils::get_min_bond_time_period() {
        return Err(MesonError::SwapExpireTooEarly.into());
    }
    if delta > Utils::get_max_bond_time_period() {
        return Err(MesonError::SwapExpireTooLate.into());
    }

    msg!("Signature    : {:?}", signature);
    // Utils::check_request_signature(encoded_swap, signature, initiator)?; // todo()

    state::add_posted_swap(
        program_id,
        payer_account,
        system_program,
        encoded_swap,
        pool_index,
        initiator,
        *user_account.key,
        save_ps_account_input,
    )?;

    let decimals = Mint::unpack(&token_mint_account.data.borrow())?.decimals;
    invoke_signed(
        &transfer_checked(
            token_program_info.key,
            ta_user_input.key,
            token_mint_account.key,
            ta_program_input.key,
            user_account.key,
            &[],
            amount,
            decimals,
        )
        .unwrap(),
        &[
            ta_user_input.clone(),
            token_mint_account.clone(),
            ta_program_input.clone(),
            user_account.clone(),
        ],
        &[],
    )?;

    Ok(())
}

pub fn bond_swap<'a, 'b>(
    program_id: &Pubkey,
    sender_account: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
    save_ps_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    pool_index: u64,
) -> ProgramResult {
    let pool_index_expected =
        state::pool_index_of(program_id, sender_account, save_poaa_account_input)?;
    if pool_index != pool_index_expected {
        Err(MesonError::PoolIndexMismatch.into())
    } else {
        state::bond_posted_swap(program_id, encoded_swap, pool_index, save_ps_account_input)
    }
}

pub fn cancel_swap<'a, 'b>(
    program_id: &Pubkey,
    token_mint_account: &'a AccountInfo<'b>,
    token_program_info: &'a AccountInfo<'b>,
    save_ps_account_input: &'a AccountInfo<'b>,
    ta_user_input: &'a AccountInfo<'b>,
    ta_program_input: &'a AccountInfo<'b>,
    contract_signer_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
) -> ProgramResult {
    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp.to_le() as u64;
    let expire_ts = Utils::expire_ts_from(encoded_swap);
    if expire_ts > now_timestamp {
        return Err(MesonError::SwapCannotCancelBeforeExpire.into());
    }

    let (expected_contract_signer, bump_seed) =
        Pubkey::find_program_address(&[ConstantValue::CONTRACT_SIGNER], program_id);
    if expected_contract_signer != *contract_signer_account_input.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }
    let posted = state::remove_posted_swap(program_id, encoded_swap, save_ps_account_input)?;

    let user_pubkey_expected = TokenAccount::unpack(&ta_user_input.data.borrow())?.owner;
    if posted.from_address != user_pubkey_expected {
        return Err(MesonError::TokenAccountMismatch.into());
    }
    let amount = Utils::amount_from(encoded_swap);
    let decimals = Mint::unpack(&token_mint_account.data.borrow())?.decimals;

    invoke_signed(
        &transfer_checked(
            token_program_info.key,
            ta_program_input.key,
            token_mint_account.key,
            ta_user_input.key,
            &expected_contract_signer,
            &[],
            amount,
            decimals,
        )
        .unwrap(),
        &[
            ta_program_input.clone(),
            token_mint_account.clone(),
            ta_user_input.clone(),
            contract_signer_account_input.clone(),
        ],
        &[&[ConstantValue::CONTRACT_SIGNER, &[bump_seed]]],
    )?;
    Ok(())
}

pub fn execute_swap<'a, 'b>(
    program_id: &Pubkey,
    token_mint_account: &'a AccountInfo<'b>,
    token_program_info: &'a AccountInfo<'b>,
    save_ps_account_input: &'a AccountInfo<'b>,
    save_oop_account_input: &'a AccountInfo<'b>,
    ta_lp_input: &'a AccountInfo<'b>,
    ta_program_input: &'a AccountInfo<'b>,
    contract_signer_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    recipient: [u8; 20],
    // deposit_to_pool: todo(), default false
) -> ProgramResult {
    let (expected_contract_signer, bump_seed) =
        Pubkey::find_program_address(&[ConstantValue::CONTRACT_SIGNER], program_id);
    if expected_contract_signer != *contract_signer_account_input.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }

    let PostedSwap {
        pool_index,
        initiator,
        from_address: _,
    } = state::remove_posted_swap(program_id, encoded_swap, save_ps_account_input)?;

    if pool_index == 0 {
        return Err(MesonError::PoolIndexCannotBeZero.into());
    }

    msg!("Signature    : {:?}", signature);
    msg!("Recipient    : {:?}", recipient);
    msg!("Initiator    : {:?}", initiator);
    // Utils::check_release_signature(encoded_swap, recipient, signature, initiator)?;

    let lp_pubkey = owner_of_pool(program_id, pool_index, save_oop_account_input)?;
    let lp_pubkey_expected = TokenAccount::unpack(&ta_lp_input.data.borrow())?.owner;
    if lp_pubkey != lp_pubkey_expected {
        return Err(MesonError::TokenAccountMismatch.into());
    }
    let decimals = Mint::unpack(&token_mint_account.data.borrow())?.decimals;
    let amount = Utils::amount_from(encoded_swap);

    invoke_signed(
        &transfer_checked(
            token_program_info.key,
            ta_program_input.key,
            token_mint_account.key,
            ta_lp_input.key,
            &expected_contract_signer,
            &[],
            amount,
            decimals,
        )
        .unwrap(),
        &[
            ta_program_input.clone(),
            token_mint_account.clone(),
            ta_lp_input.clone(),
            contract_signer_account_input.clone(),
        ],
        &[&[ConstantValue::CONTRACT_SIGNER, &[bump_seed]]],
    )?;
    Ok(())
}
