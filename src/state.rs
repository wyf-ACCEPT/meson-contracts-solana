use arrayref::array_ref;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::error::MesonError;

pub struct ConstantValue {}

impl ConstantValue {
    pub const AUTHORITY_PHRASE: &[u8] = b"authority";
    pub const SUPPORT_COINS_PHRASE: &[u8] = b"supported_coins";
    pub const SAVE_POSTED_SWAP_PHRASE: &[u8] = b"posted_swaps";
    pub const SAVE_LOCKED_SWAP_PHRASE: &[u8] = b"locked_swaps";
    pub const SAVE_OWNER_OF_POOLS_PHRASE: &[u8] = b"pool_owners";
    pub const SAVE_POOL_OF_AUTHORIZED_ADDR_PHRASE: &[u8] = b"pool_of_authorized_addr";
}

pub struct PostedSwap {
    pool_index: u64,
    initiator: [u8; 20],
    from_address: Pubkey,
}

pub struct LockedSwap {
    pool_index: u64,
    until: u64,
    recipient: Pubkey,
}

// impl Sealed for PostedSwap {}

// impl Sealed for LockedSwap {}

// impl Pack for PostedSwap {
//     const LEN: usize = 60;

//     fn pack_into_slice(&self, dst: &mut [u8]) {
//         dst[0..8].copy_from_slice(&self.pool_index.to_be_bytes());
//         dst[8..28].copy_from_slice(&self.initiator);
//         dst[28..60].copy_from_slice(self.from_address.as_ref());
//     }

//     fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
//         Ok(Self {
//             pool_index: u64::from_be_bytes(*array_ref![src, 0, 8]),
//             initiator: *array_ref![src, 8, 20],
//             from_address: Pubkey::new_from_array(*array_ref![src, 28, 32]),
//         })
//     }
// }

impl PostedSwap {
    fn pack(&self) -> [u8; 60] {
        let mut dst = [0; 60];
        dst[0..8].copy_from_slice(&self.pool_index.to_be_bytes());
        dst[8..28].copy_from_slice(&self.initiator);
        dst[28..60].copy_from_slice(self.from_address.as_ref());
        dst
    }

    fn unpack(src: [u8; 60]) -> Self {
        Self {
            pool_index: u64::from_be_bytes(*array_ref![src, 0, 8]),
            initiator: *array_ref![src, 8, 20],
            from_address: Pubkey::new_from_array(*array_ref![src, 28, 32]),
        }
    }
}

// impl Pack for LockedSwap {
//     const LEN: usize = 48;

//     fn pack_into_slice(&self, dst: &mut [u8]) {
//         dst[0..8].copy_from_slice(&self.pool_index.to_be_bytes());
//         dst[8..16].copy_from_slice(&self.until.to_be_bytes());
//         dst[16..48].copy_from_slice(self.recipient.as_ref());
//     }

//     fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
//         Ok(Self {
//             pool_index: u64::from_be_bytes(*array_ref![src, 0, 8]),
//             until: u64::from_be_bytes(*array_ref![src, 8, 8]),
//             recipient: Pubkey::new_from_array(*array_ref![src, 16, 32]),
//         })
//     }
// }

impl LockedSwap {
    fn pack(&self) -> [u8; 48] {
        let mut dst = [0; 48];
        dst[0..8].copy_from_slice(&self.pool_index.to_be_bytes());
        dst[8..16].copy_from_slice(&self.until.to_be_bytes());
        dst[16..48].copy_from_slice(self.recipient.as_ref());
        dst
    }

    fn unpack(src: [u8; 48]) -> Self {
        Self {
            pool_index: u64::from_be_bytes(*array_ref![src, 0, 8]),
            until: u64::from_be_bytes(*array_ref![src, 8, 8]),
            recipient: Pubkey::new_from_array(*array_ref![src, 16, 32]),
        }
    }
}

