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
    // =                        Add Supported Token                        =
    // =                                                                   =
    // =====================================================================
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    // let token_mint0 = Pubkey::new_unique();
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
                    AccountMeta::new(mint_pubkey, false),
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
    println!("Token mint address 0: {}", mint_pubkey);
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
    let rent = Rent::default();

    let ta_program = Keypair::new(); // Token account for the program
    let ta_payer = Keypair::new();
    let ta_alice = Keypair::new();
    let ta_bob = Keypair::new();
    let (token_transfer_pubkey, _) =
        Pubkey::find_program_address(&[b"token_transfer"], &program_id);

    // Setup the mint (so the payer is the admin of the token)
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

    // Setup token accounts for everyone
    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            f1_temp(ta_program.pubkey()),
            f2_temp(ta_program.pubkey(), token_transfer_pubkey),
            f1_temp(ta_alice.pubkey()),
            f2_temp(ta_alice.pubkey(), alice.pubkey()),
            f1_temp(ta_bob.pubkey()),
            f2_temp(ta_bob.pubkey(), bob.pubkey()),
        ],
        Some(&payer.pubkey()),
        &[&payer, &ta_program, &ta_alice, &ta_bob],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== Register token account ==================");
    println!("Token account for [Program]: {}", ta_program.pubkey());
    println!("Token account for [Alice]  : {}", ta_alice.pubkey());
    println!("Token account for [Bob]    : {}", ta_bob.pubkey());

    // Mint some tokens for everyone
    let mint_amount = 500_000_000;
    let f3_temp = |token_account_pubkey: Pubkey| {
        spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint_pubkey,
            &token_account_pubkey,
            &payer_pubkey,
            &[],
            mint_amount,
        )
        .unwrap()
    };
    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            f3_temp(ta_program.pubkey()),
            f3_temp(ta_alice.pubkey()),
            f3_temp(ta_bob.pubkey()),
        ],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== Mint some tokens ==================");
    let ta_program_info = get_account_info(&mut banks_client, ta_program.pubkey()).await;
    let ta_alice_info = get_account_info(&mut banks_client, ta_alice.pubkey()).await;
    let ta_bob_info = get_account_info(&mut banks_client, ta_bob.pubkey()).await;
    println!(
        "USDC Balance of [Program]: {:?}",
        TokenAccount::unpack(ta_program_info.data()).unwrap().amount
    );
    println!(
        "USDC Balance of [Alice]  : {:?}",
        TokenAccount::unpack(ta_alice_info.data()).unwrap().amount
    );
    println!(
        "USDC Balance of [Bob]    : {:?}",
        TokenAccount::unpack(ta_bob_info.data()).unwrap().amount
    );

    println!("{:?}", Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap().as_ref());

    // =====================================================================
    // =                                                                   =
    // =                       S.1 Post-swap by Alice                      =
    // =                                                                   =
    // =====================================================================
    let encoded_swap: [u8; 32] = [
        0x01, 0x00, 0x00, 0xe4, 0xe1, 0xc0, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81,
        0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x01, 0xf5, 0x00, 0x01,
        0xf5, 0x00,
    ];
    let signature: [u8; 64] = [
        0xb3, 0x18, 0x4c, 0x25, 0x7c, 0xf9, 0x73, 0x06, 0x92, 0x50, 0xee, 0xfd, 0x84, 0x9a, 0x74,
        0xd2, 0x72, 0x50, 0xf8, 0x34, 0x3c, 0xbd, 0xa7, 0x61, 0x51, 0x91, 0x14, 0x9d, 0xd3, 0xc1,
        0xb6, 0x1d, 0x5d, 0x4e, 0x2b, 0x5e, 0xcc, 0x76, 0xa5, 0x9b, 0xaa, 0xbf, 0x10, 0xa8, 0xd5,
        0xd1, 0x16, 0xed, 0xb9, 0x5a, 0x5b, 0x20, 0x55, 0xb9, 0xb1, 0x9f, 0x71, 0x52, 0x40, 0x96,
        0x97, 0x5b, 0x29, 0xc2,
    ];
    let initiator: [u8; 20] = [
        0x2e, 0xf8, 0xa5, 0x1f, 0x8f, 0xf1, 0x29, 0xdb, 0xb8, 0x74, 0xa0, 0xef, 0xb0, 0x21, 0x70,
        0x2f, 0x59, 0xc1, 0xb2, 0x11,
    ];
    let (save_ps_pubkey, _) = Pubkey::find_program_address(
        &[ConstantValue::SAVE_POSTED_SWAP_PHRASE, &encoded_swap],
        &program_id,
    );

    let mut data_input_array = [4 as u8; 125];
    data_input_array[1..33].copy_from_slice(&encoded_swap);
    data_input_array[33..97].copy_from_slice(&signature);
    data_input_array[97..117].copy_from_slice(&initiator);
    data_input_array[117..125].copy_from_slice(&alice_pool_index.to_be_bytes());

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(payer_pubkey, false),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(token_list_pda, false),
                AccountMeta::new(save_ps_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== S.1 Post Swap ==================");
    println!("Data account for post-swap: {}", save_ps_pubkey);
    let ps_info = get_account_info(&mut banks_client, save_ps_pubkey).await;
    let (pool_index, initiator, from_address) =
        array_refs![array_ref![ps_info.data(), 0, 60], 8, 20, 32];
    println!(
        "Data inside:\n\tPool index: {}\n\tInitiator: {:?}\n\tFrom addr: {}",
        u64::from_be_bytes(*pool_index),
        initiator,
        Pubkey::from(*from_address)
    );
}
