### Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana v1.5.0 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool

### Create accounts

1. `solana-keygen new --outfile $HOME/account.json`
2. `solana config set --keypair $HOME/account.json`

### Run application on localnet

1. Initialize local dev environment with `npm localnet:up`
2. Change the network for Solana CLI to the localnet by `solana config set --url http://127.0.0.1:8899`
3. Drop some SOLs to the testing account `solana airdrop 100 $(solana address)`
4. Build & deploy the program `npm run build:program`

### Run tests

1. `cd experiments`
2. Create sample staking and validator accounts `npm run prepare:env`
3. (Optional) Install `ts-node` if not already installed `npm i -g ts-node`
4. `npm-run-start`

