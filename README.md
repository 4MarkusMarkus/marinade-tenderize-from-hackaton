# 🥩🔨 Don’t just stake me, Tenderize.me first!

Tenderize.me is a non-custodial liquid staking solution, focused on yield maximisation and ease of use while keeping staking decentralized! 

## Overview
### Bebefits
🔨🥩    Making staking easy and tender
👨‍🌾      Maximizing yield, automatic reinvesting
🌊      Liquifying staking by minting tenderSol token

We provide users with tenderSol, a value accruing staking derivative token, which allows them to enter or exit staking with no bonding and unbonding period!
Our main focus is to make staking more efficient and easy for regular non-technical Solana community members.
Having a tenderSol token that represents their staking position allows users to use it again, for example as a collateral in other protocols.

### Features
**Non-custodial solution**
Tenderize.me team does not hold any users’ funds. Therefore there is no counterparty risk, users do not need to trust us as the third party. We also split users’ stakes to multiple validators to minimize the risks. 

**tSOL - Value accruing derivative token**
When staking, people receive a token representing their staking position. This token is fungible, tradable and automatically accrues value from staking rewards.

**No unstaking period**
Combination of smart reserve with derivative token allows users to start or stop staking by simply trading their derivative token for regular Solana token without unbonding period.

## Vision
Our vision is to make staking stupid simple, thus leveling the playing field for the everyday user and give them the benefits of blockchain OGs 😎.
We aim to serve users first, hence being chain neutral, offering coins based on what people desire. 
We believe in the decentralized Web3 vision, therefore our main principles is keep our solution trust minimized and validator agnostic.

## Video
OUR submission video will go here

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
npm install
npm run prepare:env
npm start
```

Tada! you can Tenderize your SOLs now  🥩🔨!