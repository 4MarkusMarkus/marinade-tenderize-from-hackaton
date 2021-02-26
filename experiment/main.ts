import {
  Account,
  PublicKey,
  sendAndConfirmTransaction,
  SystemInstruction,
  SystemProgram,
  Transaction,
} from '@solana/web3.js';
import { TenderizeProgram } from './tenderize';
import { Tester } from './tester';
import { execShellCommand } from './util/shell';

async function main() {
  console.log('\n ...Start by building Tester...');
  const tester = await Tester.build();

  console.log('\n ...Initiliaze Tenderize...');
  await tester.initTenderize();

  const validators = await tester.getValidators();

  let state = await tester.tenderize!.readState();

  if (!state || state.version == 0) {
    console.log('\n ...Create stake pool...');
    await tester.createStakePool();

    console.log('\n ...Calling deposit function...');

    await tester.tenderize!.deposit({
      userSource: tester.payerAccount,
      amount: 100000000000,
      userToken: tester.userTokenAccount.publicKey,
    });

    state = await tester.tenderize!.readState();
  }

  if ((await tester.tenderize!.readValidators()).length == 0) {
    console.log('\n ...Add validators...');
    for (const validator of validators) {
      await tester.tenderize!.addValidator({
        validator,
      });
    }
  }

  if (true) {
    console.log('\n ...Approving token...');
    await execShellCommand(
      `spl-token approve ${tester.userTokenAccount.publicKey} 1 ${await (
        await tester.tenderize!.getWithdrawAuthority()
      ).toBase58()}`
    );

    console.log('\n ...Calling withdraw function...');
    await tester.tenderize!.withdraw({
      userTokenSource: tester.userTokenAccount.publicKey,
      amount: 1000000000,
      userSolTarget: tester.payerAccount.publicKey,
    });
  }
  if (true) {
    console.log('\n ...Delegating reserve...');

    await tester.tenderize!.delegateReserveBatch(10000000000);
  }

  if (true) {
    console.log('\n ...Merge stakes...');

    await tester.tenderize!.mergeAllStakes();
  }

  if (true) {
    console.log('\n ...Updating pool...');

    await tester.tenderize!.updatePool();
  }

  console.log('Success');
}

main().then(
  () => process.exit(),
  (err) => {
    console.error(err);
    process.exit(-1);
  }
);
