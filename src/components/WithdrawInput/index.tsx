import React, { useCallback, useState } from 'react';
import {
  useSliderInput,
} from '../../hooks';
// import { LendingReserve } from '../../models/lending';
import { TokenIcon } from '../TokenIcon';
import { Button, Card } from 'antd';
import { NumericInput } from '../Input/numeric';
import { useConnection } from '../../contexts/connection';
import { useWallet } from '../../contexts/wallet';
// import { PublicKey } from '@solana/web3.js';
import { ActionConfirmation } from './../ActionConfirmation';
import { LABELS } from '../../constants';
import { TENDERIZED_SOL_MINT_ID, WRAPPED_SOL_MINT } from '../../utils/ids';
import { withdraw } from '../../actions';
import { useAccountByMint } from '../../contexts/accounts';
import { useTenderize } from '../../contexts/tenderize';
import BN from 'bn.js';

export const WithdrawInput = (props: { className?: string }) => {
  const connection = useConnection();
  const { wallet } = useWallet();
  const [pendingTx, setPendingTx] = useState(false);
  const [showConfirmation, setShowConfirmation] = useState(false);

  const from = useAccountByMint(TENDERIZED_SOL_MINT_ID.toBase58());

  const { value, setValue } = useSliderInput((val) => {
    return val; // TODO
  });

  const tenderize = useTenderize();

  const onWithdraw = useCallback(() => {
    setPendingTx(true);

    (async () => {
      try {
        await withdraw(from!, 100000000, connection, wallet!, tenderize!.info);

        setValue('');
        setShowConfirmation(true);
      } catch (e) {
        console.log(`error: ${e}`);
      } finally {
        setPendingTx(false);
      }
    })();
  }, [
    connection,
    setValue,
    from,
    wallet,
    tenderize
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
              <div>tSOL</div>
            </div>

            <Button
              type='primary'
              onClick={onWithdraw}
              loading={pendingTx}
              disabled={!from || from.info.amount.eq(new BN(0))}
            >
              {LABELS.WITHDRAW_ACTION}
            </Button>
          </div>
        )}
    </Card>
  );
};
