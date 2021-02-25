import {
  Account, Connection, PublicKey, sendAndConfirmTransaction, StakeProgram, SystemProgram, SYSVAR_CLOCK_PUBKEY, SYSVAR_RENT_PUBKEY, SYSVAR_STAKE_HISTORY_PUBKEY, Transaction, TransactionInstruction,
} from '@solana/web3.js';

const SPL_TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const MIN_STAKE_ACCOUNT_BALANCE = 1000000000;

export interface ValidatorInfo {
  votePubkey: PublicKey,
  balance: number,
  lastUpdateEpoch: number,
  stakeCount: number
}

export interface CreateStakePoolParams {
  feeDenominator: number,
  feeNumerator: number,
};

export interface CreateValidatorStakeParams {
  validator: PublicKey,
}

export interface AddValidatorParams {
  validator: PublicKey,
}

export interface DepositParams {
  userSource: Account,
  amount: number,
  userToken: PublicKey,
}

export interface TestDepositParams {
  amount: number,
  userWallet: Account,
  validators: PublicKey[]
}

export interface TestWithdrawParams {
  amount: number,
  userWallet: Account,
  validators: PublicKey[]
}

export interface DepositReserveValidatorParam {
  address: PublicKey,
  amount: number,
  stakeIndex: number
}

export interface DepositReserveParams {
  validators: DepositReserveValidatorParam[]
}

export interface UpdateListBalanceParams {
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

  async getReserveAddress(): Promise<PublicKey> {
    return (await PublicKey.findProgramAddress([this.stakePool.publicKey.toBuffer(), Buffer.from('reserve')], this.programId))[0];
  }

  async getDepositAuthority(): Promise<PublicKey> {
    return (await PublicKey.findProgramAddress([this.stakePool.publicKey.toBuffer(), Buffer.from('deposit')], this.programId))[0];
  }

  async getWithdrawAuthority(): Promise<PublicKey> {
    return (await PublicKey.findProgramAddress([this.stakePool.publicKey.toBuffer(), Buffer.from('withdraw')], this.programId))[0];
  }

  async getStakeForValidator(validator: PublicKey, index: number) {
    const indexBuffer = Buffer.alloc(4);
    indexBuffer.writeUInt32LE(index, 0);
    return (await PublicKey.findProgramAddress([validator.toBuffer(), this.stakePool.publicKey.toBuffer(), indexBuffer], this.programId))[0];
  }

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

  async readState() {
    const stateAccount = await this.connection.getAccountInfo(this.stakePool.publicKey, 'singleGossip');

  }

  async readValidators(): Promise<ValidatorInfo[]> {
    const validatorListAccount = await this.connection.getAccountInfo(this.validatorStakeListAccount.publicKey, 'singleGossip');
    const validatorCount = validatorListAccount!.data.readUInt16LE(1);
    const validators: ValidatorInfo[] = []
    for (let i = 0; i < validatorCount; ++i) {
      validators.push({
        votePubkey: new PublicKey(validatorListAccount!.data.slice(3 + (32 + 8 + 8 + 4) * i, 3 + (32 + 8 + 8 + 4) * i + 32)),
        balance: Number(validatorListAccount!.data.readBigUInt64LE(3 + (32 + 8 + 8 + 4) * i + 32)),
        lastUpdateEpoch: Number(validatorListAccount!.data.readBigUInt64LE(3 + (32 + 8 + 8 + 4) * i + 32 + 8)),
        stakeCount: validatorListAccount!.data.readUInt32LE(3 + (32 + 8 + 8 + 4) * i + 32 + 8 + 8),
      })
    }
    return validators;
  }

