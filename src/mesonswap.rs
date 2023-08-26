use arrayref::array_ref;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{error::MesonError, state, utils::Utils};

pub fn post_swap<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    save_map_token_account: &'a AccountInfo<'b>,
    save_ps_account_input: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
    pool_index: u64,
) -> ProgramResult {
    Utils::for_initial_chain(encoded_swap)?;
    state::match_token_address(
        save_map_token_account,
        token_mint_account,
        Utils::in_coin_index_from(encoded_swap),
    )?;

    let amount = Utils::amount_from(encoded_swap);
    Utils::amount_within_max(amount)?;

    //     // Assertion about time-lock.
    //     let delta = MesonHelpers::expire_ts_from(encoded_swap) - timestamp::now_seconds();
    //     assert!(delta > MesonHelpers::get_MIN_BOND_TIME_PERIOD(), ESWAP_EXPIRE_TOO_EARLY);
    //     assert!(delta < MesonHelpers::get_MAX_BOND_TIME_PERIOD(), ESWAP_EXPIRE_TOO_LATE);
    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp.to_be() as u64;
    msg!("{}", now_timestamp);
    // todo!();

    Utils::check_request_signature(encoded_swap, signature, initiator)?;
    state::add_posted_swap(
        program_id,
        payer_account,
        system_program,
        encoded_swap,
        pool_index,
        initiator,
        *payer_account.key,
        save_ps_account_input,
    )?;

    Ok(())
}

// public entry fun postSwap<CoinType>(
//     sender: &signer,
//     encoded_swap: vector<u8>,
//     signature: vector<u8>, // must be signed by `initiator`
//     initiator: vector<u8>, // an eth address of (20 bytes), the signer to sign for release
//     pool_index: u64,
// ) {
//     MesonHelpers::check_request_signature(encoded_swap, signature, initiator);

//     vector::push_back(&mut encoded_swap, 0xff); // so it cannot be identical to a swap_id
//     MesonStates::add_posted_swap(encoded_swap, pool_index, initiator, signer::address_of(sender));
//     let coins = coin::withdraw<CoinType>(sender, amount);
//     MesonStates::coins_to_pending(encoded_swap, coins);
// }
