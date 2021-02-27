import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import * as BufferLayout from 'buffer-layout';
// import { TOKEN_PROGRAM_ID, TENDERIZE_PROGRAM_ID } from '../../utils/ids';
// import { wadToLamports } from '../../utils/utils';
import * as Layout from './../../utils/layout';
// import { LendingInstruction } from './lending';

export const LendingReserveLayout: typeof BufferLayout.Structure = BufferLayout.struct(
  [
    BufferLayout.u8('version'),
    Layout.publicKey('owner'),
    BufferLayout.blob(2, 'padding'),

    Layout.publicKey('validatorStakeList'),
    Layout.publicKey('creditList'),
    Layout.publicKey('poolMint'),
    Layout.publicKey('ownerFeeAccount'),
    Layout.publicKey('creditReserve'),
    Layout.publicKey('tokenProgram'),
    BufferLayout.blob(5, 'padding'),

    Layout.uint64('stakeTotal'),
    Layout.uint64('poolTotal'),
    Layout.uint64('lastEpochUpdate'),
    Layout.uint64('feeDenominator'),
    Layout.uint64('feeNumerator'),
  ]
);

export const isLendingReserve = (info: AccountInfo<Buffer>) => {
  return info.data.length === LendingReserveLayout.span;
};

export interface LendingReserve {
  version: number;
  owner: PublicKey;
  validatorStakeList: PublicKey;
  creditList: PublicKey;
  poolMint: PublicKey;
  ownerFeeAccount: PublicKey;
  creditReserve: PublicKey;
  tokenProgram: PublicKey;
  stakeTotal: BN;
  poolTotal: BN;
  lastEpochUpdate: BN;
  feeDenominator: BN;
  feeNumerator: BN;
}

export const LendingReserveParser = (
  pubKey: PublicKey,
  info: AccountInfo<Buffer>
) => {
  const buffer = Buffer.from(info.data);
  const data = LendingReserveLayout.decode(buffer) as LendingReserve;

  // if (data.lastUpdateSlot.toNumber() === 0) {
  //   return;
  // }

  const details = {
    pubkey: pubKey,
    account: {
      ...info,
    },
    info: data,
  };

  return details;
};

// export const initReserveInstruction = (
//   liquidityAmount: number | BN,
//   maxUtilizationRate: number,

//   from: PublicKey, // Liquidity input SPL Token account. $authority can transfer $liquidity_amount
//   to: PublicKey, // Collateral output SPL Token account,

//   reserveAccount: PublicKey,
//   liquidityMint: PublicKey,
//   liquiditySupply: PublicKey,
//   collateralMint: PublicKey,
//   collateralSupply: PublicKey,
//   lendingMarket: PublicKey,
//   lendingMarketAuthority: PublicKey,
//   transferAuthority: PublicKey,

//   dexMarket: PublicKey // TODO: optional
// ): TransactionInstruction => {
//   const dataLayout = BufferLayout.struct([
//     BufferLayout.u8('instruction'),
//     Layout.uint64('liquidityAmount'),
//     BufferLayout.u8('maxUtilizationRate'),
//   ]);

//   const data = Buffer.alloc(dataLayout.span);
//   dataLayout.encode(
//     {
//       instruction: LendingInstruction.InitReserve, // Init reserve instruction
//       liquidityAmount: new BN(liquidityAmount),
//       maxUtilizationRate: maxUtilizationRate,
//     },
//     data
//   );

//   const keys = [
//     { pubkey: from, isSigner: false, isWritable: true },
//     { pubkey: to, isSigner: false, isWritable: true },
//     { pubkey: reserveAccount, isSigner: false, isWritable: true },
//     { pubkey: liquidityMint, isSigner: false, isWritable: false },
//     { pubkey: liquiditySupply, isSigner: false, isWritable: true },
//     { pubkey: collateralMint, isSigner: false, isWritable: true },
//     { pubkey: collateralSupply, isSigner: false, isWritable: true },

//     // NOTE: Why lending market needs to be a signer?
//     { pubkey: lendingMarket, isSigner: true, isWritable: true },
//     { pubkey: lendingMarketAuthority, isSigner: false, isWritable: false },
//     { pubkey: transferAuthority, isSigner: true, isWritable: false },
//     { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
//     { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
//     { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },

//     // optionals
//     { pubkey: dexMarket, isSigner: false, isWritable: false },
//   ];
//   return new TransactionInstruction({
//     keys,
//     programId: TENDERIZE_PROGRAM_ID,
//     data,
//   });
// };

// export const accrueInterestInstruction = (
//   ...reserveAccount: PublicKey[]
// ): TransactionInstruction => {
//   const dataLayout = BufferLayout.struct([BufferLayout.u8('instruction')]);

//   const data = Buffer.alloc(dataLayout.span);
//   dataLayout.encode(
//     {
//       instruction: LendingInstruction.AccrueReserveInterest,
//     },
//     data
//   );

//   const keys = [
//     { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
//     ...reserveAccount.map((reserve) => ({
//       pubkey: reserve,
//       isSigner: false,
//       isWritable: true,
//     })),
//   ];
//   return new TransactionInstruction({
//     keys,
//     programId: TENDERIZE_PROGRAM_ID,
//     data,
//   });
// };

// export const calculateUtilizationRatio = (reserve: LendingReserve) => {
//   const totalBorrows = wadToLamports(
//     reserve.state.borrowedLiquidityWad
//   ).toNumber();
//   const currentUtilization =
//     totalBorrows / (reserve.state.availableLiquidity.toNumber() + totalBorrows);

//   return currentUtilization;
// };

// export const reserveMarketCap = (reserve?: LendingReserve) => {
//   const available = reserve?.state.availableLiquidity.toNumber() || 0;
//   const borrowed = wadToLamports(
//     reserve?.state.borrowedLiquidityWad
//   ).toNumber();
//   const total = available + borrowed;

//   return total;
// };

// // export const collateralExchangeRate = (reserve?: LendingReserve) => {
// //   return (
// //     (reserve?.state.collateralMintSupply.toNumber() || 1) /
// //     reserveMarketCap(reserve)
// //   );
// // };

// export const collateralToLiquidity = (
//   collateralAmount: BN | number,
//   reserve?: LendingReserve
// ) => {
//   const amount =
//     typeof collateralAmount === 'number'
//       ? collateralAmount
//       : collateralAmount.toNumber();
//   return Math.floor(amount / collateralExchangeRate(reserve));
// };

// export const liquidityToCollateral = (
//   liquidityAmount: BN | number,
//   reserve?: LendingReserve
// ) => {
//   const amount =
//     typeof liquidityAmount === 'number'
//       ? liquidityAmount
//       : liquidityAmount.toNumber();
//   return Math.floor(amount * collateralExchangeRate(reserve));
// };