  async addValidator(params: AddValidatorParams) {
    console.log(`Add validator ${params.validator}`);
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
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: false },
        { pubkey: this.owner.publicKey, isSigner: true, isWritable: false },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: params.validator, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async deposit(params: DepositParams) {
    console.log(`Deposit ${params.amount}`);
    const transaction = new Transaction();
    transaction.add(await this.depositInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  async depositInstruction(params: DepositParams) {
    const data = Buffer.alloc(1 + 8);
    let p = data.writeUInt8(6, 0);
    p = data.writeBigInt64LE(BigInt(params.amount), p);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: await this.getWithdrawAuthority(), isSigner: false, isWritable: false },
        { pubkey: await this.getReserveAddress(), isSigner: false, isWritable: true },
        { pubkey: params.userSource.publicKey, isSigner: true, isWritable: true },
        { pubkey: params.userToken, isSigner: false, isWritable: true },
        { pubkey: this.ownersFee, isSigner: false, isWritable: true },
        { pubkey: this.poolMintToken, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }
  /*
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
  
      const keys = [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: params.stakePoolDepositAuthority, isSigner: false, isWritable: false },
        { pubkey: params.userWallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: new PublicKey("StakeConfig11111111111111111111111111111111"), isSigner: false, isWritable: false },];
  
      console.log(`Depositing ${params.amount} into`);
      for (const validator of params.validators) {
        const stake = await this.getStakeForValidator(validator);
        console.log(`Validator ${validator.toBase58()} stake ${stake.toBase58()}`);
        keys.push({
          pubkey: stake,
          isSigner: false,
          isWritable: true
        });
  
        keys.push({
          pubkey: validator,
          isSigner: false,
          isWritable: false
        })
      }
  
      return new TransactionInstruction({
        keys,
        programId: this.programId,
        data,
      });
    }
  
    async testWithdraw(params: TestWithdrawParams): Promise<void> {
      const transaction = new Transaction();
      transaction.add(await this.testWithdrawInstruction(params));
      await sendAndConfirmTransaction(
        this.connection,
        transaction,
        [params.userWallet],
        {
          commitment: 'singleGossip',
          preflightCommitment: 'singleGossip'
        });
    }
  
    async testWithdrawInstruction(params: TestWithdrawParams) {
      const data = Buffer.alloc(1 + 8);
      let p = data.writeUInt8(11, 0);
      p = data.writeBigUInt64LE(BigInt(params.amount), p);
  
      const keys = [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: params.stakePoolWithdrawAuthority, isSigner: false, isWritable: false },
        { pubkey: params.userWallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },];
  
      console.log(`Whithdraw ${params.amount} from`);
      for (const validator of params.validators) {
        const stake = await this.getStakeForValidator(validator);
        console.log(`Validator ${validator.toBase58()} stake ${stake.toBase58()}`);
        keys.push({
          pubkey: stake,
          isSigner: false,
          isWritable: true
        });
      }
  
      return new TransactionInstruction({
        keys,
        programId: this.programId,
        data,
      });
    }
  */

  async delegateReserve(params: DepositReserveParams): Promise<void> {
    const transaction = new Transaction();
    transaction.add(await this.delegateReserveInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  async delegateReserveInstruction(params: DepositReserveParams) {
    const data = Buffer.alloc(1 + 4 + (8 + 8) * params.validators.length);
    let p = data.writeUInt8(12, 0);
    p = data.writeUInt32LE(params.validators.length, p);

    const keys = [
      { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
      { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: await this.getWithdrawAuthority(), isSigner: false, isWritable: false },
      { pubkey: await this.getDepositAuthority(), isSigner: false, isWritable: false },
      { pubkey: await this.getReserveAddress(), isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: new PublicKey("StakeConfig11111111111111111111111111111111"), isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },];

    console.log(`Delegate from reserve into`);
    for (const validator of params.validators) {
      p = data.writeBigUInt64LE(BigInt(validator.amount), p);
      p = data.writeBigUInt64LE(BigInt(validator.stakeIndex), p);

      keys.push({
        pubkey: validator.address,
        isSigner: false,
        isWritable: false
      })

      const stake = await this.getStakeForValidator(validator.address, validator.stakeIndex);
      console.log(`Validator ${validator.address.toBase58()} with stake #${validator.stakeIndex} ${stake.toBase58()}`);
      keys.push({
        pubkey: stake,
        isSigner: false,
        isWritable: true
      });
    }

    return new TransactionInstruction({
      keys,
      programId: this.programId,
      data,
    });
  }

  async delegateReserveBatch(totalAmount: number) {
    if (totalAmount < MIN_STAKE_ACCOUNT_BALANCE) {
      throw Error("Too low delegation");
    }

    const validators = await this.readValidators();
    const stakeTotal = validators.map((v) => v.balance).reduce((a, b) => a + b, 0);
    const targetAmount = Math.ceil((stakeTotal + totalAmount) / validators.length);

    const instructions: DepositReserveValidatorParam[] = [];
    let amountLeft = totalAmount;
    for (const validator of validators) {
      if (amountLeft < 1) {
        break;
      }

      if (validator.balance >= targetAmount) {
        continue;
      }

      let delegateAmount = Math.min(targetAmount - validator.balance, amountLeft);

      amountLeft -= delegateAmount;

      if (amountLeft < MIN_STAKE_ACCOUNT_BALANCE) {
        delegateAmount += amountLeft;
        amountLeft = 0;
      }

      let currentEpochStakeIndex = -1;
      for (let index = 0; index < validator.stakeCount; index++) {
        const stakeAddress = await this.getStakeForValidator(validator.votePubkey, index);
        const stakeData = await this.connection.getStakeActivation(stakeAddress, 'singleGossip');
        if ((stakeData.state == 'activating') && (stakeData.active == 0)) {
          currentEpochStakeIndex = index;
          break;
        }
      }

      if (currentEpochStakeIndex >= 0) {
        console.log(`Redelegate to stake #${currentEpochStakeIndex} ${await this.getStakeForValidator(validator.votePubkey, currentEpochStakeIndex)}`);
        instructions.push({
          address: validator.votePubkey,
          amount: delegateAmount,
          stakeIndex: currentEpochStakeIndex
        });
        continue;
      }

      let firstFreeIndex = validator.stakeCount;
      for (let index = 0; index < validator.stakeCount; index++) {
        const stakeAddress = await this.getStakeForValidator(validator.votePubkey, index);
        const stakeAccount = await this.connection.getAccountInfo(stakeAddress);
        if (!stakeAccount || (stakeAccount.owner == SystemProgram.programId)) {
          firstFreeIndex = index;
        }
      }

      console.log(`Init stake #${firstFreeIndex} ${await this.getStakeForValidator(validator.votePubkey, firstFreeIndex)}`);
      instructions.push({
        address: validator.votePubkey,
        amount: delegateAmount,
        stakeIndex: firstFreeIndex
      });
    }

    if (amountLeft > 0) {
      throw Error(`Left ${amountLeft}`);
    }

    await this.delegateReserve({
      validators: instructions
    })
  }

  async updateListBalanceReserve(params: UpdateListBalanceParams): Promise<void> {
    const transaction = new Transaction();
    transaction.add(await this.updateListBalanceInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  async updateListBalanceInstruction(params: UpdateListBalanceParams): Promise<TransactionInstruction> {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(4, 0);

    const keys = [
      { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: false },
      { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },];

    const allValidators = await this.readValidators();
    for (const validator of params.validators) {
      keys.push({
        pubkey: validator,
        isSigner: false,
        isWritable: false
      })

      const validatorInfo = allValidators.find((v) => v.votePubkey.equals(validator));
      for (let i = 0; i < validatorInfo!.stakeCount; ++i) {
        const stake = await this.getStakeForValidator(validator, i);
        keys.push({
          pubkey: stake,
          isSigner: false,
          isWritable: false
        });
      }
    }

    return new TransactionInstruction({
      keys,
      programId: this.programId,
      data,
    });
  }

  async updatePoolBalance(): Promise<void> {
    const transaction = new Transaction();
    transaction.add(await this.updatePoolBalanceInstruction());
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }

  async updatePoolBalanceInstruction(): Promise<TransactionInstruction> {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(5, 0);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.validatorStakeListAccount.publicKey, isSigner: false, isWritable: false },
        { pubkey: await this.getReserveAddress(), isSigner: false, isWritable: false },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async updatePool(): Promise<void> {
    console.log("Updating pool");
    const transaction = new Transaction();
    transaction.add(await this.updateListBalanceInstruction({
      validators: (await this.readValidators()).map((v) => v.votePubkey)
    }));
    transaction.add(await this.updatePoolBalanceInstruction());
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip'
      });
  }
}