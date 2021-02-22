/**
 * Hello world
 */

import { Account, PublicKey, SystemInstruction, SystemProgram } from '@solana/web3.js';
import { TenderizeProgram } from './tenderize';
import { Tester } from './tester';

async function main() {
  const tester = await Tester.build();
  await tester.initTenderize();

  const withrdawAuthority = new PublicKey("5es37KhF5VKHtSPXNDzwPNMSndizyNFvHzeFwzEKW3vg");
  const depositAuthotiry = new PublicKey("9EVGoPwR9TLrxnuAqhLgk5hBkJY9ogbeUyUu83vqsYki");

  if (true) {
    await tester.createStakePool();
    for (const validator of await tester.getValidators()) {
      await tester.tenderize!.createValidatorStake({
        validator,
        stakePoolDepositAuthority: depositAuthotiry, // TODO calculate automaticaly
        stakePoolWithdrawAuthority: withrdawAuthority
      });
      await tester.tenderize!.addValidator({
        stakePoolDepositAuthority: depositAuthotiry,
        stakePoolWithdrawAuthority: withrdawAuthority, // TODO calculate automaticaly
        validator,
      })
    }
  }
  await tester.tenderize!.testDeposit({
    amount: 100,
    userWallet: tester.payerAccount,
    stakePoolDepositAuthority: depositAuthotiry,
    validators: await tester.getValidators()
  });
  /*
  await tester.tenderize!.deposit({
    stakePoolDepositAuthority: depositAuthotiry, // TODO calculate automaticaly
    stakePoolWithdrawAuthority: withrdawAuthority, // TODO calculate automaticaly
    input: new PublicKey('DqtKB4byyy9r3Vywt2jt3Mj5bE9iq6eMU3QMy8sMt7iB'),
    validatorsStake: validatorsStake,
    outputTokenAccount: new PublicKey("7dRm2RFiC5xGn9KodMtgS1oFXMNdaPhjTYiSeJrUdwVn"),
    feeTokenAccount: new PublicKey("7dRm2RFiC5xGn9KodMtgS1oFXMNdaPhjTYiSeJrUdwVn"),
  });*/
  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
