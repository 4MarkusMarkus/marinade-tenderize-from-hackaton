import { Account, Connection, TransactionInstruction } from "@solana/web3.js";
import { sendTransaction } from "../contexts/connection";
import { WalletAdapter } from "../contexts/wallet";
import { approve, TokenAccount } from "../models";
import { Tenderize } from "../models/lending";
import { creditInstruction } from "../models/lending/credit";
import { WITHDRAW_AUTHORITY_PDA } from "../utils/ids";
import { notify } from "../utils/notifications";


export const credit = async (
  from: TokenAccount, // CollateralAccount
  amountLamports: number, // in collateral token (lamports)
  connection: Connection,
  wallet: WalletAdapter,
  tenderize: Tenderize
) => {
  if (!wallet.publicKey) {
    throw new Error('Wallet is not connected');
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

  const cancelAuthority = new Account(); // TODO: save it for the future

  instructions.push(
    creditInstruction(
      {
        userTokenSource: fromAccount,
        userSolTarget: toAccount,
        amount: amountLamports,
        cancelAuthority: cancelAuthority.publicKey
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
      message: 'Requested withdraw in the future',
      type: 'success',
      description: `Transaction - ${tx}`,
    });
  } catch (e) {
    console.log(e);
    // TODO:
  }
}