use arrayref::{array_ref, array_refs};

use crate::error::MesonError;

#[derive(Clone, Debug, PartialEq)]
pub enum MesonInstruction {
    /// The admin(deployer) must call this init function first!
    /// Account data:
    /// 1. payer_account: the contract deployer, also the admin
    /// 2. system_program: that is `11111111111111111111111111111111`
    /// 3. authority_account: to save the address of admin
    /// 4. map_token_account: to save the supported coin list
    InitContract,

    /// Account data:
    /// 1. admin_account: the admin account, must be a signer
    /// 2. authority_account
    /// 3. new_admin: the new admin address
    TransferPremiumManager,

    /// Account data:
    /// 1. admin_account
    /// 2. authority_account
    /// 3. map_token_account
    /// 4. token_mint_account: the mint address of the coin to add to support list
    AddSupportToken { coin_index: u8 },

    /// Account data:
    /// 1. payer_account
    /// 2. system_program
    /// 3. authorized_account: the address to add to LP pools
    /// 4. save_poaa_account_input: the data account to save `authorized address -> pool index` pair (8-bytes long)
    /// 5. save_oop_account_input: the data account to save `pool index -> authorized address` pair (32-bytes long)
    RegisterPool { pool_index: u64 },

    /// Account data:
    /// 1. payer_account
    /// 2. system_program
    /// 3. user_account: the user who wants to swap
    /// 4. token_mint_account
    /// 5. save_map_token_account: same as `map_token_account`
    /// 6. save_ps_account_input: the data account to save `encoded -> postedSwap` pair (60-bytes)
    /// 7. ta_user_input: the token account for the user
    /// 8. ta_program_input: the token account for the program
    PostSwap {
        encoded_swap: [u8; 32],
        signature: [u8; 64],
        initiator: [u8; 20],
        pool_index: u64,
    },
}

impl MesonInstruction {
    /// Unpacks a byte buffer into a [TokenInstruction](enum.TokenInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, MesonError> {
        let (&tag, rest) = input.split_first().ok_or(MesonError::InvalidInstruction)?;
        Ok(match tag {
            0 => MesonInstruction::InitContract,

            1 => MesonInstruction::TransferPremiumManager,

            2 => MesonInstruction::AddSupportToken {
                coin_index: rest[0],
            },

            3 => MesonInstruction::RegisterPool {
                pool_index: u64::from_be_bytes(*array_ref![rest, 0, 8]),
            },

            4 => {
                let rest_fix = *array_ref![rest, 0, 124];
                let (encoded_swap_ref, signature_ref, initiator_ref, pool_index_ref) =
                    array_refs![&rest_fix, 32, 64, 20, 8];
                MesonInstruction::PostSwap {
                    encoded_swap: *encoded_swap_ref,
                    signature: *signature_ref,
                    initiator: *initiator_ref,
                    pool_index: u64::from_be_bytes(*pool_index_ref),
                }
            }

            _ => return Err(MesonError::InvalidInstruction),
        })
    }
}