fn create_related_account<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    map_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    phrase_prefix: &[u8],
    phrase: &[u8],
    data_length: usize,
) -> ProgramResult {
    let (map_pda, map_bump) = Pubkey::find_program_address(&[phrase_prefix, phrase], program_id);

    if map_pda != *map_account.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }
    if !map_account.is_writable {
        return Err(MesonError::PdaAccountNotWritable.into());
    }
    if !map_account.data_is_empty() {
        return Err(MesonError::PdaAccountAlreadyCreated.into());
    }

    let rent = Rent::get()?; // Important!!
    let rent_lamports = rent.minimum_balance(data_length);

    let create_map_ix = &system_instruction::create_account(
        payer_account.key,
        map_account.key,
        rent_lamports,
        data_length as u64,
        program_id,
    );

    invoke_signed(
        create_map_ix,
        &[
            payer_account.clone(),
            map_account.clone(),
            system_program.clone(),
        ],
        &[&[phrase_prefix.as_ref(), phrase.as_ref(), &[map_bump]]],
    )?;

    Ok(())
}

fn write_related_account<'a, 'b>(
    map_account: &'a AccountInfo<'b>,
    content: &[u8],
) -> ProgramResult {
    // // Don't need to check beacuse only this program can rewrite the value
    let mut account_data = map_account.data.borrow_mut();
    account_data.copy_from_slice(content);
    Ok(())
}

fn check_admin<'a, 'b>(
    program_id: &Pubkey,
    admin_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
) -> ProgramResult {
    let (authority_expected, _) =
        Pubkey::find_program_address(&[ConstantValue::AUTHORITY_PHRASE], program_id);
    if authority_expected != *authority_account.key {
        return Err(MesonError::PdaAccountMismatch.into());
    }
    if !admin_account.is_signer || (*authority_account.data.borrow() != admin_account.key.as_ref())
    {
        return Err(MesonError::AdminNotSigner.into());
    }
    Ok(())
}

fn check_pda_match<'a, 'b>(
    account_input: &'a AccountInfo<'b>,
    pubkey_expected: Pubkey,
) -> ProgramResult {
    if *account_input.key != pubkey_expected {
        Err(MesonError::PdaAccountMismatch.into())
    } else {
        Ok(())
    }
}

/// save_poaa_account_input: `poaa` -> `pool of authorized address`
/// save_oop_account_input : `oop` -> `owner of pool`
/// save_ps_account_input  : `ps` -> `posted swap`
/// save_si_account_input  : `si` -> `swap id`

fn check_poaa_directly<'a, 'b>(
    program_id: &Pubkey,
    authorized_account_input: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    let (save_poaa_pubkey_expected, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_POOL_OF_AUTHORIZED_ADDR_PHRASE,
            authorized_account_input.key.as_ref(),
        ],
        program_id,
    );
    check_pda_match(save_poaa_account_input, save_poaa_pubkey_expected)
}

fn check_oop_directly<'a, 'b>(
    program_id: &Pubkey,
    pool_index: u64,
    save_oop_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    let (save_oop_pubkey_expected, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_OWNER_OF_POOLS_PHRASE,
            &pool_index.to_be_bytes(),
        ],
        program_id,
    );
    check_pda_match(save_oop_account_input, save_oop_pubkey_expected)
}

fn check_postedswap_directly<'a, 'b>(
    program_id: &Pubkey,
    encoded_swap: [u8; 32],
    save_ps_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    let (save_ps_pubkey_expected, _) = Pubkey::find_program_address(
        &[ConstantValue::SAVE_POSTED_SWAP_PHRASE, &encoded_swap],
        program_id,
    );
    check_pda_match(save_ps_account_input, save_ps_pubkey_expected)
}

fn check_lockedswap_directly<'a, 'b>(
    program_id: &Pubkey,
    swap_id: [u8; 32],
    save_si_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    let (save_si_pubkey_expected, _) = Pubkey::find_program_address(
        &[ConstantValue::SAVE_LOCKED_SWAP_PHRASE, &swap_id],
        program_id,
    );
    check_pda_match(save_si_account_input, save_si_pubkey_expected)
}

pub fn init_contract<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    map_token_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    save_poaa_account_input_admin: &'a AccountInfo<'b>,
    save_oop_account_input_admin: &'a AccountInfo<'b>,
) -> ProgramResult {
    create_related_account(
        program_id,
        payer_account, // This is the Admin of Meson contracts!
        authority_account,
        system_program,
        ConstantValue::AUTHORITY_PHRASE,
        b"",
        32, // To save the admin address (also the premium)
    )?;
    write_related_account(authority_account, payer_account.key.as_ref())?;
    create_related_account(
        program_id,
        payer_account,
        map_token_account,
        system_program,
        ConstantValue::SUPPORT_COINS_PHRASE,
        b"",
        32 * 256, // We support at most 256 coins.
    )?;
    check_oop_directly(program_id, 0, save_oop_account_input_admin)?;
    check_poaa_directly(program_id, payer_account, save_poaa_account_input_admin)?;
    register_pool_index(
        program_id,
        payer_account,
        system_program,
        0,
        payer_account,
        save_poaa_account_input_admin,
        save_oop_account_input_admin,
    )?;
    Ok(())
}

