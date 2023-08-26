use arrayref::{array_ref, array_refs};
use solana_program::system_instruction;
use std::str::FromStr;

use {
    meson_contracts_solana::{entrypoint::process_instruction, state::ConstantValue},
    solana_program::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        system_program,
        sysvar::{rent::Rent, Sysvar},
    },
    solana_program_test::*,
    solana_sdk::{
        account::{Account, ReadableAccount},
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
    spl_token::{
        self,
        state::{Account as TokenAccount, Mint},
    },
};

async fn get_account_info(banks_client: &mut BanksClient, account: Pubkey) -> Account {
    banks_client.get_account(account).await.unwrap().unwrap()
}

async fn update_blockhash(banks_client: &mut BanksClient, recent_blockhash: Hash) -> Hash {
    banks_client
        .get_new_latest_blockhash(&recent_blockhash)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_all() {
    let program_id = Pubkey::from_str("Meson11111111111111111111111111111111111111").unwrap();
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "meson_contracts_solana",
        program_id,
        processor!(process_instruction),
    )
    .start()
    .await;
    let payer_pubkey = payer.pubkey();
    // let payer_pubkey = Pubkey::new_unique();

    // =====================================================================
    // =                                                                   =
    // =                            Init Contract                          =
    // =                                                                   =
    // =====================================================================
    let (auth_pda, _) = Pubkey::find_program_address(&[b"authority"], &program_id);
    let (token_list_pda, _) = Pubkey::find_program_address(&[b"supported_coins"], &program_id);
    let (save_poaa_pubkey_admin, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_POOL_OF_AUTHORIZED_ADDR_PHRASE,
            payer_pubkey.as_ref(),
        ],
        &program_id,
    );
    let (save_oop_pubkey_admin, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_OWNER_OF_POOLS_PHRASE,
            &(0 as u64).to_be_bytes(),
        ],
        &program_id,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0 as u8],
            vec![
                AccountMeta::new(payer_pubkey, false),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(auth_pda, false),
                AccountMeta::new(token_list_pda, false),
                AccountMeta::new(save_poaa_pubkey_admin, false),
                AccountMeta::new(save_oop_pubkey_admin, false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== Init Contract ==================");
    println!("Program   pubkey: {}", program_id);
    println!("Payer     pubkey: {}", payer_pubkey);
    let authority_info = get_account_info(&mut banks_client, auth_pda).await;
    println!(
        "Current   admin : {}",
        Pubkey::from(*array_ref![authority_info.data(), 0, 32])
    );
    let premium_info = get_account_info(&mut banks_client, save_oop_pubkey_admin).await;
    println!(
        "Premium manager : {}",
        Pubkey::from(*array_ref![premium_info.data(), 0, 32])
    );

    // show_account_info(&mut banks_client, payer_pubkey).await;
    // show_account_info(&mut banks_client, auth_pda).await;
    // show_account_info(&mut banks_client, token_list_pda).await;
    // show_account_info(&mut banks_client, program_id).await;

    // =====================================================================
    // =                                                                   =
    // =                            Transfer Admin                         =
    // =                                                                   =
    // =====================================================================
    let new_admin = Keypair::new(); // Temporary admin
    let alice = Keypair::new(); // LP
    let bob = Keypair::new(); // User
    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            Instruction::new_with_bincode(
                program_id,
                &[1 as u8],
                vec![
                    AccountMeta::new(payer_pubkey, false),
                    AccountMeta::new(auth_pda, false),
                    AccountMeta::new(new_admin.pubkey(), false),
                ],
            ),
            system_instruction::transfer(&payer_pubkey, &new_admin.pubkey(), 1500000000),
            system_instruction::transfer(&payer_pubkey, &alice.pubkey(), 7500000000),
            system_instruction::transfer(&payer_pubkey, &bob.pubkey(), 2500000000),
        ],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== Transfer Admin ==================");
    let authority_info = get_account_info(&mut banks_client, auth_pda).await;
    println!("New       pubkey: {}", new_admin.pubkey());
    println!(
        "New       admin : {} (balance: {})",
        Pubkey::from(*array_ref![authority_info.data(), 0, 32]),
        banks_client.get_balance(new_admin.pubkey()).await.unwrap()
    );

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[1 as u8],
            vec![
                AccountMeta::new(new_admin.pubkey(), false),
                AccountMeta::new(auth_pda, false),
                AccountMeta::new(payer_pubkey, false),
            ],
        )],
        Some(&new_admin.pubkey()),
        &[&new_admin],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();
    let authority_info = get_account_info(&mut banks_client, auth_pda).await;
    println!(
        "Admin trans-back: {} (balance: {})",
        Pubkey::from(*array_ref![authority_info.data(), 0, 32]),
        banks_client.get_balance(payer_pubkey).await.unwrap()
    );

    // =====================================================================
    // =                                                                   =
    // =                          Add Support Token                        =
    // =                                                                   =
    // =====================================================================
    let token_mint0 = Pubkey::new_unique();
    let token_mint3 = Pubkey::new_unique();
    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            Instruction::new_with_bincode(
                program_id,
                &[2 as u8, 0],
                vec![
                    AccountMeta::new(payer_pubkey, false),
                    AccountMeta::new(auth_pda, false),
                    AccountMeta::new(token_list_pda, false),
                    AccountMeta::new(token_mint0, false),
                ],
            ),
            Instruction::new_with_bincode(
                program_id,
                &[2 as u8, 3],
                vec![
                    AccountMeta::new(payer_pubkey, false),
                    AccountMeta::new(auth_pda, false),
                    AccountMeta::new(token_list_pda, false),
                    AccountMeta::new(token_mint3, false),
                ],
            ),
        ],
        Some(&payer_pubkey),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== Add Support Token ==================");
    let token_list_info = get_account_info(&mut banks_client, token_list_pda).await;
    println!("Token mint address 0: {}", token_mint0);
    println!("Token mint address 3: {}", token_mint3);
    let t1234 = array_ref![token_list_info.data(), 0, 128];
    let (t1, t2, t3, t4) = array_refs![t1234, 32, 32, 32, 32];
    println!(
        "Support coin list : [\n\t{}, \n\t{}, \n\t{}, \n\t{}\n]",
        Pubkey::from(*t1),
        Pubkey::from(*t2),
        Pubkey::from(*t3),
        Pubkey::from(*t4)
    );

    // =====================================================================
    // =                                                                   =
    // =                       LP Register new pool                        =
    // =                                                                   =
    // =====================================================================
    let alice_pool_index: u64 = 5;
    let (save_poaa_pubkey_alice, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_POOL_OF_AUTHORIZED_ADDR_PHRASE,
            alice.pubkey().as_ref(),
        ],
        &program_id,
    );
    let (save_oop_pubkey_alice, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_OWNER_OF_POOLS_PHRASE,
            &(alice_pool_index as u64).to_be_bytes(),
        ],
        &program_id,
    );
    let mut data_input_array = [3 as u8; 9];
    data_input_array[1..].copy_from_slice(&alice_pool_index.to_be_bytes());
    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(alice.pubkey(), false),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(alice.pubkey(), false),
                AccountMeta::new(save_poaa_pubkey_alice, false),
                AccountMeta::new(save_oop_pubkey_alice, false),
            ],
        )],
        Some(&alice.pubkey()),
        &[&alice],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== Register new pool ==================");
    let pool_info = get_account_info(&mut banks_client, save_oop_pubkey_alice).await;
    println!(
        "Owner of pool {}    : {}",
        alice_pool_index,
        Pubkey::from(*array_ref![pool_info.data(), 0, 32])
    );
    let aa_info = get_account_info(&mut banks_client, save_poaa_pubkey_alice).await;
    println!(
        "Pool index for addr: {} -> {:?}",
        alice.pubkey(),
        u64::from_be_bytes(*array_ref![aa_info.data(), 0, 8])
    );

    // =====================================================================
    // =                                                                   =
    // =                         Create a token                            =
    // =                                                                   =
    // =====================================================================
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let rent = Rent::default();

    let ta_program = Keypair::new(); // Token account for the program
    let ta_payer = Keypair::new();
    let ta_alice = Keypair::new();
    let ta_bob = Keypair::new();
    let (token_transfer_pubkey, _) =
        Pubkey::find_program_address(&[b"token_transfer"], &program_id);

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint_pubkey,
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint_pubkey,
                &payer.pubkey(),
                None,
                6,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &mint_keypair],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // (ta_program.pubkey(), token_transfer_pubkey),
    // (ta_payer.pubkey(), payer_pubkey),
    // (ta_alice.pubkey(), alice.pubkey()),
    // (ta_bob.pubkey(), bob.pubkey()),

    let f1_temp = |token_account_pubkey: Pubkey| {
        system_instruction::create_account(
            &payer.pubkey(),
            &token_account_pubkey,
            rent.minimum_balance(TokenAccount::LEN),
            TokenAccount::LEN as u64,
            &spl_token::id(),
        )
    };

    let f2_temp = |token_account_pubkey: Pubkey, owner: Pubkey| {
        spl_token::instruction::initialize_account(
            &spl_token::id(),
            &token_account_pubkey,
            &mint_pubkey,
            &owner,
        )
        .unwrap()
    };

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            f1_temp(ta_program.pubkey()),
            f2_temp(ta_program.pubkey(), token_transfer_pubkey),
            f1_temp(ta_payer.pubkey()),
            f2_temp(ta_payer.pubkey(), payer_pubkey),
        ],
        Some(&payer.pubkey()),
        &[&payer, &ta_program],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();
}
