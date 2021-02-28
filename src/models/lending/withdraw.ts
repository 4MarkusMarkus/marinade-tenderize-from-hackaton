import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import * as BufferLayout from 'buffer-layout';
import { Tenderize } from '.';
import { TOKEN_PROGRAM_ID, TENDERIZE_PROGRAM_ID, STAKE_POOL_ID, WITHDRAW_AUTHORITY_PDA, RESERVE_ADDRESS_PDA } from '../../utils/ids';
import * as Layout from './../../utils/layout';
import { LendingInstruction } from './lending';

export interface WithdrawParams {
  userTokenSource: PublicKey;
  userSolTarget: PublicKey;
  amount: number | BN;
}

export const withdrawInstruction = (
  params: WithdrawParams,
  tenderize: Tenderize
): TransactionInstruction => {
  const dataLayout = BufferLayout.struct([
    BufferLayout.u8('instruction'),
    Layout.uint64('collateralAmount'),
  ]);

  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: LendingInstruction.WithdrawReserveLiquidity,
      collateralAmount: new BN(params.amount),
    },
    data
  );

  const keys = [
    { pubkey: STAKE_POOL_ID, isSigner: false, isWritable: true },
    { pubkey: WITHDRAW_AUTHORITY_PDA, isSigner: false, isWritable: false, },
    { pubkey: RESERVE_ADDRESS_PDA, isSigner: false, isWritable: true, },
    { pubkey: params.userTokenSource, isSigner: false, isWritable: true },
    { pubkey: tenderize.poolMint, isSigner: false, isWritable: true },
    { pubkey: params.userSolTarget, isSigner: false, isWritable: true },
    { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
  ];
  return new TransactionInstruction({
    keys,
    programId: TENDERIZE_PROGRAM_ID,
    data,
  });
};
