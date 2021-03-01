import { TenderizeProgram } from "./tenderize";
import { ArgumentParser } from 'argparse';
import { execShellCommand } from "./util/shell";
import assert from "assert";
import { Account, AccountInfo, Connection, LAMPORTS_PER_SOL, PublicKey, StakeActivationData } from "@solana/web3.js";
import fs from 'mz/fs';
import { argv } from "process";

async function readAccount(fileName: string): Promise<Account> {
  return new Account(JSON.parse(await fs.readFile(fileName, 'utf-8')))
}

async function ensureFunds<T>(connection: Connection, payer: PublicKey, amount: number) {
  const account = await connection.getAccountInfo(payer);
  let balance = account ? account.lamports : 0;
  while (balance < amount) {
    console.log('\n ...Airdrop 10 SOL...');
    await connection.requestAirdrop(payer, 10 * LAMPORTS_PER_SOL);
    balance += 10 * LAMPORTS_PER_SOL;
  }
}

export async function run(): Promise<void> {
  const solana_config = (await execShellCommand('solana config get')).split('\n');
  let defaultUrl = null;
  let defaultPayerAccount = null;
  for (let line of solana_config) {
    line = line.trimRight();
    let m = line.match(/^RPC URL: (.*)$/i);
    if (m) {
      defaultUrl = m[1];
    }

    m = line.match(/^Keypair Path: (.*)$/);
    if (m) {
      defaultPayerAccount = m[1];
    }
  }
  assert.ok(defaultUrl, "Can't parse solana config RPC URL");
  assert.ok(defaultPayerAccount, "Can't parse solana config account");

  const parser = new ArgumentParser({
  });

  parser.add_argument("--url", {
    default: defaultUrl
  });

  parser.add_argument("--payer", {
    default: defaultPayerAccount
  });

  parser.add_argument("--program", {
    default: "../keys/solana_bpf_tenderize-keypair.json"
  });

  parser.add_argument("--stake-pool", {
    default: "../keys/stake_pool.json"
  });

  parser.add_argument("--owner", {
    default: defaultPayerAccount
  });

  parser.add_argument("--validator-list", {
    default: "../keys/validator_list.json"
  });

  parser.add_argument("--credit-list", {
    default: "../keys/credit_list.json"
  });

  parser.add_argument("--tSOL", {
    default: "../keys/tSOL_token.json"
  });

  parser.add_argument("--owners-fee", {
    default: "../keys/owners_fee.json"
  });

  parser.add_argument("--credit-reserve", {
    default: "../keys/credit_reserve.json"
  });

  parser.add_argument("--fee-denominator", {
    type: Number,
    default: 100
  });
  parser.add_argument("--fee-numerator", {
    type: Number,
    default: 3
  });

  parser.add_argument("--min-reserve", {
    type: Number,
    default: 5
  });

  const subparsers = parser.add_subparsers({});

  const showParser = subparsers.add_parser("show", {});
  showParser.set_defaults({
    "command": "show"
  });

  const vaddParser = subparsers.add_parser("vadd", {});
  vaddParser.add_argument("votes", {
    nargs: "*"
  });
  vaddParser.set_defaults({
    "command": "vadd"
  });

  const vremParser = subparsers.add_parser("vrem", {});
  vremParser.add_argument("votes", {
    nargs: "*"
  });
  vremParser.set_defaults({
    "command": "vrem"
  });

  const delegateReserveParser = subparsers.add_parser("del", {});
  delegateReserveParser.add_argument("--amount", {
    type: Number,
  });
  delegateReserveParser.set_defaults({
    "command": "del"
  });

  const updateParser = subparsers.add_parser("update", {});
  updateParser.set_defaults(
    {
      "command": "update"
    }
  )

  const unstakeParser = subparsers.add_parser("unstake", {});
  unstakeParser.add_argument("--vote", {});
  unstakeParser.add_argument("--amount", {
    type: Number
  });
  unstakeParser.set_defaults(
    {
      "command": "unstake"
    }
  );

  const args = parser.parse_args();

  console.dir(args)

  // Prepare env
  const connection = new Connection(args["url"], 'singleGossip');

  const payer = await readAccount(args["payer"]);

  const tenderize = new TenderizeProgram(
    connection,
    payer,
    await readAccount(args["program"]),
    await readAccount(args["stake_pool"]),
    await readAccount(args["owner"]),
    await readAccount(args["validator_list"]),
    await readAccount(args['credit_list']),
    (await readAccount(args['tSOL'])).publicKey,
    (await readAccount(args['owners_fee'])).publicKey,
    (await readAccount(args['credit_reserve'])).publicKey
  );

  let state = await tenderize.readState(/*(s) => {
    state = s;
  }*/);

  let wasUpdated = false;
  if (!state || state.version == 0) {
    await execShellCommand(`spl-token create-token ${args['tSOL']}`);
    await execShellCommand(`spl-token authorize ${tenderize.poolMintToken.toBase58()} mint ${await tenderize.getWithdrawAuthority()}`);
    await execShellCommand(`spl-token create-account ${tenderize.poolMintToken.toBase58()} ${args['owners_fee']}`);
    await execShellCommand(`spl-token create-account ${tenderize.poolMintToken.toBase58()} ${args['credit_reserve']}`);
    console.log('\n ...Create stake pool...');
    await tenderize.createStakePool({
      feeDenominator: args["fee_denominator"],
      feeNumerator: args["fee_numerator"]
    });
    state = await tenderize.readState();
  } else {
    if (state.lastEpochUpdate < (await connection.getEpochInfo()).epoch) {
      console.log('\n ...Update...');

      await tenderize.mergeAllStakes();

      await tenderize.updatePool();
      wasUpdated = true;
      state = await tenderize.readState();
    }
  }

  console.log('\n ...Pay creditors...');

  await tenderize.payAllCreditors();

  let reserve = await connection.getAccountInfo(await tenderize.getReserveAddress());
  const minReserve = Math.max(LAMPORTS_PER_SOL * args["min_reserve"],
    await connection.getMinimumBalanceForRentExemption(0) + await connection.getMinimumBalanceForRentExemption(10000));
  if (!reserve || reserve.lamports < minReserve) {
    const amount = minReserve - (reserve ? reserve.lamports : 0);
    ensureFunds(connection, payer.publicKey, amount);

    console.log('\n ...Calling deposit function...');

    await tenderize.deposit({
      userSource: tenderize.payerAccount,
      amount,
      userToken: tenderize.ownersFee
    });

    reserve = await connection.getAccountInfo(await tenderize.getReserveAddress());
  }

  console.log(`Tenderize ${tenderize.stakePool.publicKey.toBase58()} with reserve ${await tenderize.getReserveAddress()} ${reserve!.lamports / LAMPORTS_PER_SOL}`);
  console.log(`Token ${tenderize.poolMintToken} validator list ${tenderize.validatorStakeListAccount.publicKey}`);
  console.log(`Balance ${Number(state!.stakeTotal) / LAMPORTS_PER_SOL} SOL / ${Number(state!.poolTotal) / LAMPORTS_PER_SOL} tSOL`);
  {
    const validators = await tenderize.readValidators();
    for (const validator of validators) {
      console.log(`Validator: ${validator.votePubkey.toBase58()} balance ${validator.balance / LAMPORTS_PER_SOL}`);
      for (let i = 0; i < validator.stakeCount; ++i) {
        const stake = await tenderize.getStakeForValidator(validator.votePubkey, i);
        let stakeInfo: StakeActivationData | undefined;
        let acc: AccountInfo<Buffer> | undefined | null;
        try {
          acc = await connection.getAccountInfo(stake);
          stakeInfo = await connection.getStakeActivation(stake);
        } catch (e) {

        }
        console.log(`  Stake #${i} ${stake.toBase58()} ${stakeInfo?.state || "free"} balance ${Number(acc?.lamports || 0) / LAMPORTS_PER_SOL} active ${Number(stakeInfo?.active || 0) / LAMPORTS_PER_SOL} inactive ${Number(stakeInfo?.inactive || 0) / LAMPORTS_PER_SOL}`)
      }
    }
  }
  {
    const creditors = await tenderize.readCreditors();
    for (const creditor of creditors) {
      console.log(`Credit for user ${creditor.target} amount ${creditor.amount}`);
    }
  }


  // Process command
  switch (args["command"]) {
    case "vadd": {
      let validators: PublicKey[];
      if (args["votes"].length === 0) {
        let currentValidators = await tenderize.readValidators();
        validators = (await connection.getVoteAccounts('singleGossip'))
          .current.map((v) => new PublicKey(v.votePubkey))
          .filter((v) => !currentValidators.find((c) => c.votePubkey.equals(v)));
      } else {
        validators = args["votes"].map(((v: string) => new PublicKey(v)));
      }

      console.log('\n ...Add validators...');
      for (const validator of validators) {
        await tenderize.addValidator({
          validator,
        });
      }
      break;
    }
    case "vrem": {
      let validators: PublicKey[];
      if (args["votes"].length === 0) {
        validators = (await tenderize.readValidators())
          .filter((v) => v.stakeCount === 0)
          .map((v) => v.votePubkey);
      } else {
        validators = args["votes"].map(((v: string) => new PublicKey(v)));
      }

      console.log('\n ...Remove validators...');
      for (const validator of validators) {
        await tenderize.removeValidator({
          validator,
        });
      }
      break;
    }
    case "del": {
      console.log('\n ...Delegating reserve...');
      let amount: number;
      if (args["amount"]) {
        amount = args["amount"] * LAMPORTS_PER_SOL;
      } else {
        amount = Math.round(0.9 * reserve!.lamports);
      }
      await tenderize.delegateReserveBatch(amount);
      break;
    }
    case "update": {
      if (!wasUpdated) {
        await tenderize.mergeAllStakes();

        await tenderize.updatePool();
        wasUpdated = true;
        state = await tenderize.readState();
      }
      break;
    }
    case "unstake": {
      console.log("unstake");
      if (args["vote"]) {
        const validators = await tenderize.readValidators();
        const v = validators.find((v) => v.votePubkey.toBase58() === args["vote"]);
        if (!v) {
          throw Error(`Invalid validator ${args["vote"]}`);
        }
        await tenderize.unstake(v, args["amount"]);
      } else {
        await tenderize.unstakeAll();
      }

      break;
    }
  }
}