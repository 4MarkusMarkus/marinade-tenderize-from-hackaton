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

    state = await tester.tenderize!.readState();
  }

  const reserve = await tester.connection.getAccountInfo(await tester.tenderize!.getReserveAddress());
  if (!reserve || reserve.lamports < 100000000000) {
    console.log('\n ...Calling deposit function...');

    await tester.tenderize!.deposit({
      userSource: tester.payerAccount,
      amount: 100000000000,
      userToken: tester.userTokenAccount.publicKey,
    });

    state = await tester.tenderize!.readState();
  }

  if ((await tester.tenderize!.readValidators()).length == 0) {
    console.log('\n ...Updating pool...');

    await tester.tenderize!.updatePool();
    state = await tester.tenderize!.readState();

    console.log('\n ...Add validators...');
    for (const validator of validators) {
      await tester.tenderize!.addValidator({
        validator,
      });
    }
  }

  if (false) {
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

    state = await tester.tenderize!.readState();
  }

  if (true) {
    console.log('\n ...Approving token...');
    await execShellCommand(
      `spl-token approve ${tester.userTokenAccount.publicKey} 1 ${await (
        await tester.tenderize!.getWithdrawAuthority()
      ).toBase58()}`
    );

    const cancelAuthority = new Account();

    console.log('\n ...Calling credit function...');
    await tester.tenderize!.credit({
      userTokenSource: tester.userTokenAccount.publicKey,
      amount: 10000000,
      userSolTarget: tester.payerAccount.publicKey,
      cancelAuthority: cancelAuthority.publicKey
    });

    console.log('\n ...Calling uncredit function...');
    await tester.tenderize!.credit({
      userTokenSource: tester.userTokenAccount.publicKey,
      amount: -2000000,
      userSolTarget: tester.payerAccount.publicKey,
      cancelAuthority
    });

    state = await tester.tenderize!.readState();
  }

  if (true) {
    console.log('\n ...Updating pool...');

    await tester.tenderize!.updatePool();
    state = await tester.tenderize!.readState();

    console.log('\n ...Pay creditors...');

    await tester.tenderize!.payCreditors(0);
  }

  if (true) {
    console.log('\n ...Delegating reserve...');

    await tester.tenderize!.delegateReserveBatch(10000000000);

    state = await tester.tenderize!.readState();
  }

  if (true) {
    console.log('\n ...Merge stakes...');

    await tester.tenderize!.mergeAllStakes();
  }

  if (true) {
    console.log('\n ...Updating pool...');

    await tester.tenderize!.updatePool();
    state = await tester.tenderize!.readState();
  }

  if (true) {
    console.log('\n ...Unstake all...');

    await tester.tenderize!.unstakeAll();
    state = await tester.tenderize!.readState();
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
