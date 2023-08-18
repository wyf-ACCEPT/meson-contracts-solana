use arrayref::array_ref;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    msg
};

pub struct ConstantValue {}

impl ConstantValue {
    const AUTHORITY_PHRASE: &[u8] = b"authority";
    const SUPPORT_COINS_PHRASE: &[u8] = b"supported_coins";
    const POSTED_SWAP_PHRASE: &[u8] = b"posted_swaps";
    const LOCKED_SWAP_PHRASE: &[u8] = b"locked_swaps";
    const POOL_OWNERS_PHRASE: &[u8] = b"pool_owners";
    const POOL_OF_AUTHORIZED_ADDR_PHRASE: &[u8] = b"pool_of_authorized_addr";
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

impl Sealed for PostedSwap {}

impl Sealed for LockedSwap {}

impl Pack for PostedSwap {
    const LEN: usize = 60;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0..8].copy_from_slice(&self.pool_index.to_le_bytes());
        dst[8..28].copy_from_slice(&self.initiator);
        dst[28..60].copy_from_slice(self.from_address.as_ref());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self {
            pool_index: u64::from_le_bytes(*array_ref![src, 0, 8]),
            initiator: *array_ref![src, 8, 20],
            from_address: Pubkey::new_from_array(*array_ref![src, 28, 32]),
        })
    }
}

impl Pack for LockedSwap {
    const LEN: usize = 48;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0..8].copy_from_slice(&self.pool_index.to_le_bytes());
        dst[8..16].copy_from_slice(&self.until.to_le_bytes());
        dst[16..48].copy_from_slice(self.recipient.as_ref());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self {
            pool_index: u64::from_le_bytes(*array_ref![src, 0, 8]),
            until: u64::from_le_bytes(*array_ref![src, 8, 8]),
            recipient: Pubkey::new_from_array(*array_ref![src, 16, 32]),
        })
    }
}

pub fn create_related_account_specified_owner<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    map_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    phrase_prefix: &[u8],
    phrase: &[u8],
    data_length: usize,
    owner: &Pubkey,
) -> ProgramResult {
    let (map_pda, map_bump) = Pubkey::find_program_address(&[phrase_prefix, phrase], program_id);
    assert!(
        !(map_pda != *map_account.key || !map_account.is_writable || !map_account.data_is_empty()),
        "Map PDA error!"
    ); // todo

    let rent = Rent::get()?; // Important!!
    let rent_lamports = rent.minimum_balance(data_length);

    let create_map_ix = &system_instruction::create_account(
        payer_account.key,
        map_account.key,
        rent_lamports,
        data_length as u64,
        owner,
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

fn create_related_account<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    map_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    phrase_prefix: &[u8],
    phrase: &[u8],
    data_length: usize,
) -> ProgramResult {
    create_related_account_specified_owner(
        program_id,
        payer_account,
        map_account,
        system_program,
        phrase_prefix,
        phrase,
        data_length,
        program_id,
    )
}

fn write_related_account<'a, 'b>(
    map_account: &'a AccountInfo<'b>,
    content: &[u8],
) -> ProgramResult {
    // // Don't need to check beacuse only this program can rewrite the value
    // let (map_pda, _) = Pubkey::find_program_address(&[phrase_prefix, phrase], program_id);
    // assert!(
    //     !(map_pda != *map_account.key || !map_account.is_writable),
    //     "Map PDA error!"
    // );

    let mut account_data = map_account.data.borrow_mut();
    account_data.copy_from_slice(content);

    Ok(())
}

pub fn init_contract<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    map_token_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
) -> ProgramResult {
    create_related_account_specified_owner(
        program_id,
        payer_account, // This is the Admin of Meson contracts!
        authority_account,
        system_program,
        ConstantValue::AUTHORITY_PHRASE,
        b"",
        0,
        payer_account.key,
    )?;
    create_related_account(
        program_id,
        payer_account,
        map_token_account,
        system_program,
        ConstantValue::SUPPORT_COINS_PHRASE,
        b"",
        32 * 16, // We support at most 16 coins.
    )?;
    Ok(())
}

pub fn transfer_premium_manager<'a, 'b>(
    program_id: &Pubkey,
    admin_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    new_admin: &'a AccountInfo<'b>,
) -> ProgramResult {
    let (authority_expected, _) = Pubkey::find_program_address(&[ConstantValue::AUTHORITY_PHRASE], program_id);
    assert!(
        !(authority_expected != *authority_account.key || !authority_account.is_writable),
        "Authority account not correct!"
    );
    assert!(admin_account.is_signer == true, "Admin should sign this transaction!");
    authority_account.assign(&new_admin.key);
    Ok(())
}

// // Named consistently with solidity contracts
// public entry fun transferPremiumManager(
//     sender: &signer,
//     new_premium_manager: address,
// ) acquires GeneralStore {
//     let store = borrow_global_mut<GeneralStore>(DEPLOYER);
//     let pool_owners = &mut store.pool_owners;
//     let old_premium_manager = table::remove(pool_owners, 0);

//     assert!(signer::address_of(sender) == old_premium_manager, EUNAUTHORIZED);

//     table::add(pool_owners, 0, new_premium_manager);
// }