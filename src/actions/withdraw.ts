import {
  Account,
  Connection,
  TransactionInstruction,
} from '@solana/web3.js';
import { AccountLayout } from '@solana/spl-token';
import { sendTransaction } from '../contexts/connection';
import { Tenderize, withdrawInstruction } from '../models/lending';
import { RESERVE_ADDRESS_PDA, WITHDRAW_AUTHORITY_PDA } from '../utils/ids';
import { notify } from '../utils/notifications';
import { approve, TokenAccount } from '../models';
import { WalletAdapter } from '../contexts/wallet';

export const withdraw = async (
  from: TokenAccount, // CollateralAccount
  amountLamports: number, // in collateral token (lamports)
  connection: Connection,
  wallet: WalletAdapter,
  tenderize: Tenderize
) => {
  if (!wallet.publicKey) {
    throw new Error('Wallet is not connected');
  }

  const reserveInfo = await connection.getAccountInfo(RESERVE_ADDRESS_PDA);
  const minReserveBalance = await connection.getMinimumBalanceForRentExemption(0) + await connection.getMinimumBalanceForRentExemption(AccountLayout.span);
  if (!reserveInfo || reserveInfo.lamports < minReserveBalance + amountLamports) {
    throw Error("Not enough funds");
  }

  notify({
    message: 'Withdrawing funds...',
    description: 'Please review transactions to approve.',
    type: 'warn',
  });

  // user from account
  const signers: Account[] = [];
  const instructions: TransactionInstruction[] = [];
  const cleanupInstructions: TransactionInstruction[] = [];

  const fromAccount = from.pubkey;

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

  // get destination account
  const toAccount = wallet.publicKey;

  // instructions.push(accrueInterestInstruction(reserveAddress));

  instructions.push(
    withdrawInstruction(
      {
        userTokenSource: fromAccount,
        userSolTarget: toAccount,
        amount: amountLamports
      },
      tenderize
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
      message: 'Funds withdrawn.',
      type: 'success',
      description: `Transaction - ${tx}`,
    });
  } catch (e) {
    console.log(e);
    // TODO:
  }
};
