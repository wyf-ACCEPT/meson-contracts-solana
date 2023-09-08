use arrayref::array_ref;
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
    state::{
        self, check_balance_account_directly, owner_of_pool, write_related_account, ConstantValue,
        PostedSwap,
    },
    utils::Utils,
};

// This function is different from `deposit_and_register` in move/solidity. You should call `register_pool_index` before call this function.
pub fn deposit_to_pool<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    authorized_account_input: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    token_program_info: &'a AccountInfo<'b>,
    save_token_list_account: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
    save_balance_lp_account_input: &'a AccountInfo<'b>,
    ta_lp_input: &'a AccountInfo<'b>,
    ta_program_input: &'a AccountInfo<'b>,
    pool_index: u64,
    coin_index: u8,
    amount: u64,
) -> ProgramResult {
    // Check token address and pool index
    state::match_pool_index(
        program_id,
        pool_index,
        authorized_account_input,
        save_poaa_account_input,
    )?;
    state::match_token_address(save_token_list_account, token_mint_account, coin_index)?;

    // Deposit token to the contract
    let decimals = Mint::unpack(&token_mint_account.data.borrow())?.decimals;
    invoke_signed(
        &transfer_checked(
            token_program_info.key,
            ta_lp_input.key,
            token_mint_account.key,
            ta_program_input.key,
            authorized_account_input.key,
            &[],
            amount,
            decimals,
        )
        .unwrap(),
        &[
            ta_lp_input.clone(),
            token_mint_account.clone(),
            ta_program_input.clone(),
            authorized_account_input.clone(),
        ],
        &[],
    )?;

    // Check the correctness of `save_balance_lp_account_input`
    state::check_balance_account_directly(
        program_id,
        pool_index,
        coin_index,
        save_balance_lp_account_input,
    )?;

    // First time to deposit -- register a data account to save the balance
    if save_balance_lp_account_input.data_len() == 0 {
        let mut pool_coin_array = [0; 9];
        pool_coin_array[0..8].copy_from_slice(&pool_index.to_be_bytes());
        pool_coin_array[8] = coin_index;
        msg!("Create new pool-coin data account.");
        state::create_related_account(
            program_id,
            payer_account,
            save_balance_lp_account_input,
            system_program,
            ConstantValue::SAVE_BALANCE_PHRASE,
            &pool_coin_array,
            8,
        )?;
    }

    // Update balance
    let balance_amount;
    {
        let balance_data = save_balance_lp_account_input.data.borrow();
        balance_amount = u64::from_be_bytes(*array_ref![balance_data, 0, 8]);
    } // See the annotation of `bond_posted_swap` for explanation of these code
    write_related_account(
        save_balance_lp_account_input,
        &(balance_amount + amount).to_be_bytes(),
    )?;

    Ok(())
}

pub fn withdraw_from_pool<'a, 'b>(
    program_id: &Pubkey,
    authorized_account_input: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    token_program_info: &'a AccountInfo<'b>,
    save_token_list_account: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
    save_balance_lp_account_input: &'a AccountInfo<'b>,
    ta_lp_input: &'a AccountInfo<'b>,
    ta_program_input: &'a AccountInfo<'b>,
    contract_signer_account_input: &'a AccountInfo<'b>,
    pool_index: u64,
    coin_index: u8,
    amount: u64,
) -> ProgramResult {
    // Check token address, pool index and contract signer account input
    state::match_pool_index(
        program_id,
        pool_index,
        authorized_account_input,
        save_poaa_account_input,
    )?;
    state::match_token_address(save_token_list_account, token_mint_account, coin_index)?;

    // Withdraw token from the contract
    let decimals = Mint::unpack(&token_mint_account.data.borrow())?.decimals;
    let (expected_contract_signer, bump_seed) =
        Pubkey::find_program_address(&[ConstantValue::CONTRACT_SIGNER], program_id);
    if expected_contract_signer != *contract_signer_account_input.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }
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

    // Update balance
    let balance_amount;
    {
        let balance_data = save_balance_lp_account_input.data.borrow();
        balance_amount = u64::from_be_bytes(*array_ref![balance_data, 0, 8]);
    } // See the annotation of `bond_posted_swap` for explanation of these code

    write_related_account(
        save_balance_lp_account_input,
        &(balance_amount - amount).to_be_bytes(),
    )?;

    Ok(())
}

