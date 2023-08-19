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
};

use crate::error::MesonError;

pub struct ConstantValue {}

impl ConstantValue {
    const AUTHORITY_PHRASE: &[u8] = b"authority";
    const SUPPORT_COINS_PHRASE: &[u8] = b"supported_coins";
    // const POSTED_SWAP_PHRASE: &[u8] = b"posted_swaps";
    // const LOCKED_SWAP_PHRASE: &[u8] = b"locked_swaps";
    // const POOL_OWNERS_PHRASE: &[u8] = b"pool_owners";
    // const POOL_OF_AUTHORIZED_ADDR_PHRASE: &[u8] = b"pool_of_authorized_addr";
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
    // let (map_pda, _) = Pubkey::find_program_address(&[phrase_prefix, phrase], program_id);
    // assert!(
    //     !(map_pda != *map_account.key || !map_account.is_writable),
    //     "Map PDA error!"
    // );

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

pub fn init_contract<'a, 'b>(
    program_id: &Pubkey,
    payer_account: &'a AccountInfo<'b>,
    map_token_account: &'a AccountInfo<'b>,
    authority_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
) -> ProgramResult {
    create_related_account(
        program_id,
        payer_account, // This is the Admin of Meson contracts!
        authority_account,
        system_program,
        ConstantValue::AUTHORITY_PHRASE,
        b"",
        32, // To save the admin address
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

// public entry fun addSupportToken<CoinType>(
//     sender: &signer,
//     coin_index: u8,
// ) acquires GeneralStore {
//     let sender_addr = signer::address_of(sender);
//     assert!(sender_addr == DEPLOYER, ENOT_DEPLOYER);

//     let store = borrow_global_mut<GeneralStore>(DEPLOYER);
//     let supported_coins = &mut store.supported_coins;
//     if (table::contains(supported_coins, coin_index)) {
//         table::remove(supported_coins, coin_index);
//     };
//     table::add(supported_coins, coin_index, type_info::type_of<CoinType>());

//     let coin_store = StoreForCoin<CoinType> {
//         in_pool_coins: table::new<u64, Coin<CoinType>>(),
//         pending_coins: table::new<vector<u8>, Coin<CoinType>>(),
//     };
//     move_to<StoreForCoin<CoinType>>(sender, coin_store);
// }
