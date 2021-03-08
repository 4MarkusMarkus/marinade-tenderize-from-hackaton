import React from 'react';
// import { useLendingReserve } from '../../hooks';
// import { useParams } from 'react-router-dom';
// import { Link } from 'react-router-dom';

// import { DepositInput } from '../../components/DepositInput';
import { Card, Tabs } from 'antd';
import { Line } from 'react-chartjs-2';
// import { sendTransaction, useConnection } from '../../contexts/connection';
// import { useNativeAccount } from '../../contexts/accounts';
// // import { STAKE_POOL_ID } from '../../utils/ids';
// import {
//   PublicKey,
//   sendAndConfirmTransaction,
//   Transaction,
// } from '@solana/web3.js';
// import { depositInstruction, DepositParams } from '../../models/lending';
// import { notify } from '../../utils/notifications';
// import {
//   useWallet,
//   WalletAdapter,
//   WalletProvider,
// } from '../../contexts/wallet';
import { DepositInput } from '../../components/DepositInput';
import { WithdrawInput } from '../../components/WithdrawInput';
// import { LendingReserve } from '../../models/lending';

const solanaLogo = require('../../img/solanaLogo.svg');
const tenderSol = require('../../img/tenderSol.svg');
const { TabPane } = Tabs;

export const DiscoverView = () => {
  // const connection = useConnection();

  // const { wallet } = useWallet();
  const dateData = [
    `Jan-21`,
    `Feb-21`,
    `Mar-21`,
    `Apr-21`,
    `May-21`,
    `Jun-21`,
    `Jul-21`,
    `Aug-21`,
    `Sep-21`,
    `Oct -21`,
    `Nov-21`,
    `Dec-21`,
    `Jan-22`,
    `Feb-22`,
    `Mar-22`,
    `Apr-22`,
    `May-22`,
    `Jun-22`,
    `Jul-22`,
    `Aug-22`,
    `Sep-22`,
    `Oct -22`,
    `Nov-22`,
    `Dec-22`,
  ];
  const data1 = [
    1342,
    1344.05,
    1309.84,
    1428.93,
    1399.34,
    1667.88,
    1825.42,
    2148.62,
    2231.59,
    2319.66,
    2238.63,
    2511.29,
    2929.99,
    2702.93,
    3237.3,
    3412.23,
    3160.59,
    3280,
    3488.19,
    3391.12,
    3406.38,
    4021.81,
    4097.73,
    5044.31,
  ];
  const data2 = [
    1342,
    1301.74,
    1236.66,
    1310.86,
    1245.32,
    1444.58,
    1531.26,
    1745.64,
    1763.1,
    1780.74,
    1673.9,
    1824.56,
    2061.76,
    1855.59,
    2152.49,
    2217.07,
    1995.37,
    2015.33,
    2075.79,
    1972.01,
    1932.57,
    2222.46,
    2200.24,
    2640.29,
  ];
  const data3 = [
    1,
    1.03,
    1.05,
    1.09,
    1.12,
    1.15,
    1.17,
    1.21,
    1.24,
    1.27,
    1.3,
    1.34,
    1.39,
    1.43,
    1.47,
    1.51,
    1.55,
    1.59,
    1.65,
    1.7,
    1.75,
    1.81,
    1.86,
    1.91,
  ];
  const state = {
    labels: dateData,

    datasets: [
      
      {
        label: '100 SOLs',
        backgroundColor: 'rgba(75,192,192,0.3)',
        borderColor: 'rgba(75,192,192,1)',
        borderWidth: 2,
        data: data2,
        fill: true,
      },
      {
        label: '100 tSOLs',
        backgroundColor: 'rgba(78, 102, 222, 0.2)',
        borderColor: '#4e66de',
        borderWidth: 2,
        data: data1,
        fill: true,
      }
    ],
  };

  const state2 = {
    labels: dateData,
    datasets: [
      {
        label: 'SOL',
        backgroundColor: 'rgba(75,192,192,0.3)',
        borderColor: 'rgba(75,192,192,1)',
        borderWidth: 2,
        data: data3,
        fill: true,
      },
    ],
  };
  // const { id } = useParams<{ id: string }>();
  // const lendingReserve = useLendingReserve('SOL');
  // console.log('Reserve:', lendingReserve);
  // const reserve = lendingReserve?.info;

  // if (!reserve || !lendingReserve) {
  //   return null;
  // }

  // Test get LendingReserve
  // connection
  //   .getAccountInfo(
  //     new PublicKey('DKZKU3K8MiBbBfurpaG2ijf6nAGoXvomNysCJZ5jbgiy'),
  //     'single'
  //   )
  //   .then((res) => console.log(res));

  // const deposit = async (params: DepositParams) => {
  //   console.log(`Deposit ${params.amount}`);
  //   const transaction = new Transaction();
  //   transaction.add(await depositInstruction(params));

  //   let tx = await sendTransaction(
  //     connection,
  //     wallet!,
  //     [depositInstruction(params)],
  //     []
  //   );

  //   notify({
  //     message: 'Obligation accounts created',
  //     description: `Transaction ${tx}`,
  //     type: 'success',
  //   });

  //   // await sendAndConfirmTransaction(
  //   //   connection,
  //   //   transaction,
  //   //   [account],
  //   //   {
  //   //     commitment: 'singleGossip',
  //   //     preflightCommitment: 'singleGossip',
  //   //   }
  //   // );
  // };

  return (
    <div
      style={{
        display: 'flex',
        height: '200%',
        width: '100%',
        justifyContent: 'center',
      }}
    >
      <div
        style={{
          width: '60%',
          marginRight: '40px',
          textAlign: 'left',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <div>
          <h1 style={{ fontWeight: 600 }}>
          Projected value of 100 tenderSOLs (tSOLs) and 100 SOLs
          </h1>
          <h3>(compared to HODL SOL)</h3>
          <Card className='card'>
            <Line
              data={state}
              options={{
                legend: {
                  display: true,
                },
                scales: {
                  yAxes: [
                    {
                      ticks: {
                        // Include a dollar sign in the ticks
                        callback: function (value: any) {
                          return '$' + value;
                        },
                      },
                    },
                  ],
                },
              }}
            />
          </Card>
          <h1 style={{ fontWeight: 600, marginTop: '30px' }}>
            Projected value of tSOL/SOL pair
          </h1>
          <Card className='card'>
            <Line
              data={state2}
              options={{
                legend: {
                  display: true,
                },
              }}
            />
          </Card>
        </div>

        <div style={{ marginTop: '30px' }}>
          <h1 style={{ fontWeight: 600 }}>What is tenderSol?</h1>
          <p className='regulartext'>
            TenderSol is a value accruing staking derivative token, which allows
            you to stake or unstake without any no bonding and unbonding period!
          </p>
          <p className='regulartext'>
            Simply put, you can stake yours SOL tokens easily and maximize your
            rewards. You can also trade your tenderSols, use them as a
            collateral or just keep them under the pillow!
          </p>
          <p className='regulartext'>
            Our main focus is to make staking more efficient and easy for Solana
            community. Having a tenderSol token that represents your staking
            position you to use it again, for example as a collateral in other
            protocols.
          </p>
          <p className='regulartext'>
            Find out more about Solana{' '}
            <a
              href='https://solana.com/'
              target='_blank'
              rel='noopener noreferrer'
              className='regulartext'
              style={{ color: '#51C0BF' }}
            >
              here
            </a>
            !
          </p>
        </div>
      </div>   
      <div style={{width: '40%'}}> 
        <div className='sticky' >
          <Card style={{ width: '100%', overflow: 'auto'}}>
            <Tabs defaultActiveKey='1' centered={true}>
              <TabPane tab='Stake' key='1' >
                <div>
                  <img
                    src={solanaLogo}
                    alt='solana logo'
                    width='100'
                    style={{ maxWidth: '30%' }}
                  />
                  <h2 className='subtext' style={{ marginTop: '5px' }}>
                    SOL Token
                  </h2>
                </div>
                <div style={{ marginTop: '5px', marginBottom: '5px' }}>
                  <h2 className='title' style={{ marginBottom: 0 }}>
                    15.5%
                  </h2>
                  <h3 style={{ marginBottom: 0 }}>Rewards</h3>
                  <h5>(projected APY)</h5>
                </div>
                {/* <InputNumber style={{ marginTop: '10px' }}></InputNumber>{' '}
                <span> SOL</span>
                <br /> */}
                <DepositInput />
                {/* <Button
                  className='tenderButton tenderButtonShade'
                  style={{ marginTop: '10px' }}
                  onClick={
                    () => alert('Not implemented')
                    deposit({
                      userSource: account,
                      amount: 100000000000,
                      userToken: '',
                    })
                  }
                >
                  Stake
                </Button> */}
              </TabPane>
              <TabPane tab='Unstake' key='2' >
                <div>
                  <img
                    src={tenderSol}
                    alt='tender solana logo'
                    width='100'
                    style={{ maxWidth: '30%' }}
                  />
                  <h2 className='subtext' style={{ marginTop: '5px' }}>
                    tSOL Token
                  </h2>
                </div>
                <div style={{ marginTop: '5px', marginBottom: '5px' }}>
                  <h2 className='title' style={{ marginBottom: 0 }}>
                    15.5%
                  </h2>
                  <h3 style={{ marginBottom: 0 }}>Rewards</h3>
                  <h5>(projected APY)</h5>
                </div>
                <WithdrawInput />
              </TabPane>
            </Tabs>
          </Card>
        </div>
      </div>
    </div>
  );
};
