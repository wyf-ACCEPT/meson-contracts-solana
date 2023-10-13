# Meson Contracts on Solana

Meson is the faster and safer way to execute low-cost, zero-slippage universal cross-chain swaps across all leading blockchains and layer-2 rollups.

This repository contains the **Rust** implementation for the Meson protocol on **Solana**. See [Meson Docs](https://docs.meson.fi/protocol/background) for the design details of Meson.

Execute the code below to run the unit tests:

```bash
cargo build-sbf
RUST_LOG=error cargo test-sbf --test functional -- --nocapture  # See the logs from functional.rs
```

<br>

## File structure

The file structure of the contract code is as follows:

```bash
src
├── entrypoint.rs
├── error.rs
├── instruction.rs
├── lib.rs
├── mesonpools.rs
├── mesonswap.rs
├── processor.rs
├── state.rs
└── utils.rs
```

- `entrypoint.rs`: the entrypoint of the contract, which is called by the Solana runtime.
- `error.rs`: defines the error types of the contract.
- `instruction.rs`: defines the instruction types of the contract. For more details see the next chapter.
- `lib.rs`: lists all the modules of the contract.
- `mesonpools.rs`: defines the data structure of the Meson Pools, and the functions that the source chain of the cross-chain swap needs to call.
- `mesonswap.rs`: defines the data structure of the Meson Swap, and the functions that the target chain of the cross-chain swap needs to call.
- `processor.rs`: defines the processor of the contract, which is called by the entrypoint.
- `state.rs`: mainly defines the functions that will change the state of the contract.
- `utils.rs`: mainly defines the utility view functions of the contract, which won't change the state.

As a developer, you need to know how to interact with the contracts. The following sections will introduce the instructions and the data structures of the contracts. You can find more details in the annotations of the code file [instruction.rs](./src/instruction.rs).

<br>

## Instructions

There are currently 13 instructions in the contract, indexed from 0 to 12. When calling a instruction, you need to pass the accounts list as the `accounts: &[AccountInfo]` params, and the related data as the `data: &[u8]` params (starting with the instruction index). The accounts list and the data are defined in the [instruction.rs](./src/instruction.rs) file.

You can find the examples of all the instructions in the [unit tests](./tests/functional.rs).

<br>

### Instruction 0  
```Rust
InitContract
```
  
  The admin(deployer) must call this init function first, to set the admin address and the supported coin list well. Here are the accounts list to pass to the instruction:
    
  1. payer_account: the contract deployer, also the admin
  2. system_program: that is `11111111111111111111111111111111`
  3. authority_account: to save the address of admin
  4. save_token_list_account: to save the supported coin list

<br>

### Instruction 1 
```Rust
TransferPremiumManager
```
  
  The admin can transfer the premium manager to another address. Here are the accounts list to pass to the instruction:
  
  1. admin_account: the admin account, must be a signer
  2. authority_account
  3. new_admin: the new admin address

<br>

### Instruction 2 
```Rust
AddSupportToken { coin_index: u8 }
```

  The admin can add a new coin to the supported coin list. Here are the accounts list to pass to the instruction:
    
  1. admin_account
  2. authority_account
  3. save_token_list_account
  4. token_mint_account: the mint address of the coin to add to support list

<br>

### Instruction 3
```Rust
RegisterPool { pool_index: u64 }
```
  
  The LP can register a new pool. Here are the accounts list:
  
  1. payer_account
  2. system_program
  3. authorized_account: the address to add to LP pools
  4. save_poaa_account_input: the data account to save `authorized address -> pool index` pair (8-bytes long)
  5. save_oop_account_input: the data account to save `pool index -> authorized address` pair (32-bytes long)
  
<br>

### Instruction 4
```Rust
PostSwap {
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
    pool_index: u64,
}
```
  
  The user can post a swap, which is the **Step 1** of the cross-chain swap. Here are the accounts list:
  
  1. payer_account
  2. system_program
  3. user_account: the user who wants to swap
  4. token_mint_account
  5. token_program_info: that is "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  6. save_token_list_account
  7. save_ps_account_input: the data account to save `encoded -> postedSwap` pair (60-bytes)
  8. ta_user_input: the token account for the user
  9. ta_program_input: the token account for the program

<br>

### Instruction 5
```Rust
BondSwap {
    encoded_swap: [u8; 32],
    pool_index: u64,
}
```
  
  The LP can bond a swap after the user called `PostSwap`. Here are the accounts list:
  
  0. sender_account: same as `authorized_account`
  1. save_poaa_account_input
  2. save_ps_account_input

<br>

### Instruction 6
```Rust
CancelSwap { encoded_swap: [u8; 32] }
```

  The user can cancel a swap after he/she called `PostSwap`. Here are the accounts list:
  
  0. token_mint_account
  1. token_program_info
  2. save_ps_account_input
  3. ta_user_input
  4. ta_program_input
  5. contract_signer_account_input

<br>

### Instruction 7
```Rust
ExecuteSwap {
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    recipient: [u8; 20],
    deposit_to_pool_bool: bool,
}
```
  
  The LP can execute a swap after the user called `PostSwap` and the LP called `BondSwap`. This is the **Step 4** of the cross-chain swap. Here are the accounts list:
  
  1. token_mint_account
  2. token_program_info
  3. save_ps_account_input
  4. save_oop_account_input
  5. save_balance_lp_account_input: the data account to save `pool_index & coin_index -> balance` pair (8-bytes long to save u64 balance)
  6. ta_lp_input: the token account for lp (the owner of pool_index)
  7. ta_program_input
  8. contract_signer_account_input: the account as a singer of the program contract

<br>

### Instruction 8
```Rust
DepositToPool {
    pool_index: u64,
    coin_index: u8,
    amount: u64,
}
```

  The LP can deposit some tokens to his/her pool. Here are the accounts list:
  
  1. payer_account
  2. system_program
  3. authorized_account_input: the address to add to LP pools
  4. token_mint_account
  5. token_program_info
  6. save_token_list_account
  7. save_poaa_account_input
  8. save_balance_lp_account_input: the data account to save `pool_index & coin_index -> balance` pair (8-bytes long to save u64 balance)
  9. ta_lp_input
  10. ta_program_input

<br>

### Instruction 9
```Rust
WithdrawFromPool {
    pool_index: u64,
    coin_index: u8,
    amount: u64,
}
```

  The LP can withdraw some tokens from his/her pool. Here are the accounts list:
  
  1. authorized_account_input
  2. token_mint_account
  3. token_program_info
  4. save_token_list_account
  5. save_poaa_account_input
  6. save_balance_lp_account_input
  7. ta_lp_input
  8. ta_program_input
  9. contract_signer_account_input

<br>

### Instruction 10
```Rust
Lock {
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
    recipient: Pubkey,
}
```

  The LP can lock the tokens on the target chain, which is the **Step 2** of the cross-chain swap. Here are the accounts list:
  
  1. payer_account
  2. system_program
  3. authorized_account_input
  4. token_mint_account
  5. save_si_account_input: the data account to save `swapId -> lockedSwap` pair (48-bytes)
  6. save_token_list_account
  7. save_poaa_account_input
  8. save_balance_lp_account_input
  9. ta_user_input
  10. ta_program_input

<br>

### Instruction 11
```Rust
Unlock {
    encoded_swap: [u8; 32],
    initiator: [u8; 20],
}
```

  The LP can unlock the tokens on the target chain if the user hasn't released the tokens before the specified time. Here are the accounts list:
  
  1. save_si_account_input
  2. save_balance_lp_account_input

<br>

### Instruction 12
```Rust
Release {
    encoded_swap: [u8; 32],
    signature: [u8; 64],
    initiator: [u8; 20],
}
```

  The user can release the tokens on the target chain, which is the **Step 3** of the cross-chain swap. Here are the accounts list:
  
  1. payer_account
  2. system_program
  3. token_mint_account
  4. token_program_info
  5. save_si_account_input
  6. save_oop_admin_account_input: the data account to save `pool_index=0(the manager) -> authorized address` pair (32-bytes long)
  7. save_balance_manager_account_input: the data account to save `pool_index=0(the manager) & coin_index -> balance` pair (8-bytes long to save u64 balance)
  8. ta_user_input
  9. ta_program_input
  10. contract_signer_account_input
