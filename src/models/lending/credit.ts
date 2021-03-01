import {
  PublicKey,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import * as BufferLayout from 'buffer-layout';
import { Tenderize } from '.';
import { TOKEN_PROGRAM_ID, TENDERIZE_PROGRAM_ID, STAKE_POOL_ID, WITHDRAW_AUTHORITY_PDA } from '../../utils/ids';
import * as Layout from './../../utils/layout';

export interface CreditParams {
  userTokenSource: PublicKey;
  userSolTarget: PublicKey;
  cancelAuthority: PublicKey;
  amount: number | BN;
}

export const creditInstruction = (
  params: CreditParams,
  tenderize: Tenderize
): TransactionInstruction => {
  const dataLayout = BufferLayout.struct([
    BufferLayout.u8('instruction'),
    Layout.uint64('collateralAmount'),
  ]);

  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: 10,
      collateralAmount: new BN(params.amount),
    },
    data
  );

  const keys = [
    { pubkey: STAKE_POOL_ID, isSigner: false, isWritable: true },
    { pubkey: tenderize.creditList, isSigner: false, isWritable: true },
    { pubkey: tenderize.creditReserve, isSigner: false, isWritable: true },
    { pubkey: WITHDRAW_AUTHORITY_PDA, isSigner: false, isWritable: false, },
    { pubkey: params.userTokenSource, isSigner: false, isWritable: true },
    { pubkey: params.userSolTarget, isSigner: false, isWritable: true },
    { pubkey: params.cancelAuthority, isSigner: false, isWritable: false },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
  ];
  return new TransactionInstruction({
    keys,
    programId: TENDERIZE_PROGRAM_ID,
    data,
  });
};
