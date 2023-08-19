use crate::error::MesonError;

#[derive(Clone, Debug, PartialEq)]
pub enum MesonInstruction {

    /// The admin(deployer) must call this init function first!
    /// Account data:
    /// 1. payer_account: the contract deployer, also the admin
    /// 2. authority_account: to save the address of admin
    /// 3. map_token_account: to save the supported coin list
    /// 4. system_program: that is `11111111111111111111111111111111`
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
    AddSupportToken {
        coin_index: u8
    }
}

impl MesonInstruction {
    /// Unpacks a byte buffer into a [TokenInstruction](enum.TokenInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, MesonError> {

        let (&tag, rest) = input.split_first().ok_or(MesonError::InvalidInstruction)?;
        Ok(match tag {
            0 => MesonInstruction::InitContract,

            1 => MesonInstruction::TransferPremiumManager,

            2 => MesonInstruction::AddSupportToken { coin_index: rest[0] },

            _ => return Err(MesonError::InvalidInstruction),
        })
    }
}