pub fn lock<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    authorized_account_input: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    save_si_account_input: &'a AccountInfo<'b>,
    save_token_list_account: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
    save_balance_lp_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
    recipient: Pubkey,
) -> ProgramResult {
    // Check the basic conditions
    let coin_index = Utils::out_coin_index_from(encoded_swap);
    let swap_id = Utils::get_swap_id(encoded_swap, initiator);
    let amount = Utils::amount_from(encoded_swap) - Utils::fee_for_lp(encoded_swap);
    Utils::amount_within_max(amount)?;
    Utils::for_target_chain(encoded_swap)?;
    state::match_token_address(save_token_list_account, token_mint_account, coin_index)?;
    let pool_index = state::pool_index_of(
        program_id,
        authorized_account_input,
        save_poaa_account_input,
    )?;
    if pool_index == 0 {
        return Err(MesonError::PoolIndexCannotBeZero.into());
    }

    // Check the time and signature
    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp.to_le() as u64;
    let until = now_timestamp + Utils::get_lock_time_period();
    if until > Utils::expire_ts_from(encoded_swap) - 300 {
        return Err(MesonError::SwapExpireTsIsSoon.into());
    }

    msg!("Signature    : {:?}", signature);
    Utils::check_request_signature(encoded_swap, signature, initiator)?; // todo()

    // Change the state of locked-swap table
    state::add_locked_swap(
        program_id,
        payer_account,
        system_program,
        swap_id,
        pool_index,
        until,
        recipient,
        save_si_account_input,
    )?;

    // Rewrite the balance of lp-pool
    state::check_balance_account_directly(
        program_id,
        pool_index,
        coin_index,
        save_balance_lp_account_input,
    )?;
    let balance_amount;
    {
        let balance_data = save_balance_lp_account_input.data.borrow();
        balance_amount = u64::from_be_bytes(*array_ref![balance_data, 0, 8]);
    }
    if amount > balance_amount {
        return Err(MesonError::PoolBalanceNotEnough.into());
    }
    write_related_account(
        save_balance_lp_account_input,
        &(balance_amount - amount).to_be_bytes(),
    )?;

    Ok(())
}

pub fn unlock<'a, 'b>(
    program_id: &Pubkey,
    save_si_account_input: &'a AccountInfo<'b>,
    save_balance_lp_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    initiator: [u8; 20],
) -> ProgramResult {
    let amount = Utils::amount_from(encoded_swap) - Utils::fee_for_lp(encoded_swap);
    let coin_index = Utils::out_coin_index_from(encoded_swap);
    let swap_id = Utils::get_swap_id(encoded_swap, initiator);
    let locked = state::remove_locked_swap(program_id, swap_id, save_si_account_input)?;

    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp.to_le() as u64;
    if locked.until > now_timestamp {
        return Err(MesonError::SwapStillInLock.into());
    }

    state::check_balance_account_directly(
        program_id,
        locked.pool_index,
        coin_index,
        save_balance_lp_account_input,
    )?;
    let balance_amount;
    {
        let balance_data = save_balance_lp_account_input.data.borrow();
        balance_amount = u64::from_be_bytes(*array_ref![balance_data, 0, 8]);
    }
    write_related_account(
        save_balance_lp_account_input,
        &(balance_amount + amount).to_be_bytes(),
    )?;

    Ok(())
}

pub fn release<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    user_account: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    token_program_info: &'a AccountInfo<'b>,
    save_si_account_input: &'a AccountInfo<'b>,
    save_oop_account_input: &'a AccountInfo<'b>,
    save_balance_manager_account_input: &'a AccountInfo<'b>,
    ta_user_input: &'a AccountInfo<'b>,
    ta_program_input: &'a AccountInfo<'b>,
    contract_signer_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
) -> ProgramResult {
    // Check the basic conditions
    let mut amount = Utils::amount_from(encoded_swap) - Utils::fee_for_lp(encoded_swap);
    let coin_index = Utils::out_coin_index_from(encoded_swap);
    let swap_id = Utils::get_swap_id(encoded_swap, initiator);
    let locked = state::remove_locked_swap(program_id, swap_id, save_si_account_input)?;
    let recipient = user_account.key;

    // Check the time and signature
    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp.to_le() as u64;
    if locked.until < now_timestamp {
        return Err(MesonError::SwapPassedLockPeriod.into());
    }

    msg!("Signature    : {:?}", signature);
    msg!("Recipient    : {:?}", recipient);
    msg!("Initiator    : {:?}", initiator);
    // let recipient_to_eth = Utils::eth_address_from_pubkey(recipient.to_bytes());
    // Utils::check_release_signature(encoded_swap, recipient_to_eth, signature, initiator)?;

    // Deal with waiving service fee
    let waived = Utils::fee_waived(encoded_swap);
    if waived {
        state::assert_is_premium_manager(program_id, payer_account, save_oop_account_input)?;
    } else {
        let service_fee = Utils::service_fee(encoded_swap);
        amount -= service_fee;
        state::check_balance_account_directly(
            program_id,
            0,
            coin_index,
            save_balance_manager_account_input,
        )?;
        let balance_amount;
        {
            let balance_data = save_balance_manager_account_input.data.borrow();
            balance_amount = u64::from_be_bytes(*array_ref![balance_data, 0, 8]);
        }
        write_related_account(
            save_balance_manager_account_input,
            &(balance_amount + service_fee).to_be_bytes(),
        )?;
    }

    // Transfer assets to user
    let decimals = Mint::unpack(&token_mint_account.data.borrow())?.decimals;
    let (expected_contract_signer, bump_seed) =
        Pubkey::find_program_address(&[ConstantValue::CONTRACT_SIGNER], program_id);
    if expected_contract_signer != *contract_signer_account_input.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }
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
