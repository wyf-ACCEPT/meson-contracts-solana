use arrayref::array_ref;
use solana_program::system_instruction;
use std::str::FromStr;

use {
    meson_contracts_solana::entrypoint::process_instruction,
    solana_program::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    },
    solana_program_test::*,
    solana_sdk::{
        account::{Account, ReadableAccount},
        signature::{Keypair, Signer},
        transaction::Transaction,
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

// async fn show_account_info(banks_client: &mut BanksClient, account: Pubkey) {
//     let program_id_data = get_account_info(banks_client, account).await;
//     println!(
//         "[AccountInfo {}]\nProgram data: {:?}\n",
//         account, program_id_data
//     );
// }

#[tokio::test]
async fn test_write() {
    let program_id = Pubkey::from_str("Meson11111111111111111111111111111111111111").unwrap();
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "meson_contracts_solana",
        program_id,
        processor!(process_instruction),
    )
    .start()
    .await;
    let payer_account = payer.pubkey();
    // let payer_account = Pubkey::new_unique();

    // =====================================================================
    // =                                                                   =
    // =                            Init Contract                          =
    // =                                                                   =
    // =====================================================================
    let (auth_pda, _) = Pubkey::find_program_address(&[b"authority"], &program_id);
    let (token_pda, _) = Pubkey::find_program_address(&[b"supported_coins"], &program_id);
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0 as u8],
            vec![
                AccountMeta::new(payer_account, false),
                AccountMeta::new(auth_pda, false),
                AccountMeta::new(token_pda, false),
                AccountMeta::new(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    println!("\n================== Init Contract ==================");
    println!("Program   pubkey: {}", program_id);
    println!("Payer     pubkey: {}", payer_account);
    let authority_info = get_account_info(&mut banks_client, auth_pda).await;
    println!(
        "Current   admin : {}",
        Pubkey::from(*array_ref![authority_info.data(), 0, 32])
    );

    // show_account_info(&mut banks_client, payer_account).await;
    // show_account_info(&mut banks_client, auth_pda).await;
    // show_account_info(&mut banks_client, token_pda).await;
    // show_account_info(&mut banks_client, program_id).await;

    // =====================================================================
    // =                                                                   =
    // =                            Transfer Admin                         =
    // =                                                                   =
    // =====================================================================
    let new_admin = Keypair::new();
    let recent_blockhash = update_blockhash(&mut banks_client, recent_blockhash).await;
    let transaction = Transaction::new_signed_with_payer(
        &[
            Instruction::new_with_bincode(
                program_id,
                &[1 as u8],
                vec![
                    AccountMeta::new(payer_account, false),
                    AccountMeta::new(auth_pda, false),
                    AccountMeta::new(new_admin.pubkey(), false),
                ],
            ),
            system_instruction::transfer(&payer_account, &new_admin.pubkey(), 500000000),
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
                AccountMeta::new(payer_account, false),
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
        banks_client.get_balance(payer_account).await.unwrap()
    );

    assert!(false); // to see the logs
}