pub fn transfer_admin<'a, 'b>(
    program_id: &Pubkey,
    admin_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    new_admin: &'a AccountInfo<'b>,
) -> ProgramResult {
    if !authority_account.is_writable {
        return Err(MesonError::PdaAccountNotWritable.into());
    }
    check_admin(program_id, admin_account, authority_account)?;
    write_related_account(authority_account, new_admin.key.as_ref())?;
    Ok(())
}

// transferPremiumManager todo()

pub fn add_support_token<'a, 'b>(
    program_id: &Pubkey,
    admin_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    map_token_account: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    coin_index: u8,
) -> ProgramResult {
    check_admin(program_id, admin_account, authority_account)?;
    let mut map_token_account_data = map_token_account.data.borrow_mut();
    let start_u8_index = coin_index as usize * 32;
    for i in 0..32 {
        map_token_account_data[start_u8_index + i] = token_mint_account.key.as_ref()[i]
    }
    Ok(())
}

pub fn token_mint_account_for_index<'a, 'b>(
    map_token_account: &'a AccountInfo<'b>,
    coin_index: u8,
) -> Pubkey {
    let map_token_account_data = map_token_account.data.borrow();
    let start_u8_index = coin_index as usize * 32;
    Pubkey::from(*array_ref![map_token_account_data, start_u8_index, 32])
}

pub fn match_token_address<'a, 'b>(
    map_token_account: &'a AccountInfo<'b>,
    token_mint_account: &'a AccountInfo<'b>,
    coin_index: u8,
) -> ProgramResult {
    let token_addr_1 = *token_mint_account.key;
    let token_addr_2 = token_mint_account_for_index(map_token_account, coin_index);
    if token_addr_1 != token_addr_2 {
        Err(MesonError::CoinTypeMismatch.into())
    } else {
        Ok(())
    }
}

pub fn owner_of_pool<'a, 'b>(
    program_id: &Pubkey,
    pool_index: u64,
    save_oop_account_input: &'a AccountInfo<'b>, // place to save the authorized address of a specified pool index
) -> Result<Pubkey, ProgramError> {
    check_oop_directly(program_id, pool_index, save_oop_account_input)?;

    let owner_pubkey_data = save_oop_account_input.data.borrow();
    Ok(Pubkey::from(*array_ref![owner_pubkey_data, 0, 32]))
}

pub fn assert_is_premium_manager<'a, 'b>(
    program_id: &Pubkey,
    premium_manager_account: &'a AccountInfo<'b>,
    save_oop_account_input: &'a AccountInfo<'b>, // place to save premium manager's pool index
) -> ProgramResult {
    if *premium_manager_account.key != owner_of_pool(program_id, 0, save_oop_account_input)? {
        Err(MesonError::OnlyPremiumManager.into())
    } else {
        Ok(())
    }
}

pub fn pool_index_of<'a, 'b>(
    program_id: &Pubkey,
    authorized_account_input: &'a AccountInfo<'b>, // the address itself
    save_poaa_account_input: &'a AccountInfo<'b>,  // place to save address's pool index
) -> Result<u64, ProgramError> {
    check_poaa_directly(
        program_id,
        authorized_account_input,
        save_poaa_account_input,
    )?;

    let account_data = save_poaa_account_input.data.borrow();
    Ok(u64::from_be_bytes(*array_ref![account_data, 0, 8]))
}

pub fn pool_index_if_owner<'a, 'b>(
    program_id: &Pubkey,
    authorized_account_input: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
    save_oop_account_input: &'a AccountInfo<'b>,
) -> Result<u64, ProgramError> {
    let pool_index = pool_index_of(
        program_id,
        authorized_account_input,
        save_poaa_account_input,
    )?;
    if *authorized_account_input.key
        != owner_of_pool(program_id, pool_index, save_oop_account_input)?
    {
        Err(MesonError::PoolNotPoolOwner.into())
    } else {
        Ok(pool_index)
    }
}

