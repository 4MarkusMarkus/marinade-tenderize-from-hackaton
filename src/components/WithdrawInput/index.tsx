import React, { useCallback, useState, useEffect } from 'react';
import {
  InputType,
  useSliderInput,
  useTokenName,
  useUserBalance,
} from '../../hooks';
import { TokenIcon } from '../TokenIcon';
import { Card, Slider } from 'antd';
import { NumericInput } from '../Input/numeric';
import { useConnection } from '../../contexts/connection';
import { useWallet } from '../../contexts/wallet';
import { ActionConfirmation } from './../ActionConfirmation';
import { LABELS, marks } from '../../constants';
import { RESERVE_ADDRESS_PDA, TENDERIZED_SOL_MINT_ID } from '../../utils/ids';
import { withdraw, credit } from '../../actions';
import { useTenderize } from '../../contexts/tenderize';
import { ConnectButton } from '../ConnectButton';
import { AccountLayout } from '@solana/spl-token';
import { AccountInfo } from '@solana/web3.js';

export const WithdrawInput = (props: { className?: string }) => {
  const connection = useConnection();
  const { wallet } = useWallet();
  const [pendingTx, setPendingTx] = useState(false);
  const [reserve, setReserve] = useState<AccountInfo<Buffer> | null>();
  const [minimumBalance, setMinimumBalance] = useState(0);
  const [showConfirmation, setShowConfirmation] = useState(false);

  const tenderize = useTenderize();

  useEffect(() => {
    if (!connection) {
      return;
    }
    connection.getAccountInfo(RESERVE_ADDRESS_PDA).then((res) => {
      setReserve(res);
    });
  }, [connection]);

  useEffect(() => {
    if (!connection || !AccountLayout) {
      return;
    }
    connection
      .getMinimumBalanceForRentExemption(AccountLayout.span)
      .then((rent) => {
        setMinimumBalance(rent + 1);
      });
  }, [connection]);

  const isReserveDepleted = useCallback(
    (amount: number) => !reserve || reserve.lamports < minimumBalance + amount,
    [minimumBalance, reserve]
  );

  const name = useTokenName(TENDERIZED_SOL_MINT_ID);
  const { accounts: fromAccounts, balance, balanceLamports } = useUserBalance(
    TENDERIZED_SOL_MINT_ID
  );

  const convert = useCallback(
    (val: string | number) => {
      if (typeof val === 'string') {
        return (parseFloat(val) / balance) * 100;
      } else {
        return ((val * balance) / 100).toFixed(2);
      }
    },
    [balance]
  );

  const { value, setValue, pct, setPct, type } = useSliderInput(convert);

  const amount =
    type === InputType.Percent
      ? (pct * balanceLamports) / 100
      : Math.ceil(balanceLamports * (parseFloat(value) / balance));

  const onWithdraw = useCallback(() => {
    setPendingTx(true);

    (async () => {
      try {
        !isReserveDepleted(amount)
          ? await withdraw(
              fromAccounts[0],
              amount,
              connection,
              wallet!,
              tenderize!.info
            )
          : await credit(
              fromAccounts[0],
              amount,
              connection,
              wallet!,
              tenderize!.info
            );
        setValue('');
        setShowConfirmation(true);
      } catch (e) {
        console.log(`error: ${e}`);
      } finally {
        setPendingTx(false);
      }
    })();
  }, [
    amount,
    connection,
    fromAccounts,
    setValue,
    wallet,
    tenderize,
    isReserveDepleted,
  ]);

  const bodyStyle: React.CSSProperties = {
    display: 'flex',
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    height: '100%',
  };

  return (
    <Card className={props.className} bodyStyle={bodyStyle}>
      {showConfirmation ? (
        <ActionConfirmation onClose={() => setShowConfirmation(false)} />
      ) : (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-around',
          }}
        >
          <div className='deposit-input-title'>{LABELS.WITHDRAW_QUESTION}</div>
          <div className='token-input'>
            <TokenIcon mintAddress={TENDERIZED_SOL_MINT_ID} />
            <NumericInput
              value={value}
              onChange={setValue}
              autoFocus={true}
              style={{
                fontSize: 20,
                boxShadow: 'none',
                borderColor: 'transparent',
                outline: 'transparent',
              }}
              placeholder='0.00'
            />
            <div>{name}</div>
          </div>
          <Slider marks={marks} value={pct} onChange={setPct} />
          <ConnectButton
            className='tenderButton tenderButtonShade'
            type='primary'
            onClick={onWithdraw}
            loading={pendingTx}
            disabled={fromAccounts.length === 0}
          >
            {!isReserveDepleted(amount)
              ? LABELS.WITHDRAW_ACTION
              : LABELS.GET_IN_LINE_ACTION}
          </ConnectButton>
          {isReserveDepleted(amount) && (
            <div>
              Our reserve pool for instant withdrawals has not enough funds for
              now, but we will send you your tenderized stake back shortly, in
              the meantime, you still get rewards from staking.
            </div>
          )}
        </div>
      )}
    </Card>
  );
};
