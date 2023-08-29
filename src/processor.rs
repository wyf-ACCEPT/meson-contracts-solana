use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

// use crate::state::{create_related_account, write_related_account};
use crate::{
    instruction::MesonInstruction,
    mesonswap::post_swap,
    state::{add_support_token, init_contract, register_pool_index, transfer_admin},
};

pub struct Processor {}
impl Processor {
    const TOKEN_PROGRAM_ID: [u8; 32] = [6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169];

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
            MesonInstruction::RegisterPool { pool_index } => {
                Self::process_register_pool(program_id, accounts, pool_index)
            }
            MesonInstruction::PostSwap {
                encoded_swap,
                signature,
                initiator,
                pool_index,
            } => Self::process_post_swap(
                program_id,
                accounts,
                encoded_swap,
                signature,
                initiator,
                pool_index,
            ),
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

    fn process_post_swap(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        encoded_swap: [u8; 32],
        signature: [u8; 64],
        initiator: [u8; 20],
        pool_index: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let token_mint_account = next_account_info(account_info_iter)?;
        let save_map_token_account = next_account_info(account_info_iter)?;
        let save_ps_account_input = next_account_info(account_info_iter)?;
        let ta_user_input = next_account_info(account_info_iter)?;
        let ta_program_input = next_account_info(account_info_iter)?;

        post_swap(
            program_id,
            &Pubkey::from(Self::TOKEN_PROGRAM_ID),
            payer_account,
            system_program,
            user_account,
            token_mint_account,
            save_map_token_account,
            save_ps_account_input,
            ta_user_input,
            ta_program_input,
            encoded_swap,
            signature,
            initiator,
            pool_index,
        )
    }
}
