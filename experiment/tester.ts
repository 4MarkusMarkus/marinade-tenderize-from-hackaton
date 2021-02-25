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
import { TenderizeProgram } from './tenderize';

export class Tester {
  connection: Connection;
  payerAccount: Account;
  userTokenAccount: Account;
  tenderize?: TenderizeProgram;

  private constructor(connection: Connection, payerAccount: Account, userTokenAccount: Account) {
    this.connection = connection;
    this.payerAccount = payerAccount;
    this.userTokenAccount = userTokenAccount;
  }

  async initTenderize() {
    this.tenderize = new TenderizeProgram(
      this.connection,
      this.payerAccount,
      await Tester.loadAccount("solana_bpf_tenderize-keypair"),
      await Tester.loadAccount("stake_pool"),
      this.payerAccount, // owner
      await Tester.loadAccount("validator_list"),
      await Tester.loadAccount('credit_list'),
      (await Tester.loadAccount('tSOL_token')).publicKey,
      (await Tester.loadAccount('owners_fee')).publicKey,
      (await Tester.loadAccount('credit_reserve')).publicKey);
  }

  static async loadAccount(name: String): Promise<Account> {
    const keypair_file_name = path.join('..', 'keys', name + '.json');
    return new Account(JSON.parse(await fs.readFile(keypair_file_name, 'utf-8')));
  }

  static async saveAccount(name: String, account: Account): Promise<void> {
    console.log(account.secretKey);
    await fs.writeFile(
      path.join('..', 'keys', name + '.json'),
      JSON.stringify(account.secretKey));
  }

  static async loadOrNewAccount(name: String): Promise<Account> {
    try {
      return await Tester.loadAccount(name);
    } catch (e) {
      const account = new Account();
      await Tester.saveAccount(name, account)
      return account;
    }
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

  async createStakePool(): Promise<void> {
    await this.tenderize!.createStakePool({
      feeDenominator: 100,
      feeNumerator: 3
    });
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

    return new Tester(connection, payerAccount!, await Tester.loadAccount("user_tokens"));
  }

  async getValidators() {
    const votes = await this.connection.getVoteAccounts('singleGossip');
    return votes.current.map((v) => new PublicKey(v.votePubkey));
  }
}
