use arrayref::array_ref;
use solana_program::{
    account_info::AccountInfo,
    program_pack::{Pack, Sealed},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program::invoke_signed,
    pubkey::Pubkey, 
    sysvar::{rent::Rent, Sysvar},
    system_instruction,
};

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


pub fn write_some_data<'a, 'b>(
    program_id: &Pubkey,
    user_account: &'a AccountInfo<'b>,
    map_account: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    data_length: usize,
    phrase: &[u8],
) -> ProgramResult {
    let (map_pda, map_bump) = Pubkey::find_program_address(&[phrase, user_account.key.as_ref()], program_id);

    assert!(
        !(map_pda != *map_account.key || !map_account.is_writable || !map_account.data_is_empty()),
        "Map PDA error!"
    );      // todo

    let rent = Rent::get()?; // Important!!
    let rent_lamports = rent.minimum_balance(data_length);

    let create_map_ix = &system_instruction::create_account(
        user_account.key,
        map_account.key,
        rent_lamports,
        data_length as u64,
        program_id,
    );

    invoke_signed(
        create_map_ix,
        &[
            user_account.clone(),
            map_account.clone(),
            system_program.clone(),
        ],
        &[&[phrase.as_ref(), user_account.key.as_ref(), &[map_bump]]],
    )?;

    Ok(())
}

