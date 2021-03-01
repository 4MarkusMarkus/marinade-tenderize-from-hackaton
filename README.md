# 🥩🔨 Don’t just stake me, Tenderize.me first!

Tenderize.me is a non-custodial liquid staking solution, focused on yield maximisation and ease of use while keeping staking decentralized! 

Start Tenderizing now on testnet at https://solana.tenderize.me/ !

## Overview
### Benefits
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

## How it Works

![image](https://github.com/Tenderize/Tenderize-on-solana/blob/main/docs/Tenderize.me-schemaF.png)

### Problems we are solving

**Unstaking period**

We want the token to be truly liquid. We use balancer pool to do this. We keep a liquid portion of the funds in balancer to keep buffer for withdrawals. Thus people do not need to wait for unstaking.

**Liquidifying staking position**

If people stake just by providing funds to the staker contract, their funds are illiquid. By providing people with tenderToken, we allow them to sell / buy their position freely on secondary market or use it as collateral.

**Higher staking rewards**

Staking rewards are received for longer than if LPs just used staking pools, in which case there are no staking rewards during worming up + cooling down period.

**Risk mitigation**

We stake across multiple staking pools hence being less prone to slashing and distributing the risk

**Gas Cost savings**

Some of the staking solutions are quite expensive gaswise. Therefore it does not make sense for people to stake small amounts of funds.

**Automatic reinvesting of staking rewards**

All staking rewards are automatically reinvested, thus we save gas cost and time of our fellow crypto comrades.

---

## Vision
Our vision is to make staking stupid simple, thus leveling the playing field for the everyday user and give them the benefits of blockchain OGs 😎.
We aim to serve users first, hence being chain neutral, offering coins based on what people desire. 
We believe in the decentralized Web3 vision, therefore our main principles is keep our solution trust minimized and validator agnostic.

## Want to know more?

You can learn more about how the system works in our DOCs folder.
There you can find:

- Quick run of what we are trying to achieve in our [**presentation**](https://docs.google.com/presentation/d/1bxq5OFFLnhV04XF_nPiQVW-V7kMjSFducbtnAa89CZM/edit?usp=sharing)
- System overview in this [**document**](https://docs.google.com/document/d/1U-Hq9P6M7Epuh3WJ-dXEb6PXd-GBIGlgEodyjIEStVc/edit?usp=sharing)
- Overview of our solution in this [**schema**](https://github.com/Tenderize/Tenderize-on-solana/blob/main/docs/Tenderize.me-schemaF.png)



## Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana v1.5.0 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool
3. Install NodeJS and npm

## quickstart

### testnet deployment

You can Tenderize your SOLs on solana testnet already.

Just go to https://solana.tenderize.me/ change your network to testnet and you are ready to Tenderize!!! 

You can check out a deployed AMM pool with tSOLs/SOLs tokens as well. "Swap" button in upper right corner will take you to the pool (deployed on testnet too).

### local deployment

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
solana config set --url http://localhost:8899/ 
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
./tadm.ts
```
### testing

> adding all validators in the network

```bash
./tadm.ts vadd

```

> distributes reserve to validators, needs to be called at the end of epoch

```bash
./tadm.ts del

```

> update creditors, validator balances, restakes rewards. This script needs to be run at least once at the end of each epoch. 

```bash
./tadm.ts

```

Tada! you can Tenderize your SOLs now  🥩🔨!