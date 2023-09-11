use arrayref::{array_ref, array_refs};
use solana_program::pubkey::Pubkey;

use crate::error::MesonError;

#[derive(Clone, Debug, PartialEq)]
pub enum MesonInstruction {
    // The admin(deployer) must call this init function first!
    /// [0]
    /// 0. payer_account: the contract deployer, also the admin
    /// 1. system_program: that is `11111111111111111111111111111111`
    /// 2. authority_account: to save the address of admin
    /// 3. save_token_list_account: to save the supported coin list
    InitContract,

    /// [1]
    /// 0. admin_account: the admin account, must be a signer
    /// 1. authority_account
    /// 2. new_admin: the new admin address
    TransferPremiumManager,

    /// [2]
    /// 0. admin_account
    /// 1. authority_account
    /// 2. save_token_list_account
    /// 3. token_mint_account: the mint address of the coin to add to support list
    AddSupportToken { coin_index: u8 },

    /// [3]
    /// 0. payer_account
    /// 1. system_program
    /// 2. authorized_account: the address to add to LP pools
    /// 3. save_poaa_account_input: the data account to save `authorized address -> pool index` pair (8-bytes long)
    /// 4. save_oop_account_input: the data account to save `pool index -> authorized address` pair (32-bytes long)
    RegisterPool { pool_index: u64 },

    /// [4]
    /// 0. payer_account
    /// 1. system_program
    /// 2. user_account: the user who wants to swap
    /// 3. token_mint_account
    /// 4. token_program_info: that is "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// 5. save_token_list_account
    /// 6. save_ps_account_input: the data account to save `encoded -> postedSwap` pair (60-bytes)
    /// 7. ta_user_input: the token account for the user
    /// 8. ta_program_input: the token account for the program
    PostSwap {
        encoded_swap: [u8; 32],
        signature: [u8; 64],
        initiator: [u8; 20],
        pool_index: u64,
    },

    /// [5]
    /// 0. sender_account: same as `authorized_account`
    /// 1. save_poaa_account_input
    /// 2. save_ps_account_input
    BondSwap {
        encoded_swap: [u8; 32],
        pool_index: u64,
    },

    /// [6]
    /// 0. token_mint_account
    /// 1. token_program_info
    /// 2. save_ps_account_input
    /// 3. ta_user_input
    /// 4. ta_program_input
    /// 5. contract_signer_account_input
    CancelSwap { encoded_swap: [u8; 32] },

    /// [7]
    /// 0. token_mint_account
    /// 1. token_program_info
    /// 2. save_ps_account_input
    /// 3. save_oop_account_input
    /// 4. ta_lp_input: the token account for lp (the owner of pool_index)
    /// 5. ta_program_input
    /// 6. contract_signer_account_input: the account as a singer of the program contract
    ExecuteSwap {
        encoded_swap: [u8; 32],
        signature: [u8; 64],
        recipient: [u8; 20],
    },

    /// [8]
    /// 0. payer_account
    /// 1. system_program
    /// 2. authorized_account_input: the address to add to LP pools
    /// 3. token_mint_account
    /// 4. token_program_info
    /// 5. save_token_list_account
    /// 6. save_poaa_account_input
    /// 7. save_balance_lp_account_input: the data account to save `pool_index & coin_index -> balance` pair (8-bytes long to save u64 balance)
    /// 8. ta_lp_input
    /// 9. ta_program_input
    DepositToPool {
        pool_index: u64,
        coin_index: u8,
        amount: u64,
    },

    /// [9]
    /// 0. authorized_account_input
    /// 1. token_mint_account
    /// 2. token_program_info
    /// 3. save_token_list_account
    /// 4. save_poaa_account_input
    /// 5. save_balance_lp_account_input
    /// 6. ta_lp_input
    /// 7. ta_program_input
    /// 8. contract_signer_account_input
    WithdrawFromPool {
        pool_index: u64,
        coin_index: u8,
        amount: u64,
    },

