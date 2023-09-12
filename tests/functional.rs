use meson_contracts_solana::utils::Utils;

use {
    arrayref::{array_ref, array_refs},
    meson_contracts_solana::{entrypoint::process_instruction, state::ConstantValue},
    solana_program::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction, system_program,
        sysvar::rent::Rent,
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
    std::{
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
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

async fn show_usdc_balance_all(
    banks_client: &mut BanksClient,
    ta_program: &Keypair,
    ta_alice: &Keypair,
    ta_bob: &Keypair,
    step: &str,
) {
    let ta_program_info = get_account_info(banks_client, ta_program.pubkey()).await;
    let ta_alice_info = get_account_info(banks_client, ta_alice.pubkey()).await;
    let ta_bob_info = get_account_info(banks_client, ta_bob.pubkey()).await;
    println!(
        "After {}: USDC Balance of [Program]: {:?}",
        step,
        TokenAccount::unpack(ta_program_info.data()).unwrap().amount
    );
    println!(
        "After {}: USDC Balance of [ Alice ]: {:?}",
        step,
        TokenAccount::unpack(ta_alice_info.data()).unwrap().amount
    );
    println!(
        "After {}: USDC Balance of [  Bob  ]: {:?}",
        step,
        TokenAccount::unpack(ta_bob_info.data()).unwrap().amount
    );
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
    println!("\n================== Init Contract ==================");
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
    println!("\n================== Transfer Admin ==================");
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
    println!("\n================== Add Support Token ==================");
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
    println!("\n================== Register new pool ==================");
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
    println!("\n================== Create a token ==================");
    let rent = Rent::default();

    let ta_program = Keypair::new(); // Token account for the program
    let ta_alice = Keypair::new();
    let ta_bob = Keypair::new();
    let (contract_signer_pubkey, _) =
        Pubkey::find_program_address(&[ConstantValue::CONTRACT_SIGNER], &program_id);

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
    println!("Token mint (Token address): {}", mint_pubkey);

    // (ta_program.pubkey(), contract_signer_pubkey),
    // (ta_payer.pubkey(), payer_pubkey),
    // (ta_alice.pubkey(), alice.pubkey()),
    // (ta_bob.pubkey(), bob.pubkey()),

    println!("\n================== Register token account ==================");
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
            f2_temp(ta_program.pubkey(), contract_signer_pubkey),
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

    println!("Token account for [Program]: {}", ta_program.pubkey());
    println!("Token account for [ Alice ]: {}", ta_alice.pubkey());
    println!("Token account for [  Bob  ]: {}", ta_bob.pubkey());

    println!("\n================== Mint some tokens ==================");
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

    show_usdc_balance_all(
        &mut banks_client,
        &ta_program,
        &ta_alice,
        &ta_bob,
        "minting",
    )
    .await;

    println!("\n================== Approve ==================");
    let approve_amount = 400_000_000;
    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            spl_token::instruction::approve(
                &spl_token::id(),
                &ta_alice.pubkey(),
                &program_id,
                &alice.pubkey(),
                &[],
                approve_amount,
            )
            .unwrap(),
            spl_token::instruction::approve(
                &spl_token::id(),
                &ta_bob.pubkey(),
                &program_id,
                &bob.pubkey(),
                &[],
                approve_amount,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &alice, &bob],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let ta_alice_info = get_account_info(&mut banks_client, ta_alice.pubkey()).await;
    println!(
        "Token account info [Alice]  : {:?}",
        TokenAccount::unpack(ta_alice_info.data()).unwrap()
    );
    let ta_bob_info = get_account_info(&mut banks_client, ta_bob.pubkey()).await;
    println!(
        "Token account info [ Bob ]  : {:?}",
        TokenAccount::unpack(ta_bob_info.data()).unwrap()
    );

    // =====================================================================
    // =                                                                   =
    // =                    Step.1.1 Post-swap by Bob                      =
    // =                                                                   =
    // =====================================================================
    println!("\n================== Step 1.1 Post Swap ==================");
    // let mut encoded_swap: [u8; 32] = [
    //     0x01, 0x00, 0x00, 0xe4, 0xe1, 0xc0, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81,
    //     0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x01, 0xf5, 0x00, 0x01,
    //     0xf5, 0x00,
    // ];  // for fee-waived
    let mut encoded_swap: [u8; 32] = [
        0x01, 0x00, 0x00, 0xe4, 0xe1, 0xc0, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81,
        0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x01, 0xf5, 0x00, 0x01,
        0xf5, 0x00,
    ];  // for not fee-waived
    let now_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expire_time_expect = (now_timestamp + 3600).to_be_bytes();
    encoded_swap[21..26].copy_from_slice(array_ref![expire_time_expect, 3, 5]);

    let fake_signature_request: [u8; 64] = [
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
    data_input_array[33..97].copy_from_slice(&fake_signature_request);
    data_input_array[97..117].copy_from_slice(&initiator);
    data_input_array[117..125].copy_from_slice(&(0 as u64).to_be_bytes());

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(payer_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(bob.pubkey(), true),
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(spl_token::id(), false),
                AccountMeta::new(token_list_pda, false),
                AccountMeta::new(save_ps_pubkey, false),
                AccountMeta::new(ta_bob.pubkey(), false),
                AccountMeta::new(ta_program.pubkey(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer, &bob],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

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

    show_usdc_balance_all(
        &mut banks_client,
        &ta_program,
        &ta_alice,
        &ta_bob,
        "post-swap",
    )
    .await;

    // =====================================================================
    // =                                                                   =
    // =                    Step.1.2 Bond-swap by Alice                    =
    // =                                                                   =
    // =====================================================================
    println!("\n================== Step 1.2 Bond Swap ==================");

    let mut data_input_array = [5 as u8; 41];
    data_input_array[1..33].copy_from_slice(&encoded_swap);
    data_input_array[33..41].copy_from_slice(&alice_pool_index.to_be_bytes());

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(alice.pubkey(), true),
                AccountMeta::new(save_poaa_pubkey_alice, false),
                AccountMeta::new(save_ps_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer, &alice],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Data account for post-swap: {}", save_ps_pubkey);
    let ps_info = get_account_info(&mut banks_client, save_ps_pubkey).await;
    let (pool_index, initiator, from_address) =
        array_refs![array_ref![ps_info.data(), 0, 60], 8, 20, 32];
    println!(
        "Data inside after bond-swap:\n\tPool index: {}\n\tInitiator: {:?}\n\tFrom addr: {}",
        u64::from_be_bytes(*pool_index),
        initiator,
        Pubkey::from(*from_address)
    );

    // =====================================================================
    // =                                                                   =
    // =                  LP Deposit & Withdraw assets                     =
    // =                                                                   =
    // =====================================================================
    println!("\n================== Deposit assets to pool ==================");

    let coin_index = 0;
    let deposit_amount: u64 = 170_000_000;
    let (save_balance_pubkey_alice, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_BALANCE_PHRASE,
            &alice_pool_index.to_be_bytes(),
            &[coin_index],
        ],
        &program_id,
    );

    let mut data_input_array = [8 as u8; 18];
    data_input_array[1..9].copy_from_slice(&alice_pool_index.to_be_bytes());
    data_input_array[9] = coin_index;
    data_input_array[10..18].copy_from_slice(&deposit_amount.to_be_bytes());

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(payer_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(alice.pubkey(), true),
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(spl_token::id(), false),
                AccountMeta::new(token_list_pda, false),
                AccountMeta::new(save_poaa_pubkey_alice, false),
                AccountMeta::new(save_balance_pubkey_alice, false),
                AccountMeta::new(ta_alice.pubkey(), false),
                AccountMeta::new(ta_program.pubkey(), false),
            ],
        )],
        Some(&alice.pubkey()),
        &[&payer, &alice],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let pool_info = get_account_info(&mut banks_client, save_oop_pubkey_alice).await;
    println!(
        "Owner of pool {}           : {}",
        alice_pool_index,
        Pubkey::from(*array_ref![pool_info.data(), 0, 32])
    );
    let balance_info = get_account_info(&mut banks_client, save_balance_pubkey_alice).await;
    println!(
        "Balance for coin {}, pool {}  [ Pool of Alice ]: {}",
        coin_index,
        alice_pool_index,
        u64::from_be_bytes(*array_ref![balance_info.data(), 0, 8])
    );

    println!("\n================== Deposit again ==================");

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(payer_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(alice.pubkey(), true),
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(spl_token::id(), false),
                AccountMeta::new(token_list_pda, false),
                AccountMeta::new(save_poaa_pubkey_alice, false),
                AccountMeta::new(save_balance_pubkey_alice, false),
                AccountMeta::new(ta_alice.pubkey(), false),
                AccountMeta::new(ta_program.pubkey(), false),
            ],
        )],
        Some(&alice.pubkey()),
        &[&payer, &alice],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let pool_info = get_account_info(&mut banks_client, save_oop_pubkey_alice).await;
    println!(
        "Owner of pool {}           : {}",
        alice_pool_index,
        Pubkey::from(*array_ref![pool_info.data(), 0, 32])
    );
    let balance_info = get_account_info(&mut banks_client, save_balance_pubkey_alice).await;
    println!(
        "Balance for coin {}, pool {}  [Pool Alice]: {}",
        coin_index,
        alice_pool_index,
        u64::from_be_bytes(*array_ref![balance_info.data(), 0, 8])
    );
    show_usdc_balance_all(
        &mut banks_client,
        &ta_program,
        &ta_alice,
        &ta_bob,
        "deposit",
    ).await;

    println!("\n================== Withdraw assets from pool ==================");

    let withdraw_amount: u64 = 100_000_000;
    data_input_array[0] = 9;
    data_input_array[10..18].copy_from_slice(&withdraw_amount.to_be_bytes());

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(alice.pubkey(), true),
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(spl_token::id(), false),
                AccountMeta::new(token_list_pda, false),
                AccountMeta::new(save_poaa_pubkey_alice, false),
                AccountMeta::new(save_balance_pubkey_alice, false),
                AccountMeta::new(ta_alice.pubkey(), false),
                AccountMeta::new(ta_program.pubkey(), false),
                AccountMeta::new(contract_signer_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer, &alice],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let pool_info = get_account_info(&mut banks_client, save_oop_pubkey_alice).await;
    println!(
        "Owner of pool {}           : {}",
        alice_pool_index,
        Pubkey::from(*array_ref![pool_info.data(), 0, 32])
    );
    let balance_info = get_account_info(&mut banks_client, save_balance_pubkey_alice).await;
    println!(
        "Balance for coin {}, pool {}  [Pool Alice]: {}",
        coin_index,
        alice_pool_index,
        u64::from_be_bytes(*array_ref![balance_info.data(), 0, 8])
    );
    show_usdc_balance_all(
        &mut banks_client,
        &ta_program,
        &ta_alice,
        &ta_bob,
        "deposit",
    ).await;

    // =====================================================================
    // =                                                                   =
    // =                     Step.2 Locked by Alice                        =
    // =                                                                   =
    // =====================================================================
    println!("\n================== Step 2. Lock ==================");

    let initiator: [u8; 20] = [
        0x2e, 0xf8, 0xa5, 0x1f, 0x8f, 0xf1, 0x29, 0xdb, 0xb8, 0x74, 0xa0, 0xef, 0xb0, 0x21, 0x70,
        0x2f, 0x59, 0xc1, 0xb2, 0x11,
    ];
    let recipient = bob.pubkey();

    let swap_id = Utils::get_swap_id(encoded_swap, initiator);
    let (save_si_pubkey, _) = Pubkey::find_program_address(
        &[ConstantValue::SAVE_LOCKED_SWAP_PHRASE, &swap_id],
        &program_id,
    );

    let mut data_input_array = [10 as u8; 149];
    data_input_array[1..33].copy_from_slice(&encoded_swap);
    data_input_array[33..97].copy_from_slice(&fake_signature_request);
    data_input_array[97..117].copy_from_slice(&initiator);
    data_input_array[117..149].copy_from_slice(&recipient.to_bytes());

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(payer_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(alice.pubkey(), false),
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(save_si_pubkey, false),
                AccountMeta::new(token_list_pda, false),
                AccountMeta::new(save_poaa_pubkey_alice, false),
                AccountMeta::new(save_balance_pubkey_alice, false),
            ],
        )],
        Some(&alice.pubkey()),
        &[&payer, &alice],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let balance_info = get_account_info(&mut banks_client, save_balance_pubkey_alice).await;
    println!(
        "Balance for coin {}, pool {}  [Pool Alice]: {}",
        coin_index,
        alice_pool_index,
        u64::from_be_bytes(*array_ref![balance_info.data(), 0, 8])
    );
    show_usdc_balance_all(
        &mut banks_client,
        &ta_program,
        &ta_alice,
        &ta_bob,
        "deposit",
    ).await;

    // =====================================================================
    // =                                                                   =
    // =                      Step.3 Release to Bob                        =
    // =                                                                   =
    // =====================================================================
    println!("\n================== Step 3. Release ==================");

    let fake_signature_release = [
        0x12, 0x05, 0x36, 0x1a, 0xab, 0xc8, 0x9e, 0x5b, 0x30, 0x59, 0x2a, 0x2c, 0x95, 0x59, 0x2d,
        0xdc, 0x12, 0x70, 0x50, 0x61, 0x0e, 0xfe, 0x92, 0xff, 0x64, 0x55, 0xc5, 0xcf, 0xd4, 0x3b,
        0xdd, 0x82, 0x58, 0x53, 0xed, 0xcf, 0x1f, 0xa7, 0x2f, 0x10, 0x99, 0x2b, 0x46, 0x72, 0x1d,
        0x17, 0xcb, 0x31, 0x91, 0xa8, 0x5c, 0xef, 0xd2, 0xf8, 0x32, 0x5b, 0x1a, 0xc5, 0x9c, 0x7d,
        0x49, 0x8f, 0xa2, 0x12,
    ];

    let (save_balance_pubkey_manager, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_BALANCE_PHRASE,
            &(0 as u64).to_be_bytes(),
            &[coin_index],
        ],
        &program_id,
    );    
    let (save_oop_pubkey_manager, _) = Pubkey::find_program_address(
        &[
            ConstantValue::SAVE_OWNER_OF_POOLS_PHRASE,
            &(0 as u64).to_be_bytes(),
        ],
        &program_id,
    );

    let mut data_input_array = [12 as u8; 117];
    data_input_array[1..33].copy_from_slice(&encoded_swap);
    data_input_array[33..97].copy_from_slice(&fake_signature_release);
    data_input_array[97..117].copy_from_slice(&initiator);

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(payer_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(spl_token::id(), false),
                AccountMeta::new(save_si_pubkey, false),
                AccountMeta::new(save_oop_pubkey_manager, false),
                AccountMeta::new(save_balance_pubkey_manager, false),
                AccountMeta::new(ta_bob.pubkey(), false),
                AccountMeta::new(ta_program.pubkey(), false),
                AccountMeta::new(contract_signer_pubkey, false),
            ],
        )],
        Some(&alice.pubkey()),
        &[&payer, &alice],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let balance_info = get_account_info(&mut banks_client, save_balance_pubkey_alice).await;
    println!(
        "Balance for coin {}, pool {}  [Pool Alice]: {}",
        coin_index,
        alice_pool_index,
        u64::from_be_bytes(*array_ref![balance_info.data(), 0, 8])
    );
    show_usdc_balance_all(
        &mut banks_client,
        &ta_program,
        &ta_alice,
        &ta_bob,
        "deposit",
    ).await;

    // =====================================================================
    // =                                                                   =
    // =                    Step 4. Execute-swap by Alice                   =
    // =                                                                   =
    // =====================================================================
    println!("\n================== Step 4. Execute Swap ==================");

    let recipient = [
        0x01, 0x01, 0x5a, 0xce, 0x92, 0x0c, 0x71, 0x67, 0x94, 0x44, 0x59, 0x79, 0xbe, 0x68, 0xd4,
        0x02, 0xd2, 0x8b, 0x28, 0x05,
    ];

    let mut data_input_array = [7 as u8; 118];
    data_input_array[1..33].copy_from_slice(&encoded_swap);
    data_input_array[33..97].copy_from_slice(&fake_signature_release);
    data_input_array[97..117].copy_from_slice(&recipient);
    data_input_array[117] = 1;  // deposit to pool?

    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bytes(
            program_id,
            &data_input_array,
            vec![
                AccountMeta::new(mint_pubkey, false),
                AccountMeta::new(spl_token::id(), false),
                AccountMeta::new(save_ps_pubkey, false),
                AccountMeta::new(save_oop_pubkey_alice, false),
                AccountMeta::new(save_balance_pubkey_alice, false),
                AccountMeta::new(ta_alice.pubkey(), false),
                AccountMeta::new(ta_program.pubkey(), false),
                AccountMeta::new(contract_signer_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("Data account for post-swap: {}", save_ps_pubkey);
    let ps_info = get_account_info(&mut banks_client, save_ps_pubkey).await;
    let (pool_index, initiator, from_address) =
        array_refs![array_ref![ps_info.data(), 0, 60], 8, 20, 32];
    println!(
        "Data inside after execute-swap:\n\tPool index: {}\n\tInitiator: {:?}\n\tFrom addr: {}",
        u64::from_be_bytes(*pool_index),
        initiator,
        Pubkey::from(*from_address)
    );

    let balance_info_admin = get_account_info(&mut banks_client, save_balance_pubkey_manager).await;
    println!(
        "\nBalance for coin {}, pool 0  [Pool of Manager]: {}",
        coin_index,
        u64::from_be_bytes(*array_ref![balance_info_admin.data(), 0, 8])
    );
    let balance_info = get_account_info(&mut banks_client, save_balance_pubkey_alice).await;
    println!(
        "Balance for coin {}, pool {}  [ Pool of Alice ]: {}",
        coin_index,
        alice_pool_index,
        u64::from_be_bytes(*array_ref![balance_info.data(), 0, 8])
    );
    show_usdc_balance_all(
        &mut banks_client,
        &ta_program,
        &ta_alice,
        &ta_bob,
        "execute-swap",
    )
    .await;

}
