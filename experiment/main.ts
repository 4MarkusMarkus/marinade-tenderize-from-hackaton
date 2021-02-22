/**
 * Hello world
 */

import { Account, SystemInstruction, SystemProgram } from '@solana/web3.js';
import { TenderizeProgram } from './tenderize';
import { Tester } from './tester';

async function main() {
  const tester = await Tester.build();
  await tester.createStakePool(await Tester.loadAccount("stake_pool"));
  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
