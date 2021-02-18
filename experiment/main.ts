/**
 * Hello world
 */

import { Account } from '@solana/web3.js';
import { Tester } from './tester';

async function main() {
  console.log("Let's say hello to a Solana account...");

  const tester = await Tester.build();

  const program = await tester.getProgramId("solana_bpf_tenderize");
  const state = await tester.makeAccount(program);
  await tester.runProgram(program, state);

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
