use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::transfer;

use crate::{
    error::MesonError,
    state::{self, ConstantValue},
    utils::Utils,
};

pub fn post_swap<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    user_account: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    token_program_info: &'a AccountInfo<'b>,
    save_map_token_account: &'a AccountInfo<'b>,
    save_ps_account_input: &'a AccountInfo<'b>,
    ta_user_input: &'a AccountInfo<'b>,
    ta_program_input: &'a AccountInfo<'b>,
    contract_signer_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
    pool_index: u64,
) -> ProgramResult {
    let (expected_contract_signer, bump_seed) =
        Pubkey::find_program_address(&[ConstantValue::CONTRACT_SIGNER], program_id);
    if expected_contract_signer != *contract_signer_account_input.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }

    Utils::for_initial_chain(encoded_swap)?;
    state::match_token_address(
        save_map_token_account,
        token_mint_account,
        Utils::in_coin_index_from(encoded_swap),
    )?;

    let amount = Utils::amount_from(encoded_swap);
    Utils::amount_within_max(amount)?;

    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp.to_le() as u64;
    let delta = Utils::expire_ts_from(encoded_swap) - now_timestamp;
    if delta < Utils::get_min_bond_time_period() {
        return Err(MesonError::SwapExpireTooEarly.into())
    }
    if delta > Utils::get_max_bond_time_period() {
        return Err(MesonError::SwapExpireTooLate.into())
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

    invoke_signed(
        &transfer(
            token_program_info.key,
            ta_user_input.key,
            ta_program_input.key,
            user_account.key,
            &[],
            amount,
        )
        .unwrap(),
        &[
            ta_user_input.clone(),
            ta_program_input.clone(),
            user_account.clone(),
        ],
        &[&[ConstantValue::CONTRACT_SIGNER, &[bump_seed]]],
    )?;

    Ok(())
}

