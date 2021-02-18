import {
  Connection,
  Account,
  LAMPORTS_PER_SOL,
  PublicKey,
  TransactionInstruction,
  sendAndConfirmTransaction,
  Transaction,
  SystemProgram,
} from '@solana/web3.js';
// import { newAccountWithLamports } from './util/new-account-with-lamports';
import path from 'path';
import fs from 'mz/fs';
import { execShellCommand } from './util/shell';
import * as assert from 'assert';

export class Tester {
  connection: Connection;
  payerAccount: Account;

  private constructor(connection: Connection, payerAccount: Account) {
    this.connection = connection;
    this.payerAccount = payerAccount;
  }

  async getProgramId(name: String): Promise<PublicKey> {
    const keypair_file_name = path.join('..', 'program', 'dist', name + "-keypair.json");
    const keypair = JSON.parse(await fs.readFile(keypair_file_name, 'utf8'));
    const programAccount = new Account(keypair);
    return programAccount.publicKey;
  }

  async runProgram(id: PublicKey, stateAccount: Account): Promise<void> {
    const instruction = new TransactionInstruction({
      keys: [{ pubkey: stateAccount.publicKey, isSigner: true, isWritable: true }],
      programId: id,
      data: Buffer.alloc(0),
    });
    await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(instruction),
      [this.payerAccount, stateAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      },
    );
  }

  async makeAccount(owner: PublicKey, lamports = 100000000, space = 100): Promise<Account> {
    const account = new Account();
    const transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: this.payerAccount.publicKey,
        newAccountPubkey: account.publicKey,
        lamports,
        space,
        programId: owner,
      }),
    );

    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount, account],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      },
    );

    const l = await this.connection.getBalance(account.publicKey);
    console.log(
      'Making account',
      account.publicKey.toBase58(),
      'containing',
      l / LAMPORTS_PER_SOL,
      'Sol to pay for fees',
    );

    return account;
  }

  static async build(): Promise<Tester> {
    const solana_config = (await execShellCommand('solana config get')).split('\n');
    let url = null;
    let payerAccount = null;
    for (let line of solana_config) {
      line = line.trimRight();
      let m = line.match(/^RPC URL: (.*)$/i);
      if (m) {
        url = m[1];
      }

      m = line.match(/^Keypair Path: (.*)$/);
      if (m) {
        const keypair = await fs.readFile(m[1], 'utf8');
        payerAccount = new Account(JSON.parse(keypair));
      }
    }
    assert.ok(url, "Can't parse solana config RPC URL");
    assert.ok(payerAccount, "Can't parse solana config account");

    const connection = new Connection(url!, 'singleGossip');
    const version = await connection.getVersion();
    console.log('Connection to cluster established:', url, version);

    const lamports = await connection.getBalance(payerAccount!.publicKey);
    console.log(
      'Using account',
      payerAccount!.publicKey.toBase58(),
      'containing',
      lamports / LAMPORTS_PER_SOL,
      'Sol to pay for fees',
    );

    return new Tester(connection, payerAccount!);
  }
}
