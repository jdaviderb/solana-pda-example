import {describe, expect, test} from '@jest/globals';
import { Connection, Transaction, TransactionInstruction, PublicKey, Keypair, sendAndConfirmTransaction, SystemProgram } from '@solana/web3.js';
import { wait } from '../utils';


// in the account structure the first 32 bytes is the owner account and after that is the counter value

describe('Counter Program', () => {
  const programId = new PublicKey(process.env['SOLANA_PROGRAM_ID'] as string);
  const wallet = Keypair.generate();
  const port = process.env['RPC_PORT'];
  const connection = new Connection(`http://127.0.0.1:${port}`, 'confirmed');


  test('Initialize Counter', async () => {
    await connection.requestAirdrop(wallet.publicKey, 5000000000);
    await wait(1000);
   
    const counter = await PublicKey.createWithSeed(
      wallet.publicKey,
      'counter',
      programId,
    );

    const [counter2] = PublicKey.findProgramAddressSync([wallet.publicKey.toBuffer(), Buffer.from("counter")], programId);
    
    const InitInstruction = new TransactionInstruction({
      keys: [
        { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: counter2, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false}
      ],
      programId: new PublicKey(programId),
      data: Buffer.from([0]), // 0: Initialize, 1: Increment
    })

    const transaction = new Transaction()
      .add(InitInstruction);
    transaction.feePayer = wallet.publicKey;

    await sendAndConfirmTransaction(connection, transaction, [wallet]);

    const counterData = await connection.getAccountInfo(counter2);
    const owner = counterData?.data.slice(0, 32) as Buffer;
    const counterValue = (counterData?.data.slice(32, 33) as Buffer).readUint8(0);

    expect(wallet.publicKey.toBuffer().equals(owner)).toBe(true);
    expect(counterValue).toBe(1)
  });

  test("Increment Counter", async () => {
    const [counter] = PublicKey.findProgramAddressSync([wallet.publicKey.toBuffer(), Buffer.from("counter")], programId);

    const increaseInstruction = new TransactionInstruction({
      keys: [
        { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: counter, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
      ],
      programId: new PublicKey(programId),
      data: Buffer.from([1]) // 0: Initialize, 1: Increment,
    });

    const transaction = new Transaction().add(increaseInstruction);

    await sendAndConfirmTransaction(connection, transaction, [wallet]);

    const counterData = await connection.getAccountInfo(counter);
    const owner = counterData?.data.slice(0, 32) as Buffer;
    const counterValue = (counterData?.data.slice(32, 33) as Buffer).readUint8(0);

    expect(wallet.publicKey.toBuffer().equals(owner)).toBe(true);
    expect(counterValue).toBe(2);
  });
});
