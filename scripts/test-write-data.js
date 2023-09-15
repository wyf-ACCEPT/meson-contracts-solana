require('dotenv').config();
const {
  Connection, Keypair, Transaction, sendAndConfirmTransaction, TransactionInstruction, PublicKey
} = require('@solana/web3.js');

// const privateKeyString = process.env.PRIVATE_KEY;
const privateKeyString = '80,83,11,9,145,1,91,144,244,223,98,141,59,249,88,33,153,25,201,147,156,91,95,119,243,156,94,140,196,71,237,36,63,108,109,236,195,131,233,185,220,251,212,47,126,10,250,90,192,76,94,87,130,78,96,21,245,253,63,34,12,116,126,191'
const privateKeyBytes = new Uint8Array(privateKeyString.split(',').map(byteStr => parseInt(byteStr.trim(), 10)));
const aliceSigner = Keypair.fromSecretKey(privateKeyBytes)
console.log(aliceSigner)

async function main() {
  const url = 'https://api.devnet.solana.com'
  const connection = new Connection(url, 'confirmed');
  await connection.getVersion();
  console.log(privateKeyBytes)
  console.log('Private key bytes:', privateKeyBytes);
}

main()


// // Client
// console.log("My address:", pg.wallet.publicKey.toString());
// const balance = await pg.connection.getBalance(pg.wallet.publicKey);
// console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

// const programId = new web3.PublicKey(
//   "Gs35x84NqxTJMZLPtPZiEV61uwXMiTQE15e9nP2gbNdc"
// );

// async function callOnce(web3.PublicKey) {
//   // Find data account address
//   const [findAddress, bump] = await web3.PublicKey.findProgramAddress(
//     [Buffer.from("map"), user.toBuffer()],
//     programId
//   );
//   console.log(findAddress.toString(), bump);

//   // Call counter program
//   const transaction = new web3.Transaction().add(
//     new web3.TransactionInstruction({
//       keys: [
//         { pubkey: user, isSigner: false, isWritable: false },
//         { pubkey: findAddress, isSigner: false, isWritable: true },
//         {
//           pubkey: new web3.PublicKey("11111111111111111111111111111111"),
//           isSigner: false,
//           isWritable: false,
//         },
//       ],
//       programId: programId,
//     })
//   );

//   // send the transaction to the Solana cluster
//   console.log("Sending transaction...");
//   const txHash = await web3.sendAndConfirmTransaction(
//     pg.connection,
//     transaction,
//     [pg.wallet.keypair]
//   );
//   console.log("Transaction sent with hash:", txHash);
// }

// // // Watch the lamports needed to save the data
// // console.log(await createCounterAccount(64)); // for 1 encodedSwap
// // console.log(await createCounterAccount(6400)); // for ~100 encodedSwap
// // console.log(await createCounterAccount(64000)); // for ~1000 encodedSwap
// // console.log(await callOnce(pg.wallet.publicKey));
// const info = await pg.connection.getAccountInfo(
//   new web3.PublicKey("2pYESjLwKGuojjMuoqvcAN72BbJEaMWLK4PQFxA2kAQT")
// );
// console.log(info.data);
// console.log(info.owner.toString());
