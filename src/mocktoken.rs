// lib.rs

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

entrypoint!(process_instruction);

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {

    msg!(&spl_token::id().to_string());

    let (authorize_key, bump_seed) = Pubkey::find_program_address(&[b"token_test"], program_id);
    let (mint, _) = Pubkey::find_program_address(&[b"hi"], program_id);

    invoke_signed(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(), 
            &mint,
            &authorize_key,
            None,
            6
        )
        .unwrap(),
        &[],
        &[&[b"token_test", &[bump_seed]]],
    )?;

    msg!(format!("Mint: {}", mint).as_str());
    msg!(format!("Auth: {}", authorize_key).as_str());
    msg!(format!("Prgm: {}", program_id).as_str());
    msg!(format!("Seed: {}", bump_seed).as_str());

    // invoke_signed(
    //     &spl_token::instruction::mint_to(
    //         &spl_token::id(), 
    //         &mint, 
    //         account_pubkey, owner_pubkey, signer_pubkeys, amount), account_infos, signers_seeds)

    // // Log a string
    // msg!("static string");

    // msg!(std::str::from_utf8(instruction_data).unwrap());

    // // Log a public key
    // program_id.log();
    // println!("{:?} {:?}", accounts, instruction_data);

    // // Log unix timestamp
    // let clock = Clock::get()?;
    // let now_ts = clock.unix_timestamp.to_string();
    // msg!(now_ts.as_str());
    Ok(())
}
