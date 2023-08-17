use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::state::write_some_data;

pub struct Processor {}
impl Processor {
    pub fn process<'a>(program_id: &Pubkey, accounts: &[AccountInfo], _input: &[u8]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer_account = next_account_info(account_info_iter)?;
        let map_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        write_some_data(program_id, payer_account, map_account, system_program, 2, b"hello", b"world")?;
        Ok(())
    }
}
