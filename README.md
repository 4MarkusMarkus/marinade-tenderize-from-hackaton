## Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana v1.5.0 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool
3. Install NodeJS and npm

### Create accounts

1. `solana-keygen new --outfile $HOME/account.json`
2. Set a default account for the network `solana config set --keypair $HOME/account.json`
3. Change the network for Solana CLI to the localnet by `solana config set --url http://127.0.0.1:8899`

### Run application on localnet

1. Initialize local cluster with `npm run localnet:up`
3. Drop some SOLs to the testing account `solana airdrop 100`
3. Build & deploy the program `npm run build:program`

### Run tests

1. `cd experiments`
2. `npm i`
3. Create sample staking and validator accounts `npm run prepare:env`
4. `npm run start` - if you see that `ts-node` is missing, install it with `npm i -g ts-node`

You can reset the environment by removing the container with `npm run localnet:down` and then repeat the whole process again.
