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
import { deposit } from '../../actions/deposit';
import { ActionConfirmation } from './../ActionConfirmation';
import { LABELS, marks } from '../../constants';
import { WRAPPED_SOL_MINT } from '../../utils/ids';
import { ConnectButton } from '../ConnectButton';
import { AccountLayout } from '@solana/spl-token';

export const DepositInput = (props: { className?: string }) => {
  const connection = useConnection();
  const { wallet } = useWallet();
  const [pendingTx, setPendingTx] = useState(false);
  const [minimumBalance, setMinimumBalance] = useState(0);
  const [showConfirmation, setShowConfirmation] = useState(false);

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

  const name = useTokenName(WRAPPED_SOL_MINT);
  const { accounts: fromAccounts, balance, balanceLamports } = useUserBalance(
    WRAPPED_SOL_MINT
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

  const onDeposit = useCallback(() => {
    setPendingTx(true);

    const amount =
      type === InputType.Percent
        ? (pct * (balanceLamports - minimumBalance * 2)) / 100
        : Math.ceil(
            (balanceLamports - minimumBalance) * (parseFloat(value) / balance)
          );

    (async () => {
      try {
        await deposit(fromAccounts[0], amount, connection, wallet!);
        setValue('');
        setShowConfirmation(true);
      } catch (e) {
        console.log(`error: ${e}`);
      } finally {
        setPendingTx(false);
      }
    })();
  }, [
    type,
    pct,
    balanceLamports,
    value,
    balance,
    minimumBalance,
    fromAccounts,
    connection,
    wallet,
    setValue,
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
          <div className='deposit-input-title'>{LABELS.DEPOSIT_QUESTION}</div>
          <div className='token-input'>
            <TokenIcon mintAddress={WRAPPED_SOL_MINT} />
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
            onClick={onDeposit}
            loading={pendingTx}
            disabled={fromAccounts.length === 0}
          >
            {LABELS.DEPOSIT_ACTION}
          </ConnectButton>
        </div>
      )}
    </Card>
  );
};
