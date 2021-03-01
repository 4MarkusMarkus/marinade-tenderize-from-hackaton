# 🥩🔨 Don’t just stake me, Tenderize.me first!

> Tenderize.me is a non-custodial liquid staking solution, focused on yield maximisation and ease of use while keeping staking decentralized! 

---

## Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana v1.5.0 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool
3. Install NodeJS and npm

## quickstart

```bash
git clone https://github.com/Tenderize/Tenderize-on-solana.git

cd Tenderize-on-solana
```

> prepare project + start frontend

```bash
npm install
npm start

```

>  open a second terminal window and run local validator(chain)

```bash
npm run test-validator

```

>  open a third terminal window and build program

```bash
npm run build:program

```
> after progam has been built, initialize deployment

```bash
cd experiment
npm run prepare:env
npm start
```

Tada! you can Tenderize your SOLs now  🥩🔨!