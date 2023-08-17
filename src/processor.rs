use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

// use crate::state::{create_related_account, write_related_account};
use crate::state::init_contract;

pub struct Processor {}
impl Processor {
    pub fn process<'a>(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        _input: &[u8],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer_account = next_account_info(account_info_iter)?;
        // let map_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        let map_token_account = next_account_info(account_info_iter)?;
        let authority_account = next_account_info(account_info_iter)?;
        // create_related_account(
        //     program_id,
        //     payer_account,
        //     map_account,
        //     system_program,
        //     b"hello",
        //     b"world",
        //     2,
        // )?;
        // write_related_account(
        //     map_account,
        //     &[1 as u8, 2],
        // )?;

        init_contract(
            program_id,
            payer_account,
            map_token_account,
            authority_account,
            system_program,
        )?;

        Ok(())
    }
}
