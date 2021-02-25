import { Account, PublicKey, sendAndConfirmTransaction, SystemInstruction, SystemProgram, Transaction } from '@solana/web3.js';
import { TenderizeProgram } from './tenderize';
import { Tester } from './tester';
import { execShellCommand } from './util/shell';

async function main() {
  const tester = await Tester.build();
  await tester.initTenderize();

  const validators = await tester.getValidators();
  if (true) {
    await tester.createStakePool();
    for (const validator of validators) {
      await tester.tenderize!.addValidator({
        validator,
      })
    }
  }
  if (true) {
    await tester.tenderize!.deposit({
      userSource: tester.payerAccount,
      amount: 100000000000,
      userToken: tester.userTokenAccount.publicKey,
    })
  }

  if (true) {
    await execShellCommand(`spl-token approve ${tester.userTokenAccount.publicKey} 1 ${await (await tester.tenderize!.getWithdrawAuthority()).toBase58()}`)

    await tester.tenderize!.withdraw({
      userTokenSource: tester.userTokenAccount.publicKey,
      amount: 1000000000,
      userSolTarget: tester.payerAccount.publicKey,
    })
  }
  if (true) {
    await tester.tenderize!.delegateReserveBatch(10000000000);
  }
  if (true) {
    await tester.tenderize!.updatePool();
  }

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
