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

## How it Works

![image](https://github.com/Tenderize/Tenderize-on-solana/blob/main/docs/Tenderize.me%20-%20Solana%20program%20schemaF.png)

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

### System Interactions

**Deposit**

Alice deposits solana token into the system and gets back tenderSolana

1. Solana Token is transferred to Manager program

2. Based on current shareprice of tenderSolana the appropriate amount of tenderSols are minted and transferred to Alice

3. The original token is kept as a reserve in Manager program main account. At the end of the epoch function delageteStake() is called, this function does following:


    1. Pays out LPs who wanted to withdraw their stake in previous epoch, but weren’t able to (owed[])


    2. The rest is staked, in our case tenderized 😉. Manager sends rest minus 10% (reserve to pay out LPs) to staking pools.

Things to observe:

No worming up + cooling down period. 

 \
Staking rewards are received for longer than if LPs just used staking pools, since there are no staking rewards during worming up + cooling down period.


**Withdrawal**

Bob withdraws his SOL token by providing tSOL



*   IF enough funds in reserve 
    *   IF YES Withdraw()
        *   1. tSOLs are transferred to Reserve and burned
        *   2. SOLs is send back to Bob
    *   IF NOT 
        *   We will notify user that there are not enough funds in the reserve, in such a case user can decide to come later or to “Get in line to Unstake”, in such case fn getInLin() is called which does following:
            *   1. tSOLs are transferred to Reserve
            *   2. Bob receives SOLs at the end of epoch, if there is enough funds in the reserve at this time
            *   3. If there is not, he would wait until next epoch

**Things to observe:**

Bob receives his initial deposit + accrued staking rewards since tenderSolana token has accrued in value in the time between his deposit and withdrawal.

If he has to wait for his deposit, he is receiving staking rewards for all this time. 

If it happens that there is a time where change in deposits in negative for couple of continues epoch, it may happen that LPs would have to wait for a long time to get their Solana tokens back. In such a case Manager will unstake necessary amount to pay out LPs 



## Vision
Our vision is to make staking stupid simple, thus leveling the playing field for the everyday user and give them the benefits of blockchain OGs 😎.
We aim to serve users first, hence being chain neutral, offering coins based on what people desire. 
We believe in the decentralized Web3 vision, therefore our main principles is keep our solution trust minimized and validator agnostic.

## Want to know more?

You can learn more about how the system works in our DOCs folder.
There you can find:

- Quick run of what we are trying to achieve in our [**presentation**](https://docs.google.com/presentation/d/1bxq5OFFLnhV04XF_nPiQVW-V7kMjSFducbtnAa89CZM/edit?usp=sharing)
- System overview in this [**document**](https://docs.google.com/document/d/1U-Hq9P6M7Epuh3WJ-dXEb6PXd-GBIGlgEodyjIEStVc/edit?usp=sharing)
- Overview of our solution in this [**schema**](https://github.com/Tenderize/Tenderize-on-solana/blob/main/docs/Tenderize.me%20-%20Solana%20program%20schemaF.png)



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