use std::str::FromStr;

use solana_program::system_program;

use {
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey
    },
    solana_program_test::*,
    solana_sdk::{signature::Signer, transaction::Transaction},
    meson_contracts_solana::entrypoint::process_instruction,
};

async fn show_account_info(banks_client: &mut BanksClient, account: Pubkey) {
    let program_id_data = banks_client
        .get_account(account)
        .await.unwrap().unwrap();
    println!("[AccountInfo {}]\nProgram data: {:?}\n", account, program_id_data);
}

#[tokio::test]
async fn test_write() {
    let program_id = Pubkey::from_str("Meson11111111111111111111111111111111111111").unwrap();

    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "meson_contracts_solana",
        program_id,
        processor!(process_instruction),
    ).start().await;

    let payer_account = payer.pubkey();
    // let payer_account = Pubkey::new_unique();
    // let (map_pda, _) =
    //     Pubkey::find_program_address(&[b"hello", b"world"], &program_id);

    println!("Program ID: {}", program_id);
    // println!("{:?}", program_id.as_ref());

    let (auth_pda, _) = Pubkey::find_program_address(&[b"authority"], &program_id);
    let (token_pda, _) = Pubkey::find_program_address(&[b"supported_coins"], &program_id);

    let transaction = Transaction::new_signed_with_payer(
        &[
            Instruction::new_with_bincode(
                program_id, 
                &(), 
                vec![
                    AccountMeta::new(payer_account, false),
                    AccountMeta::new(system_program::id(), false),
                    AccountMeta::new(token_pda, false),
                    AccountMeta::new(auth_pda, false),
                ])
        ], 
        Some(&payer.pubkey()), 
        &[&payer], 
        recent_blockhash
    );
    banks_client.process_transaction(transaction).await.unwrap();

    show_account_info(&mut banks_client, payer_account).await;
    show_account_info(&mut banks_client, auth_pda).await;
    show_account_info(&mut banks_client, token_pda).await;
    show_account_info(&mut banks_client, program_id).await;

    assert!(false);     // to see the logs
}
