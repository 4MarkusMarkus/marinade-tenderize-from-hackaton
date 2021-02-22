import {
  Account, Connection, PublicKey, sendAndConfirmTransaction, SystemProgram, SYSVAR_CLOCK_PUBKEY, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction,
} from '@solana/web3.js';

export interface CreateStakePoolParams {
  feeDenominator: number,
  feeNumerator: number,
  stakePool: Account,
  owner: Account,
  mint: PublicKey,
  ownersFee: PublicKey
};

export class TenderizeProgram {
  connection: Connection;
  payerAccount: Account;
  programAccount: Account;

  constructor(connection: Connection, payerAccount: Account, programAccount: Account) {
    this.connection = connection;
    this.payerAccount = payerAccount;
    this.programAccount = programAccount;
  }

  get programId(): PublicKey {
    return this.programAccount.publicKey;
  }

  /*
  static async getStakePoolMintAuthority(stakePool: PublicKey): Promise<PublicKey> {
    return (await PublicKey.findProgramAddress([stakePool.toBuffer(), new TextEncoder().encode('withdraw')], stakePool))[0];
  }
  */

  async createStakePool(params: CreateStakePoolParams): Promise<void> {
    const stakePoolLength = 1000 + 4 + 32 + 4 + 4 + 32 + 32 + 32 + 8 + 8 + 8 + 2 * 8;

    const validatorStakeListAccount = new Account();
    const validatorStakeListLength = 60000 + 4 + 4 + 1000 * (32 + 8 + 8);

    const transaction = new Transaction();
    transaction.add(SystemProgram.createAccount({
      fromPubkey: this.payerAccount.publicKey,
      newAccountPubkey: params.stakePool.publicKey,
      lamports: await this.connection.getMinimumBalanceForRentExemption(stakePoolLength),
      space: stakePoolLength,
      programId: this.programId
    }));
    transaction.add(SystemProgram.createAccount({
      fromPubkey: this.payerAccount.publicKey,
      newAccountPubkey: validatorStakeListAccount.publicKey,
      lamports: await this.connection.getMinimumBalanceForRentExemption(validatorStakeListLength),
      space: validatorStakeListLength,
      programId: this.programId
    }));
    transaction.add(TenderizeProgram.createStakePoolInstruction(
      this.programAccount,
      params,
      params.stakePool.publicKey,
      validatorStakeListAccount.publicKey));

    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount, params.owner, params.stakePool, validatorStakeListAccount]);
  }

  static createStakePoolInstruction(programAccount: Account, params: CreateStakePoolParams, stakePool: PublicKey, validatorStakeList: PublicKey) {
    const data = Buffer.alloc(4 + 8 + 8);
    data.writeUInt32LE(0, 0);
    data.writeBigUInt64LE(BigInt(params.feeDenominator), 4);
    data.writeBigUInt64LE(BigInt(params.feeNumerator), 4 + 8);

    return new TransactionInstruction({
      keys: [
        { pubkey: stakePool, isSigner: false, isWritable: true },
        { pubkey: params.owner.publicKey, isSigner: true, isWritable: false },
        { pubkey: validatorStakeList, isSigner: false, isWritable: true },
        { pubkey: params.mint, isSigner: false, isWritable: false },
        { pubkey: params.ownersFee, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false } // spl-token program
      ],
      programId: programAccount.publicKey,
      data,
    });
  }

}