pub fn register_pool_index<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    pool_index: u64,
    authorized_account_input: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
    save_oop_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    // Don't need to check pool_index!=0, because it's already been created (the pool_index=0 related pda account)

    // Check PDA address
    let authorized_pubkey = *authorized_account_input.key;
    check_oop_directly(program_id, pool_index, save_oop_account_input)?;
    check_poaa_directly(
        program_id,
        authorized_account_input,
        save_poaa_account_input,
    )?;

    // create `save_oop_account` to save `pool_index(u64) -> owner_pubkey/authorized_pubkey(Pubkey)`
    create_related_account(
        program_id,
        payer_account,
        save_oop_account_input,
        system_program,
        ConstantValue::SAVE_OWNER_OF_POOLS_PHRASE,
        &pool_index.to_be_bytes(),
        32,
    )?;
    write_related_account(save_oop_account_input, authorized_pubkey.as_ref())?;

    // create `save_poaa_account` to save `owner_pubkey/authorized_pubkey(Pubkey) -> pool_index(u64)`
    create_related_account(
        program_id,
        payer_account,
        save_poaa_account_input,
        system_program,
        ConstantValue::SAVE_POOL_OF_AUTHORIZED_ADDR_PHRASE,
        authorized_pubkey.as_ref(),
        8,
    )?;
    write_related_account(save_poaa_account_input, &pool_index.to_be_bytes())?;

    Ok(())
}

pub fn add_authorized<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    pool_index: u64,
    authorized_account_input: &'a AccountInfo<'b>,
    save_poaa_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    if pool_index == 0 {
        return Err(MesonError::PoolIndexCannotBeZero.into());
    }
    check_poaa_directly(
        program_id,
        authorized_account_input,
        save_poaa_account_input,
    )?;

    create_related_account(
        program_id,
        payer_account,
        save_poaa_account_input,
        system_program,
        ConstantValue::SAVE_POOL_OF_AUTHORIZED_ADDR_PHRASE,
        authorized_account_input.key.as_ref(),
        8,
    )?;
    write_related_account(save_poaa_account_input, &pool_index.to_be_bytes())?;

    Ok(())
}

// remove_authorized todo()

// transfer_pool_owner todo()

pub fn add_posted_swap<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    encoded_swap: [u8; 32],
    pool_index: u64,
    initiator: [u8; 20],
    from_address: Pubkey,
    save_ps_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    check_postedswap_directly(program_id, encoded_swap, save_ps_account_input)?; // needed?
    create_related_account(
        program_id,
        payer_account,
        save_ps_account_input,
        system_program,
        ConstantValue::SAVE_POSTED_SWAP_PHRASE,
        &encoded_swap,
        60,
    )?;
    write_related_account(
        save_ps_account_input,
        &(PostedSwap {
            pool_index,
            initiator,
            from_address,
        })
        .pack(),
    )?;
    Ok(())
}

pub fn bond_posted_swap<'a, 'b>(
    program_id: &Pubkey,
    encoded_swap: [u8; 32],
    pool_index: u64,
    save_ps_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    check_postedswap_directly(program_id, encoded_swap, save_ps_account_input)?;
    let temp_data = save_ps_account_input.data.borrow();
    let mut posted = PostedSwap::unpack(*array_ref![temp_data, 0, 60]);
    if posted.from_address == Pubkey::from([0; 32]) {
        Err(MesonError::SwapNotExists.into())
    } else if posted.pool_index != 0 {
        Err(MesonError::SwapBondedToOthers.into())
    } else {
        posted.pool_index = pool_index;
        write_related_account(save_ps_account_input, &posted.pack())
    }
}

// remove_posted_swap todo()

pub fn add_locked_swap<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    swap_id: [u8; 32],
    pool_index: u64,
    until: u64,
    recipient: Pubkey,
    save_si_account_input: &'a AccountInfo<'b>,
) -> ProgramResult {
    check_lockedswap_directly(program_id, swap_id, save_si_account_input)?;
    create_related_account(
        program_id,
        payer_account,
        save_si_account_input,
        system_program,
        ConstantValue::SAVE_LOCKED_SWAP_PHRASE,
        &swap_id,
        48,
    )?;
    write_related_account(
        save_si_account_input,
        &(LockedSwap {
            pool_index,
            until,
            recipient,
        })
        .pack(),
    )?;
    Ok(())
}