    /// [10]
    /// 0. payer_account
    /// 1. system_program
    /// 2. authorized_account_input
    /// 3. token_mint_account
    /// 4. save_si_account_input: the data account to save `swapId -> lockedSwap` pair (48-bytes)
    /// 5. save_token_list_account
    /// 6. save_poaa_account_input
    /// 7. save_balance_lp_account_input
    Lock {
        encoded_swap: [u8; 32],
        signature: [u8; 64],
        initiator: [u8; 20],
        recipient: Pubkey,
    },

    /// [11]
    /// 0. save_si_account_input
    /// 1. save_balance_lp_account_input
    Unlock {
        encoded_swap: [u8; 32],
        initiator: [u8; 20],
    },

    /// [12]
    /// 0. payer_account
    /// 1. system_program
    /// 2. token_mint_account
    /// 3. token_program_info
    /// 4. save_si_account_input
    /// 5. save_oop_admin_account_input: the data account to save `pool_index=0(the manager) -> authorized address` pair (32-bytes long)
    /// 6. save_balance_manager_account_input: the data account to save `pool_index=0(the manager) & coin_index -> balance` pair (8-bytes long to save u64 balance)
    /// 7. ta_user_input
    /// 8. ta_program_input
    /// 9. contract_signer_account_input
    Release {
        encoded_swap: [u8; 32],
        signature: [u8; 64],
        initiator: [u8; 20],
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

            5 => {
                let rest_fix = *array_ref![rest, 0, 40];
                let (encoded_swap_ref, pool_index_ref) = array_refs![&rest_fix, 32, 8];
                MesonInstruction::BondSwap {
                    encoded_swap: *encoded_swap_ref,
                    pool_index: u64::from_be_bytes(*pool_index_ref),
                }
            }

            6 => {
                let encoded_swap = *array_ref![rest, 0, 32];
                MesonInstruction::CancelSwap { encoded_swap }
            }

            7 => {
                let rest_fix = *array_ref![rest, 0, 116];
                let (encoded_swap_ref, signature_ref, recipient_ref) =
                    array_refs![&rest_fix, 32, 64, 20];
                MesonInstruction::ExecuteSwap {
                    encoded_swap: *encoded_swap_ref,
                    signature: *signature_ref,
                    recipient: *recipient_ref,
                }
            }

            8 => {
                let rest_fix = *array_ref![rest, 0, 17];
                let (pool_index_ref, coin_index_ref, amount_ref) = array_refs![&rest_fix, 8, 1, 8];
                MesonInstruction::DepositToPool {
                    pool_index: u64::from_be_bytes(*pool_index_ref),
                    coin_index: coin_index_ref[0],
                    amount: u64::from_be_bytes(*amount_ref),
                }
            }

            9 => {
                let rest_fix = *array_ref![rest, 0, 17];
                let (pool_index_ref, coin_index_ref, amount_ref) = array_refs![&rest_fix, 8, 1, 8];
                MesonInstruction::WithdrawFromPool {
                    pool_index: u64::from_be_bytes(*pool_index_ref),
                    coin_index: coin_index_ref[0],
                    amount: u64::from_be_bytes(*amount_ref),
                }
            }

            10 => {
                let rest_fix = *array_ref![rest, 0, 148];
                let (encoded_swap_ref, signature_ref, initiator_ref, recipient_ref) =
                    array_refs![&rest_fix, 32, 64, 20, 32];
                MesonInstruction::Lock {
                    encoded_swap: *encoded_swap_ref,
                    signature: *signature_ref,
                    initiator: *initiator_ref,
                    recipient: Pubkey::new_from_array(*recipient_ref),
                }
            }

            11 => {
                let rest_fix = *array_ref![rest, 0, 52];
                let (encoded_swap_ref, initiator_ref) = array_refs![&rest_fix, 32, 20];
                MesonInstruction::Unlock {
                    encoded_swap: *encoded_swap_ref,
                    initiator: *initiator_ref,
                }
            }

            12 => {
                let rest_fix = *array_ref![rest, 0, 116];
                let (encoded_swap_ref, signature_ref, initiator_ref) =
                    array_refs![&rest_fix, 32, 64, 20];
                MesonInstruction::Release {
                    encoded_swap: *encoded_swap_ref,
                    signature: *signature_ref,
                    initiator: *initiator_ref,
                }
            }

            _ => return Err(MesonError::InvalidInstruction),
        })
    }
}
