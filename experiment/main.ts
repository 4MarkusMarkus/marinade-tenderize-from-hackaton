import { Account, PublicKey, SystemInstruction, SystemProgram } from '@solana/web3.js';
import { TenderizeProgram } from './tenderize';
import { Tester } from './tester';

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
      userToken: tester.tenderize!.ownersFee, // All to my token
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
