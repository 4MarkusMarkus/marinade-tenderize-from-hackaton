import {
  Account, Connection, PublicKey, sendAndConfirmTransaction, StakeProgram, SystemProgram, SYSVAR_CLOCK_PUBKEY, SYSVAR_RENT_PUBKEY, SYSVAR_STAKE_HISTORY_PUBKEY, Transaction, TransactionInstruction,
} from '@solana/web3.js';

const SPL_TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

export interface CreateStakePoolParams {
  feeDenominator: number,
  feeNumerator: number,
};

export interface CreateValidatorStakeParams {
  validator: PublicKey,
  stakePoolDepositAuthority: PublicKey, // TODO calculate automaticaly
  stakePoolWithdrawAuthority: PublicKey, // TODO calculate automaticaly
}

export interface AddValidatorParams {
  stakePoolDepositAuthority: PublicKey, // TODO calculate automaticaly
  stakePoolWithdrawAuthority: PublicKey, // TODO calculate automaticaly
  validator: PublicKey,
}

export interface DepositParams {
  stakePoolDepositAuthority: PublicKey, // TODO calculate automaticaly
  stakePoolWithdrawAuthority: PublicKey, // TODO calculate automaticaly
  input: PublicKey,
  validatorsStake: PublicKey,
  outputTokenAccount: PublicKey,
  feeTokenAccount: PublicKey,
}

export interface TestDepositParams {
  amount: number,
  userWallet: Account,
  validators: PublicKey[]
}

export class TenderizeProgram {
  connection: Connection;
  payerAccount: Account;
  programAccount: Account;
  stakePool: Account;
  owner: Account;
  validatorStakeListAccount: Account;
  poolMintToken: PublicKey;
  ownersFee: PublicKey;

  constructor(
    connection: Connection,
    payerAccount: Account,
    programAccount: Account,
    stakePool: Account,
    owner: Account,
    validatorStakeListAccount: Account,
    poolMintToken: PublicKey,
    ownersFee: PublicKey) {
    this.connection = connection;
    this.payerAccount = payerAccount;
    this.programAccount = programAccount;
    this.stakePool = stakePool;
    this.owner = owner;
    this.validatorStakeListAccount = validatorStakeListAccount;
    this.poolMintToken = poolMintToken;
    this.ownersFee = ownersFee;
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
    console.log(`Create stake pool ${this.stakePool.publicKey} with owners fee ${this.ownersFee}`);
    const stakePoolLength = 1000 + 4 + 32 + 4 + 4 + 32 + 32 + 32 + 8 + 8 + 8 + 2 * 8;
    const validatorStakeListLength = 60000 + 4 + 4 + 1000 * (32 + 8 + 8);

    const transaction = new Transaction();
    transaction.add(SystemProgram.createAccount({
      fromPubkey: this.payerAccount.publicKey,
      newAccountPubkey: this.stakePool.publicKey,
      lamports: await this.connection.getMinimumBalanceForRentExemption(stakePoolLength),
      space: stakePoolLength,
      programId: this.programId
    }));
    transaction.add(SystemProgram.createAccount({
      fromPubkey: this.payerAccount.publicKey,
      newAccountPubkey: this.validatorStakeListAccount.publicKey,
      lamports: await this.connection.getMinimumBalanceForRentExemption(validatorStakeListLength),
      space: validatorStakeListLength,
      programId: this.programId
    }));
    transaction.add(this.createStakePoolInstruction(params));

    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount, this.owner, this.stakePool, this.validatorStakeListAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  createStakePoolInstruction(params: CreateStakePoolParams) {
    const data = Buffer.alloc(1 + 8 + 8);
    let p = data.writeUInt8(0, 0);
    p = data.writeBigUInt64LE(BigInt(params.feeDenominator), p);
    p = data.writeBigUInt64LE(BigInt(params.feeNumerator), p);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.owner.publicKey, isSigner: true, isWritable: false },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.poolMintToken, isSigner: false, isWritable: false },
        { pubkey: this.ownersFee, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
      ],
      programId: this.programAccount.publicKey,
      data,
    });
  }

  async createValidatorStake(params: CreateValidatorStakeParams) {
    console.log(`Create stake ${await this.getStakeForValidator(params.validator)} for validator ${params.validator}`);
    const transaction = new Transaction();
    transaction.add(await this.createValidatorStakeInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  async getStakeForValidator(validator: PublicKey) {
    return (await PublicKey.findProgramAddress([validator.toBuffer(), this.stakePool.publicKey.toBuffer()], this.programId))[0];
  }

  async createValidatorStakeInstruction(params: CreateValidatorStakeParams) {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(1, 0);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: false },
        { pubkey: this.payerAccount.publicKey, isSigner: true, isWritable: true },
        { pubkey: await this.getStakeForValidator(params.validator), isSigner: false, isWritable: true },
        { pubkey: params.validator, isSigner: false, isWritable: false },
        { pubkey: params.stakePoolDepositAuthority, isSigner: false, isWritable: false },
        { pubkey: params.stakePoolWithdrawAuthority, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: new PublicKey("StakeConfig11111111111111111111111111111111"), isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async addValidator(params: AddValidatorParams) {
    console.log(`Add validator ${params.validator} with stake ${await this.getStakeForValidator(params.validator)}`);
    const transaction = new Transaction();
    transaction.add(await this.addValidatorInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount, this.owner],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  async addValidatorInstruction(params: AddValidatorParams) {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(2, 0);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.owner.publicKey, isSigner: true, isWritable: false },
        { pubkey: params.stakePoolDepositAuthority, isSigner: false, isWritable: false },
        { pubkey: params.stakePoolWithdrawAuthority, isSigner: false, isWritable: false },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: await this.getStakeForValidator(params.validator), isSigner: false, isWritable: true },
        { pubkey: this.ownersFee, isSigner: false, isWritable: true }, // User account to receive pool tokens
        { pubkey: this.poolMintToken, isSigner: false, isWritable: true },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async deposit(params: DepositParams) {
    const transaction = new Transaction();
    transaction.add(this.depositInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  depositInstruction(params: DepositParams) {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(6, 0);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: params.stakePoolDepositAuthority, isSigner: false, isWritable: false },
        { pubkey: params.stakePoolWithdrawAuthority, isSigner: false, isWritable: false },
        { pubkey: params.input, isSigner: false, isWritable: true },
        { pubkey: params.validatorsStake, isSigner: false, isWritable: true },
        { pubkey: params.outputTokenAccount, isSigner: false, isWritable: true },
        { pubkey: params.feeTokenAccount, isSigner: false, isWritable: true },
        { pubkey: this.poolMintToken, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async testDeposit(params: TestDepositParams): Promise<void> {
    const transaction = new Transaction();
    transaction.add(await this.testDepositInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [params.userWallet],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  async testDepositInstruction(params: TestDepositParams) {
    const data = Buffer.alloc(1 + 8);
    let p = data.writeUInt8(10, 0);
    p = data.writeBigUInt64LE(BigInt(params.amount), p);

    const stakesKeys = await Promise.all(
      params.validators.map(async (key) => {
        return {
          pubkey: await this.getStakeForValidator(key),
          isSigner: false,
          isWritable: true
        }
      }));
    console.log(`Depositing ${params.amount} into ${stakesKeys.map(k => k.pubkey.toBase58())} accounts`);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: params.userWallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ...stakesKeys
      ],
      programId: this.programId,
      data,
    });
  }

}