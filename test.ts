import {
    Account,
    Connection,
    PublicKey,
    LAMPORTS_PER_SOL,
    SystemProgram,
    TransactionInstruction,
    Transaction,
    sendAndConfirmTransaction,
} from '@solana/web3.js';

let connection: Connection;
let payerAccount: Account;

async function main() {
    let pk = new PublicKey("8RzvJghJQYZBV6k6nU6adCLSBT4DEVNDxTs9v6EgoKaa");
    connection = new Connection("http://localhost:8899");
    let fees = 0;
    const { feeCalculator } = await connection.getRecentBlockhash();
    fees += feeCalculator.lamportsPerSignature * 100;
    payerAccount = new Account();
    const sig = await connection.requestAirdrop(
        payerAccount.publicKey,
        fees,
    );
    await connection.confirmTransaction(sig);
    console.log('Sending txn to', pk.toBase58());
    const instruction = new TransactionInstruction({
        keys: [],
        programId: pk,
        data: Buffer.from([0]),
    });
    await sendAndConfirmTransaction(
        connection,
        new Transaction().add(instruction),
        [payerAccount],
    );
}

main().then(
    () => console.log("success!"),
    err => console.error(err),
);
