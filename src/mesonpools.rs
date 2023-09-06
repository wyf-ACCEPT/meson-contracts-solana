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
    state::{self, owner_of_pool, write_related_account, ConstantValue, PostedSwap},
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
    save_balance_account_input: &'a AccountInfo<'b>,
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

    // Deal with balance of a specified pool-coin
    state::check_balance_account_directly(
        program_id,
        pool_index,
        coin_index,
        save_balance_account_input,
    )?;

    // First time to deposit -- register a data account to save the balance
    if save_balance_account_input.data_len() == 0 {
        let mut pool_coin_array = [0; 9];
        pool_coin_array[0..8].copy_from_slice(&pool_index.to_be_bytes());
        pool_coin_array[8] = coin_index;
        msg!("Create new pool-coin data account.");
        state::create_related_account(
            program_id,
            payer_account,
            save_balance_account_input,
            system_program,
            ConstantValue::SAVE_BALANCE_PHRASE,
            &pool_coin_array,
            8,
        )?;
    }

    // Update balance
    let balance_amount;
    {
        let balance_data = save_balance_account_input.data.borrow();
        balance_amount = u64::from_be_bytes(*array_ref![balance_data, 0, 8]);
    }   // See the annotation of `bond_posted_swap` for explanation of these code
    write_related_account(
        save_balance_account_input,
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
    save_balance_account_input: &'a AccountInfo<'b>,
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
    let (expected_contract_signer, bump_seed) =
        Pubkey::find_program_address(&[ConstantValue::CONTRACT_SIGNER], program_id);
    if expected_contract_signer != *contract_signer_account_input.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }

    // Withdraw token from the contract
    let decimals = Mint::unpack(&token_mint_account.data.borrow())?.decimals;
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
        let balance_data = save_balance_account_input.data.borrow();
        balance_amount = u64::from_be_bytes(*array_ref![balance_data, 0, 8]);
    }   // See the annotation of `bond_posted_swap` for explanation of these code
    
    write_related_account(
        save_balance_account_input,
        &(balance_amount - amount).to_be_bytes(),
    )?;

    Ok(())
}

// // Named consistently with solidity contracts
// public entry fun withdraw<CoinType>(sender: &signer, amount: u64, pool_index: u64) {
//     let sender_addr = signer::address_of(sender);
//     assert!(pool_index == MesonStates::pool_index_if_owner(sender_addr), EPOOL_INDEX_MISMATCH);
//     let coins = MesonStates::coins_from_pool<CoinType>(pool_index, amount);
//     coin::deposit<CoinType>(sender_addr, coins);
// }
