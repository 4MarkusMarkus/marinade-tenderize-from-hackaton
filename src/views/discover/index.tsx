import React from 'react';
// import { useLendingReserve } from '../../hooks';
// import { useParams } from 'react-router-dom';
// import { Link } from 'react-router-dom';

// import { DepositInput } from '../../components/DepositInput';
import { Button, Card, Tabs, InputNumber } from 'antd';
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
const { TabPane } = Tabs;

export const DiscoverView = () => {
  // const connection = useConnection();

  // const { wallet } = useWallet();

  const state = {
    labels: [
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
    ],

    datasets: [
      {
        label: 'SOL',
        backgroundColor: 'rgba(75,192,192,1)',
        borderColor: 'rgba(75,192,192,1)',
        borderWidth: 2,
        data: [
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
        ],
        fill: false,
      },
      {
        label: 'tSOL',
        backgroundColor: 'orange',
        borderColor: 'orange',
        borderWidth: 2,
        data: [
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
        ],
        fill: false,
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
          marginRight: '30px',
          textAlign: 'left',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <div>
          <h1>Price comparison of 100 tokens.</h1>
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
        </div>
        <div style={{ marginTop: '30px' }}>
          <h1>tSOL Vault Rewards</h1>
          <Card className='card'>
            <Line
              data={state}
              options={{
                legend: {
                  display: false,
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
        </div>
        <div style={{ marginTop: '30px' }}>
          <h1>About SOL</h1>
          <p className='subtext'>
            Solana is a fast, secure, and censorship resistant blockchain
            providing the open infrastructure required for global adoption.
          </p>
        </div>
      </div>

      <div
        className='card'
        style={{ height: '450px', width: '30%', position: 'sticky', top: 0 }}
      >
        <Card className='card' style={{ width: '100%', height: '100%' }}>
          <Tabs defaultActiveKey='1' centered={true}>
            <TabPane tab='Stake' key='1'>
              <div>
                <img
                  src={solanaLogo}
                  alt='solana logo'
                  style={{ maxWidth: '30%' }}
                />
                <h2 className='subtext' style={{ marginTop: '5px' }}>
                  SOLANA
                </h2>
              </div>
              <div style={{ marginTop: '5px', marginBottom: '5px' }}>
                <h2 className='title' style={{ marginBottom: 0 }}>
                  15.5%
                </h2>
                <h3 style={{ marginBottom: 0 }}>Vault Rewards</h3>
                <h5>(projected APY)</h5>
              </div>
              <InputNumber style={{ marginTop: '10px' }}></InputNumber>{' '}
              <span> SOL</span>
              <br />
              <DepositInput />
              <Button
                className='tenderButton tenderButtonShade'
                style={{ marginTop: '10px' }}
                onClick={
                  () => alert('Not implemented')
                  /*deposit({
                    userSource: account,
                    amount: 100000000000,
                    userToken: '',
                  })*/
                }
              >
                Stake
              </Button>
            </TabPane>
            <TabPane tab='Unstake' key='2'>
              <div>
                <img
                  src={solanaLogo}
                  alt='solana logo'
                  style={{ maxWidth: '30%' }}
                />
                <h2 className='subtext' style={{ marginTop: '5px' }}>
                  SOLANA
                </h2>
              </div>
              <div style={{ marginTop: '5px', marginBottom: '5px' }}>
                <h2 className='title' style={{ marginBottom: 0 }}>
                  15.5%
                </h2>
                <h3 style={{ marginBottom: 0 }}>Vault Rewards</h3>
                <h5>(projected APY)</h5>
              </div>
              <WithdrawInput />

            </TabPane>
          </Tabs>
        </Card>
      </div>
    </div>
  );
};
