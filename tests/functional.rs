use std::str::FromStr;

use solana_program::system_program;

use {
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_program_test::*,
    solana_sdk::{signature::Signer, transaction::Transaction},
    meson_contracts_solana::entrypoint::process_instruction,
};

#[tokio::test]
async fn test_write() {
    let program_id = Pubkey::from_str("Meson11111111111111111111111111111111111111").unwrap();

    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "meson_contracts_solana",
        program_id,
        processor!(process_instruction),
    ).start().await;

    let user_account = payer.pubkey();
    // let user_account = Pubkey::new_unique();
    let (map_pda, _) =
        Pubkey::find_program_address(&[b"hello", user_account.as_ref()], &program_id);

    println!("Program ID: {}", program_id);
    println!("{:?}", program_id.as_ref());

    let transaction = Transaction::new_signed_with_payer(
        &[
            Instruction::new_with_bincode(
                program_id, 
                &(), 
                vec![
                    AccountMeta::new(user_account, false),
                    AccountMeta::new(map_pda, false),
                    AccountMeta::new(system_program::id(), false),
                ])
        ], 
        Some(&payer.pubkey()), 
        &[&payer], 
        recent_blockhash
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let account_data = banks_client
        .get_account(map_pda)
        .await.unwrap().unwrap();
    println!("{:?}", account_data);

    assert!(false);

}
