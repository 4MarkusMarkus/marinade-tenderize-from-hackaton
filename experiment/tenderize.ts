import {
  Account,
  Connection,
  PublicKey,
  sendAndConfirmTransaction,
  StakeProgram,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_STAKE_HISTORY_PUBKEY,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';

const SPL_TOKEN_PROGRAM_ID = new PublicKey(
  'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
);
const MIN_STAKE_ACCOUNT_BALANCE = 1000000000;

export interface ValidatorInfo {
  votePubkey: PublicKey;
  balance: number;
  lastUpdateEpoch: number;
  stakeCount: number;
}

export interface CreateStakePoolParams {
  feeDenominator: number;
  feeNumerator: number;
}

export interface CreateValidatorStakeParams {
  validator: PublicKey;
}

export interface AddValidatorParams {
  validator: PublicKey;
}

export interface DepositParams {
  userSource: Account;
  amount: number;
  userToken: PublicKey;
}

export interface WithdrawParams {
  userTokenSource: PublicKey;
  amount: number;
  userSolTarget: PublicKey;
}

export interface CreditParams {
  userTokenSource: PublicKey;
  amount: number;
  userSolTarget: PublicKey;
  cancelAuthority: PublicKey | Account;
}

export interface TestDepositParams {
  amount: number;
  userWallet: Account;
  validators: PublicKey[];
}

export interface TestWithdrawParams {
  amount: number;
  userWallet: Account;
  validators: PublicKey[];
}

export interface DepositReserveValidatorParam {
  address: PublicKey;
  amount: number;
  stakeIndex: number;
}

export interface DepositReserveParams {
  validators: DepositReserveValidatorParam[];
}

export interface StakePair {
  validator: PublicKey;
  mainIndex: number;
  additionalIndex: number;
}

export interface MergeStakesParams {
  stakePairs: StakePair[];
}

export interface Unstake {
  validator: PublicKey;
  sourceIndex: number;
  splitIndex?: number;
  amount?: number;
}

export interface UnstakeParams {
  unstakes: Unstake[];
}

export interface UpdateListBalanceParams {
  validators: PublicKey[];
}

export interface State {
  version: number; // >0 for initialized
  owner: PublicKey;
  validatorStakeList: PublicKey;
  creditList: PublicKey;
  poolMint: PublicKey;
  ownerFeeAccount: PublicKey;
  creditReserve: PublicKey;
  tokenProgram: PublicKey;
  stakeTotal: bigint;
  poolTotal: bigint;
  lastEpochUpdate: bigint;
  feeDenominator: bigint;
  feeNumerator: bigint;
}

export interface Creditor {
  target: PublicKey;
  cancelAuthority: PublicKey;
  amount: number;
}

export class TenderizeProgram {
  connection: Connection;
  payerAccount: Account;
  programAccount: Account;
  stakePool: Account;
  owner: Account;
  validatorStakeListAccount: Account;
  creditListAccount: Account;
  poolMintToken: PublicKey;
  ownersFee: PublicKey;
  creditReserve: PublicKey;

  constructor(
    connection: Connection,
    payerAccount: Account,
    programAccount: Account,
    stakePool: Account,
    owner: Account,
    validatorStakeListAccount: Account,
    creditListAccount: Account,
    poolMintToken: PublicKey,
    ownersFee: PublicKey,
    creditReserve: PublicKey
  ) {
    this.connection = connection;
    this.payerAccount = payerAccount;
    this.programAccount = programAccount;
    this.stakePool = stakePool;
    this.owner = owner;
    this.validatorStakeListAccount = validatorStakeListAccount;
    this.creditListAccount = creditListAccount;
    this.poolMintToken = poolMintToken;
    this.ownersFee = ownersFee;
    this.creditReserve = creditReserve;
  }

  get programId(): PublicKey {
    return this.programAccount.publicKey;
  }

  async getReserveAddress(): Promise<PublicKey> {
    return (
      await PublicKey.findProgramAddress(
        [this.stakePool.publicKey.toBuffer(), Buffer.from('reserve')],
        this.programId
      )
    )[0];
  }

  async getDepositAuthority(): Promise<PublicKey> {
    return (
      await PublicKey.findProgramAddress(
        [this.stakePool.publicKey.toBuffer(), Buffer.from('deposit')],
        this.programId
      )
    )[0];
  }

  async getWithdrawAuthority(): Promise<PublicKey> {
    return (
      await PublicKey.findProgramAddress(
        [this.stakePool.publicKey.toBuffer(), Buffer.from('withdraw')],
        this.programId
      )
    )[0];
  }

  async getStakeForValidator(validator: PublicKey, index: number) {
    const indexBuffer = Buffer.alloc(4);
    indexBuffer.writeUInt32LE(index, 0);
    return (
      await PublicKey.findProgramAddress(
        [
          validator.toBuffer(),
          this.stakePool.publicKey.toBuffer(),
          indexBuffer,
        ],
        this.programId
      )
    )[0];
  }

  async createStakePool(params: CreateStakePoolParams): Promise<void> {
    console.log(
      `Creating stake pool ${this.stakePool.publicKey} with owners fee ${this.ownersFee}`
    );
    const stakePoolLength =
      1000 + 4 + 32 + 4 + 4 + 32 + 32 + 32 + 8 + 8 + 8 + 2 * 8;
    const validatorStakeListLength = 60000 + 4 + 4 + 1000 * (32 + 8 + 8);
    const creditListLength = 5 + 10000 * (32 + 32 + 32);

    const transaction = new Transaction();
    transaction.add(
      SystemProgram.createAccount({
        fromPubkey: this.payerAccount.publicKey,
        newAccountPubkey: this.stakePool.publicKey,
        lamports: await this.connection.getMinimumBalanceForRentExemption(
          stakePoolLength
        ),
        space: stakePoolLength,
        programId: this.programId,
      })
    );
    transaction.add(
      SystemProgram.createAccount({
        fromPubkey: this.payerAccount.publicKey,
        newAccountPubkey: this.validatorStakeListAccount.publicKey,
        lamports: await this.connection.getMinimumBalanceForRentExemption(
          validatorStakeListLength
        ),
        space: validatorStakeListLength,
        programId: this.programId,
      })
    );
    transaction.add(
      SystemProgram.createAccount({
        fromPubkey: this.payerAccount.publicKey,
        newAccountPubkey: this.creditListAccount.publicKey,
        lamports: await this.connection.getMinimumBalanceForRentExemption(
          creditListLength
        ),
        space: creditListLength,
        programId: this.programId,
      })
    );
    transaction.add(this.createStakePoolInstruction(params));

    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [
        this.payerAccount,
        this.owner,
        this.stakePool,
        this.validatorStakeListAccount,
        this.creditListAccount,
      ],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
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
        {
          pubkey: this.validatorStakeListAccount.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: this.creditListAccount.publicKey,
          isSigner: false,
          isWritable: true,
        },
        { pubkey: this.poolMintToken, isSigner: false, isWritable: false },
        { pubkey: this.ownersFee, isSigner: false, isWritable: false },
        { pubkey: this.creditReserve, isSigner: false, isWritable: true },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      programId: this.programAccount.publicKey,
      data,
    });
  }

  async readState(): Promise<State | null> {
    const stateAccount = await this.connection.getAccountInfo(
      this.stakePool.publicKey,
      'singleGossip'
    );

    if (!stateAccount || stateAccount.data.length == 0) {
      return null;
    }

    return {
      version: stateAccount.data.readUInt8(0),
      owner: new PublicKey(stateAccount.data.slice(1, 33)),
      // 2 bytes of bump seeds. Not useful
      validatorStakeList: new PublicKey(stateAccount.data.slice(35, 67)),
      creditList: new PublicKey(stateAccount.data.slice(67, 99)),
      poolMint: new PublicKey(stateAccount.data.slice(99, 131)),
      ownerFeeAccount: new PublicKey(stateAccount.data.slice(131, 163)),
      creditReserve: new PublicKey(stateAccount.data.slice(163, 195)),
      tokenProgram: new PublicKey(stateAccount.data.slice(195, 227)),
      // Padding
      stakeTotal: stateAccount.data.readBigUInt64LE(232),
      poolTotal: stateAccount.data.readBigUInt64LE(240),
      lastEpochUpdate: stateAccount.data.readBigUInt64LE(248),
      feeDenominator: stateAccount.data.readBigUInt64LE(256),
      feeNumerator: stateAccount.data.readBigUInt64LE(264),
    }
  }

  async readValidators(): Promise<ValidatorInfo[]> {
    const validatorListAccount = await this.connection.getAccountInfo(
      this.validatorStakeListAccount.publicKey,
      'singleGossip'
    );
    const validatorCount = validatorListAccount!.data.readUInt16LE(1);
    const validators: ValidatorInfo[] = [];
    for (let i = 0; i < validatorCount; ++i) {
      validators.push({
        votePubkey: new PublicKey(
          validatorListAccount!.data.slice(
            3 + (32 + 8 + 8 + 4) * i,
            3 + (32 + 8 + 8 + 4) * i + 32
          )
        ),
        balance: Number(
          validatorListAccount!.data.readBigUInt64LE(
            3 + (32 + 8 + 8 + 4) * i + 32
          )
        ),
        lastUpdateEpoch: Number(
          validatorListAccount!.data.readBigUInt64LE(
            3 + (32 + 8 + 8 + 4) * i + 32 + 8
          )
        ),
        stakeCount: validatorListAccount!.data.readUInt32LE(
          3 + (32 + 8 + 8 + 4) * i + 32 + 8 + 8
        ),
      });
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
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async addValidatorInstruction(params: AddValidatorParams) {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(2, 0);

    return new TransactionInstruction({
      keys: [
        {
          pubkey: this.stakePool.publicKey,
          isSigner: false,
          isWritable: false,
        },
        { pubkey: this.owner.publicKey, isSigner: true, isWritable: false },
        {
          pubkey: this.validatorStakeListAccount.publicKey,
          isSigner: false,
          isWritable: true,
        },
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
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async depositInstruction(params: DepositParams) {
    const data = Buffer.alloc(1 + 8);
    let p = data.writeUInt8(6, 0);
    p = data.writeBigInt64LE(BigInt(params.amount), p);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        {
          pubkey: await this.getWithdrawAuthority(),
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: await this.getReserveAddress(),
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: params.userSource.publicKey,
          isSigner: true,
          isWritable: true,
        },
        { pubkey: params.userToken, isSigner: false, isWritable: true },
        { pubkey: this.ownersFee, isSigner: false, isWritable: true },
        { pubkey: this.poolMintToken, isSigner: false, isWritable: true },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async withdraw(params: WithdrawParams) {
    console.log(`Withdraw ${params.amount}`);
    const transaction = new Transaction();
    transaction.add(await this.withdrawInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async withdrawInstruction(params: WithdrawParams) {
    const data = Buffer.alloc(1 + 8);
    let p = data.writeUInt8(7, 0);
    p = data.writeBigInt64LE(BigInt(params.amount), p);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        {
          pubkey: await this.getWithdrawAuthority(),
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: await this.getReserveAddress(),
          isSigner: false,
          isWritable: true,
        },
        { pubkey: params.userTokenSource, isSigner: false, isWritable: true },
        { pubkey: this.poolMintToken, isSigner: false, isWritable: true },
        { pubkey: params.userSolTarget, isSigner: false, isWritable: true },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async credit(params: CreditParams) {
    if (params.amount >= 0) {
      console.log(`Credit ${params.amount}`);
    } else {
      console.log(`Uncredit ${-params.amount}`);
    }
    const transaction = new Transaction();
    transaction.add(await this.creditInstruction(params));
    const signatures = [this.payerAccount];
    if (params.amount < 0) {
      signatures.push(params.cancelAuthority as Account)
    }
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      signatures,
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async creditInstruction(params: CreditParams) {
    const data = Buffer.alloc(1 + 8);
    let p = data.writeUInt8(params.amount >= 0 ? 10 : 11, 0);
    p = data.writeBigInt64LE(BigInt(Math.abs(params.amount)), p);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.creditListAccount.publicKey, isSigner: false, isWritable: true },
        { pubkey: this.creditReserve, isSigner: false, isWritable: true },
        {
          pubkey: await this.getWithdrawAuthority(),
          isSigner: false,
          isWritable: false,
        },
        { pubkey: params.userTokenSource, isSigner: false, isWritable: true },
        { pubkey: params.userSolTarget, isSigner: false, isWritable: false },
        { pubkey: (params.amount >= 0) ? params.cancelAuthority as PublicKey : (params.cancelAuthority as Account).publicKey, isSigner: (params.amount < 0), isWritable: false },
        { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async delegateReserve(params: DepositReserveParams): Promise<void> {
    const transaction = new Transaction();
    transaction.add(await this.delegateReserveInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount, this.owner],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async delegateReserveInstruction(params: DepositReserveParams) {
    const data = Buffer.alloc(1 + 4 + (8 + 4) * params.validators.length);
    let p = data.writeUInt8(12, 0);
    p = data.writeUInt32LE(params.validators.length, p);

    const keys = [
      { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
      { pubkey: this.owner.publicKey, isSigner: true, isWritable: false },
      {
        pubkey: this.validatorStakeListAccount.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: await this.getWithdrawAuthority(),
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: await this.getDepositAuthority(),
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: await this.getReserveAddress(),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      {
        pubkey: SYSVAR_STAKE_HISTORY_PUBKEY,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: new PublicKey('StakeConfig11111111111111111111111111111111'),
        isSigner: false,
        isWritable: false,
      },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ];

    console.log(`Delegate from reserve into`);
    for (const validator of params.validators) {
      p = data.writeBigUInt64LE(BigInt(validator.amount), p);
      p = data.writeUInt32LE(validator.stakeIndex, p);

      keys.push({
        pubkey: validator.address,
        isSigner: false,
        isWritable: false,
      });

      const stake = await this.getStakeForValidator(
        validator.address,
        validator.stakeIndex
      );
      console.log(
        `Validator ${validator.address.toBase58()} with stake #${validator.stakeIndex} ${stake.toBase58()}`
      );
      keys.push({
        pubkey: stake,
        isSigner: false,
        isWritable: true,
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
      throw Error('Too low delegation');
    }

    const validators = await this.readValidators();
    if (validators.length == 0) {
      throw Error("No validator added");
    }
    const stakeTotal = validators
      .map((v) => v.balance)
      .reduce((a, b) => a + b, 0);
    const targetAmount = Math.ceil(
      (stakeTotal + totalAmount) / validators.length
    );

    const instructions: DepositReserveValidatorParam[] = [];
    let amountLeft = totalAmount;
    for (const validator of validators) {
      if (amountLeft < 1) {
        break;
      }

      if (validator.balance >= targetAmount) {
        continue;
      }

      let delegateAmount = Math.min(
        targetAmount - validator.balance,
        amountLeft
      );

      amountLeft -= delegateAmount;

      if (amountLeft < MIN_STAKE_ACCOUNT_BALANCE) {
        delegateAmount += amountLeft;
        amountLeft = 0;
      }

      let currentEpochStakeIndex = -1;
      for (let index = 0; index < validator.stakeCount; index++) {
        const stakeAddress = await this.getStakeForValidator(
          validator.votePubkey,
          index
        );
        try {
          const stakeData = await this.connection.getStakeActivation(
            stakeAddress,
            'singleGossip'
          );
          if (stakeData.state == 'activating' && stakeData.active == 0) {
            currentEpochStakeIndex = index;
            break;
          }
        } catch (e) { }
      }

      if (currentEpochStakeIndex >= 0) {
        console.log(
          `Redelegate to stake #${currentEpochStakeIndex} ${await this.getStakeForValidator(
            validator.votePubkey,
            currentEpochStakeIndex
          )}`
        );
        instructions.push({
          address: validator.votePubkey,
          amount: delegateAmount,
          stakeIndex: currentEpochStakeIndex,
        });
        continue;
      }

      let firstFreeIndex = validator.stakeCount;
      for (let index = 0; index < validator.stakeCount; index++) {
        const stakeAddress = await this.getStakeForValidator(
          validator.votePubkey,
          index
        );
        const stakeAccount = await this.connection.getAccountInfo(stakeAddress);
        if (!stakeAccount || stakeAccount.owner == SystemProgram.programId) {
          firstFreeIndex = index;
        }
      }

      console.log(
        `Init stake #${firstFreeIndex} ${await this.getStakeForValidator(
          validator.votePubkey,
          firstFreeIndex
        )}`
      );
      instructions.push({
        address: validator.votePubkey,
        amount: delegateAmount,
        stakeIndex: firstFreeIndex,
      });
    }

    if (amountLeft > 0) {
      throw Error(`Left ${amountLeft}`);
    }

    await this.delegateReserve({
      validators: instructions,
    });
  }

  async mergeStakes(params: MergeStakesParams): Promise<void> {
    if (params.stakePairs.length == 0) {
      return;
    }
    const transaction = new Transaction();
    transaction.add(await this.mergeStakesInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async mergeStakesInstruction(params: MergeStakesParams): Promise<TransactionInstruction> {
    const data = Buffer.alloc(1 + 4 + (32 + 4 + 4) * params.stakePairs.length);
    let p = data.writeUInt8(13, 0);
    p = data.writeUInt32LE(params.stakePairs.length, p);

    const keys = [
      { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
      {
        pubkey: this.validatorStakeListAccount.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: await this.getDepositAuthority(),
        isSigner: false,
        isWritable: false,
      },
      { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      {
        pubkey: SYSVAR_STAKE_HISTORY_PUBKEY,
        isSigner: false,
        isWritable: false,
      },
    ];

    console.log(`Merging stakes`);
    for (const stakePair of params.stakePairs) {
      p += stakePair.validator.toBuffer().copy(data, p);
      p = data.writeUInt32LE(stakePair.mainIndex, p);
      p = data.writeUInt32LE(stakePair.additionalIndex, p);

      const mainStake = await this.getStakeForValidator(
        stakePair.validator,
        stakePair.mainIndex
      );
      const additionalStake = await this.getStakeForValidator(
        stakePair.validator,
        stakePair.additionalIndex
      );
      console.log(
        `Merge validator ${stakePair.validator.toBase58()} stake #${stakePair.mainIndex} ${mainStake.toBase58()} with #${stakePair.additionalIndex} ${additionalStake.toBase58()}`
      );

      keys.push({
        pubkey: mainStake,
        isSigner: false,
        isWritable: true,
      });

      keys.push({
        pubkey: additionalStake,
        isSigner: false,
        isWritable: true,
      });
    }

    return new TransactionInstruction({
      keys,
      programId: this.programId,
      data,
    });
  }

  async mergeAllStakes(): Promise<void> {
    const validators = await this.readValidators();
    if (validators.length == 0) {
      throw Error("No validator added");
    }

    const stakePairs: StakePair[] = [];
    for (let validator of validators) {
      const mergeIndices: number[] = [];
      for (let i = 0; i < validator.stakeCount; ++i) {
        const stakeAddress = await this.getStakeForValidator(validator.votePubkey, i);
        try {
          const stakeData = await this.connection.getStakeActivation(
            stakeAddress,
            'singleGossip'
          );
          if (stakeData.state == 'active') {
            mergeIndices.push(i)
          }
        } catch (e) { }
      }
      for (let i = mergeIndices.length - 1; i > 0; --i) {
        stakePairs.push({
          validator: validator.votePubkey,
          mainIndex: mergeIndices[0],
          additionalIndex: mergeIndices[i],
        })
      }
    }

    await this.mergeStakes({
      stakePairs
    });
  }

  async unstake(params: UnstakeParams): Promise<void> {
    if (params.unstakes.length == 0) {
      return;
    }
    const transaction = new Transaction();
    transaction.add(await this.unstakeInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount, this.owner],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async unstakeInstruction(params: UnstakeParams): Promise<TransactionInstruction> {
    const data = Buffer.alloc(1 + 4 + (32 + 4 + 4 + 8) * params.unstakes.length);
    let p = data.writeUInt8(14, 0);
    p = data.writeUInt32LE(params.unstakes.length, p);

    const keys = [
      { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
      { pubkey: this.owner.publicKey, isSigner: true, isWritable: false },
      {
        pubkey: this.validatorStakeListAccount.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: await this.getDepositAuthority(),
        isSigner: false,
        isWritable: false,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
    ];

    console.log(`Unstake`);
    for (const unstake of params.unstakes) {
      p += unstake.validator.toBuffer().copy(data, p);
      p = data.writeUInt32LE(unstake.sourceIndex, p);
      p = data.writeUInt32LE(unstake.splitIndex || unstake.sourceIndex, p);
      p = data.writeBigUInt64LE(BigInt(unstake.amount || 0), p);

      const sourceStake = await this.getStakeForValidator(
        unstake.validator,
        unstake.sourceIndex
      );

      keys.push({
        pubkey: sourceStake,
        isSigner: false,
        isWritable: true,
      });

      if (unstake.splitIndex) {
        const splitStake = await this.getStakeForValidator(
          unstake.validator,
          unstake.splitIndex
        );
        console.log(
          `Split validator ${unstake.validator.toBase58()} stake #${unstake.sourceIndex} ${sourceStake.toBase58()} with #${unstake.splitIndex} ${splitStake.toBase58()}`
        );

        keys.push({
          pubkey: splitStake,
          isSigner: false,
          isWritable: true,
        });
      } else {
        console.log(
          `Undelegate validator ${unstake.validator.toBase58()} stake #${unstake.sourceIndex} ${sourceStake.toBase58()}`
        );
      }
    }

    return new TransactionInstruction({
      keys,
      programId: this.programId,
      data,
    });
  }

  async unstakeAll(): Promise<void> {
    const validators = await this.readValidators();
    if (validators.length == 0) {
      throw Error("No validator added");
    }

    const unstakes: Unstake[] = [];
    for (let validator of validators) {
      for (let i = 0; i < validator.stakeCount; ++i) {
        const stakeAddress = await this.getStakeForValidator(validator.votePubkey, i);
        try {
          const stakeData = await this.connection.getStakeActivation(
            stakeAddress,
            'singleGossip'
          );
          if ((stakeData.active > 0) && ((stakeData.state == 'active') || (stakeData.state == 'activating'))) {
            unstakes.push({
              validator: validator.votePubkey,
              sourceIndex: i
            })
          }
        } catch (e) { }
      }
    }

    await this.unstake({
      unstakes
    });
  }

  async payCreditors(maxCount: number) {
    console.log(`Pay creditors`);
    const transaction = new Transaction();
    transaction.add(await this.payCreditorsInstruction(maxCount));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async payCreditorsInstruction(maxCount: number) {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(15, 0);

    const state = await this.readState();

    const keys = [
      { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
      { pubkey: this.creditListAccount.publicKey, isSigner: false, isWritable: true },
      {
        pubkey: await this.getWithdrawAuthority(),
        isSigner: false,
        isWritable: false,
      },
      { pubkey: await this.getReserveAddress(), isSigner: false, isWritable: true },
      { pubkey: state!.creditReserve, isSigner: false, isWritable: true },
      { pubkey: this.poolMintToken, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    const creditors = await this.readCreditors();
    for (let i = 0; ((maxCount <= 0) || (i < maxCount)) && (i < creditors.length); ++i) {
      keys.push(
        { pubkey: creditors[i].target, isSigner: false, isWritable: true },
      )
    }

    return new TransactionInstruction({
      keys,
      programId: this.programId,
      data,
    });
  }

  async readCreditors(): Promise<Creditor[]> {
    const state = await this.readState();
    const creditorsAccount = await this.connection.getAccountInfo(state!.creditList);
    const data = creditorsAccount!.data;
    const count = data.readUInt16LE(1);
    const creditors: Creditor[] = [];

    let p = 3;
    for (let i = 0; i < count; ++i) {
      const target = new PublicKey(data.slice(p, p + 32));
      p += 32;
      const cancelAuthority = new PublicKey(data.slice(p, p + 32));
      p += 32;
      const amount = Number(data.readBigUInt64LE(p));
      p += 8;
      creditors.push({
        target,
        cancelAuthority,
        amount
      })
    }

    return creditors;
  }

  async updateListBalanceReserve(
    params: UpdateListBalanceParams
  ): Promise<void> {
    const transaction = new Transaction();
    transaction.add(await this.updateListBalanceInstruction(params));
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async updateListBalanceInstruction(
    params: UpdateListBalanceParams
  ): Promise<TransactionInstruction> {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(4, 0);

    const keys = [
      { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: false },
      {
        pubkey: this.validatorStakeListAccount.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: await this.getWithdrawAuthority(),
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: await this.getReserveAddress(),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: StakeProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
    ];

    const allValidators = await this.readValidators();
    for (const validator of params.validators) {
      keys.push({
        pubkey: validator,
        isSigner: false,
        isWritable: false,
      });

      const validatorInfo = allValidators.find((v) =>
        v.votePubkey.equals(validator)
      );
      for (let i = 0; i < validatorInfo!.stakeCount; ++i) {
        const stake = await this.getStakeForValidator(validator, i);
        keys.push({
          pubkey: stake,
          isSigner: false,
          isWritable: true,
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
        preflightCommitment: 'singleGossip',
      }
    );
  }

  async updatePoolBalanceInstruction(): Promise<TransactionInstruction> {
    const data = Buffer.alloc(1);
    let p = data.writeUInt8(5, 0);

    return new TransactionInstruction({
      keys: [
        { pubkey: this.stakePool.publicKey, isSigner: false, isWritable: true },
        {
          pubkey: this.validatorStakeListAccount.publicKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: await this.getReserveAddress(),
          isSigner: false,
          isWritable: false,
        },
        { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data,
    });
  }

  async updatePool(): Promise<void> {
    console.log('Updating pool');
    const transaction = new Transaction();
    transaction.add(
      await this.updateListBalanceInstruction({
        validators: (await this.readValidators()).map((v) => v.votePubkey),
      })
    );
    transaction.add(await this.updatePoolBalanceInstruction());
    await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [this.payerAccount],
      {
        commitment: 'singleGossip',
        preflightCommitment: 'singleGossip',
      }
    );
  }
}
