import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
// import * as BufferLayout from 'buffer-layout';
import {
  TOKEN_PROGRAM_ID,
  TENDERIZE_PROGRAM_ID,
  STAKE_POOL_ID,
  WITHDRAW_AUTHORITY_PDA,
  RESERVE_ADDRESS_PDA,
  OWNER_FEE_ACCOUNT,
  TENDERIZED_SOL_MINT_ID,
  TEMP_ACCOUNT_PDA,
  WRAPPED_SOL_MINT,
} from '../../utils/ids';
// import * as Layout from './../../utils/layout';
// import { LendingInstruction } from './lending';
// import { LendingReserve } from './reserve';

/// Deposit liquidity into a reserve. The output is a collateral token representing ownership
/// of the reserve liquidity pool.
///
///   0. `[writable]` Source liquidity token account. $authority can transfer $liquidity_amount
///   1. `[writable]` Destination collateral token account.
///   2. `[writable]` Reserve account.
///   3. `[writable]` Reserve liquidity supply SPL Token account.
///   4. `[writable]` Reserve collateral SPL Token mint.
///   5. `[]` Lending market account.
///   6. `[]` Derived lending market authority.
///   7. `[]` User transfer authority ($authority).
///   8. `[]` Clock sysvar
///   9. '[]` Token program id

export interface DepositParams {
  userSource: PublicKey;
  amount: number | BN;
  userToken: PublicKey;
}

export const depositInstruction = (
  params: DepositParams
  // liquidityAmount: number | BN,
  // from: PublicKey, // Liquidity input SPL Token account. $authority can transfer $liquidity_amount
  // to: PublicKey // Collateral output SPL Token account,
  /*lendingMarket: PublicKey,
  reserveAuthority: PublicKey,
  transferAuthority: PublicKey,
  reserveAccount: PublicKey,
  reserveSupply: PublicKey,
  collateralMint: PublicKey*/
): TransactionInstruction => {
  // const dataLayout = BufferLayout.struct([
  //   BufferLayout.u8('instruction'),
  //   Layout.uint64('liquidityAmount'),
  // ]);

  // const data = Buffer.alloc(dataLayout.span);
  // dataLayout.encode(
  //   {
  //     instruction: LendingInstruction.DepositReserveLiquidity,
  //     liquidityAmount: new BN(liquidityAmount),
  //   },
  //   data
  // );

  const data = Buffer.alloc(1 + 8);
  // eslint-disable-next-line
  let p = data.writeUInt8(6, 0);
  p = data.writeBigInt64LE(BigInt(params.amount), p);

  const keys = [
    { pubkey: STAKE_POOL_ID, isSigner: false, isWritable: true },
    { pubkey: WITHDRAW_AUTHORITY_PDA, isSigner: false, isWritable: false },
    { pubkey: RESERVE_ADDRESS_PDA, isSigner: false, isWritable: true },
    { pubkey: params.userSource, isSigner: false, isWritable: true },
    { pubkey: params.userToken, isSigner: false, isWritable: true },
    { pubkey: OWNER_FEE_ACCOUNT, isSigner: false, isWritable: true },
    { pubkey: TENDERIZED_SOL_MINT_ID, isSigner: false, isWritable: true },
    { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: TEMP_ACCOUNT_PDA, isSigner: false, isWritable: true },
    { pubkey: WRAPPED_SOL_MINT, isSigner: false, isWritable: true },
  ];
  return new TransactionInstruction({
    keys,
    programId: TENDERIZE_PROGRAM_ID,
    data,
  });
};

// export const calculateDepositAPY = (reserve: LendingReserve) => {
//   const currentUtilization = calculateUtilizationRatio(reserve);
//   return currentUtilization;
// };
