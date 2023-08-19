use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

// use crate::state::{create_related_account, write_related_account};
use crate::{
    instruction::MesonInstruction,
    state::{init_contract, transfer_admin},
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
        }
    }

    fn process_init_contract(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_account = next_account_info(account_info_iter)?;
        let authority_account = next_account_info(account_info_iter)?;
        let map_token_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        init_contract(
            program_id,
            payer_account,
            map_token_account,
            authority_account,
            system_program,
        )
    }

    fn process_transfer_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let admin_account = next_account_info(account_info_iter)?;
        let authority_account = next_account_info(account_info_iter)?;
        let new_admin = next_account_info(account_info_iter)?;

        transfer_admin(program_id, admin_account, authority_account, new_admin)
    }
}
