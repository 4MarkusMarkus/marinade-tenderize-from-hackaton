import {
  Account,
  Connection,
  // PublicKey,
  TransactionInstruction,
} from '@solana/web3.js';
import { sendTransaction } from '../contexts/connection';
import { notify } from '../utils/notifications';
import { depositInstruction } from './../models/lending';
import { AccountLayout } from '@solana/spl-token';
import { ensureSplAccount, findOrCreateAccountByMint } from './account';
import { approve, TokenAccount } from '../models';
import { WalletAdapter } from '../contexts/wallet';
import { TENDERIZED_SOL_MINT_ID, WITHDRAW_AUTHORITY_PDA } from '../utils/ids';

export const deposit = async (
  from: TokenAccount,
  amountLamports: number,
  connection: Connection,
  wallet: WalletAdapter
) => {
  if (!wallet.publicKey) {
    throw new Error('Wallet is not connected');
  }

  notify({
    message: 'Depositing funds...',
    description: 'Please review transactions to approve.',
    type: 'warn',
  });

  // user from account
  const signers: Account[] = [];
  const instructions: TransactionInstruction[] = [];
  const cleanupInstructions: TransactionInstruction[] = [];

  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span
  );

  // const [authority] = await PublicKey.findProgramAddress(
  //   [reserve.lendingMarket.toBuffer()], // which account should be authority
  //   TENDERIZE_PROGRAM_ID
  // );

  const fromAccount = ensureSplAccount(
    instructions,
    cleanupInstructions,
    from,
    wallet.publicKey,
    amountLamports + accountRentExempt,
    signers
  );

  // create approval for transfer transactions
  approve(
    instructions,
    cleanupInstructions,
    fromAccount,
    wallet.publicKey,
    amountLamports,
    true,
    WITHDRAW_AUTHORITY_PDA
  );

  let toAccount = await findOrCreateAccountByMint(
    wallet.publicKey,
    wallet.publicKey,
    instructions,
    cleanupInstructions,
    accountRentExempt,
    TENDERIZED_SOL_MINT_ID,
    signers
  );

  // instructions.push(accrueInterestInstruction(reserveAddress));

  // deposit
  instructions.push(
    depositInstruction(
      {
        amount: amountLamports,
        userSource: fromAccount,
        userToken: toAccount,
      }
      /*reserve.lendingMarket,
      authority,
      transferAuthority.publicKey,
      reserveAddress,
      reserve.liquiditySupply,
      reserve.collateralMint*/
    )
  );

  try {
    let tx = await sendTransaction(
      connection,
      wallet,
      instructions.concat(cleanupInstructions),
      signers,
      true
    );

    notify({
      message: 'Funds deposited.',
      type: 'success',
      description: `Transaction - ${tx}`,
    });
  } catch (e) {
    // TODO:
    throw e;
  }
};
