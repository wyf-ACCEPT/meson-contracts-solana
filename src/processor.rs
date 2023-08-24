use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

// use crate::state::{create_related_account, write_related_account};
use crate::{
    instruction::MesonInstruction,
    state::{add_support_token, init_contract, register_pool_index, transfer_admin},
};

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = MesonInstruction::unpack(input)?;
        match instruction {
            MesonInstruction::InitContract => Self::process_init_contract(program_id, accounts),
            MesonInstruction::TransferPremiumManager => {
                Self::process_transfer_admin(program_id, accounts)
            }
            MesonInstruction::AddSupportToken { coin_index } => {
                Self::process_add_support_token(program_id, accounts, coin_index)
            }
            MesonInstruction::RigisterPool { pool_index } => {
                Self::process_register_pool(program_id, accounts, pool_index)
            }
        }
    }

    fn process_init_contract(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let authority_account = next_account_info(account_info_iter)?;
        let map_token_account = next_account_info(account_info_iter)?;
        let save_poaa_account_input_admin = next_account_info(account_info_iter)?;
        let save_oop_account_input_admin = next_account_info(account_info_iter)?;

        init_contract(
            program_id,
            payer_account,
            system_program,
            map_token_account,
            authority_account,
            save_poaa_account_input_admin,
            save_oop_account_input_admin,
        )
    }

    fn process_transfer_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let admin_account = next_account_info(account_info_iter)?;
        let authority_account = next_account_info(account_info_iter)?;
        let new_admin = next_account_info(account_info_iter)?;

        transfer_admin(program_id, admin_account, authority_account, new_admin)
    }

    fn process_add_support_token(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        coin_index: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let admin_account = next_account_info(account_info_iter)?;
        let authority_account = next_account_info(account_info_iter)?;
        let map_token_account = next_account_info(account_info_iter)?;
        let token_mint_account = next_account_info(account_info_iter)?;

        add_support_token(
            program_id,
            admin_account,
            authority_account,
            map_token_account,
            token_mint_account,
            coin_index,
        )
    }

    fn process_register_pool(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        pool_index: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let authorized_account = next_account_info(account_info_iter)?;
        let save_poaa_account_input = next_account_info(account_info_iter)?;
        let save_oop_account_input = next_account_info(account_info_iter)?;

        register_pool_index(
            program_id,
            payer_account,
            system_program,
            pool_index,
            authorized_account,
            save_poaa_account_input,
            save_oop_account_input,
        )
    }
}
