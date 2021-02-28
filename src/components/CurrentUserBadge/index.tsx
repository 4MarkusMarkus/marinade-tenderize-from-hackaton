import React, { useCallback } from 'react';
import { useWallet } from '../../contexts/wallet';
import { useConnection } from '../../contexts/connection';
import { formatNumber, shortenAddress } from '../../utils/utils';
import { Identicon } from '../Identicon';
import { useNativeAccount } from '../../contexts/accounts';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { notify } from '../../utils/notifications';
import { LABELS } from '../../constants';
import { Button } from 'antd';
import { useUserBalance } from '../../hooks';
import { TENDERIZED_SOL_MINT_ID } from '../../utils/ids';

export const CurrentUserBadge = (props: {}) => {
  const connection = useConnection();
  const { wallet, publicKey } = useWallet();

  const airdrop = useCallback(() => {
    if (!publicKey) {
      return;
    }

    connection.requestAirdrop(publicKey, 2 * LAMPORTS_PER_SOL).then(() => {
      notify({
        message: LABELS.ACCOUNT_FUNDED,
        type: 'success',
      });
    });
  }, [publicKey, connection]);

  const { account } = useNativeAccount();
  const { balance } = useUserBalance(TENDERIZED_SOL_MINT_ID);

  if (!wallet?.publicKey) {
    return null;
  }

  // should use SOL ◎ ?

  return (
    <div className='wallet-wrapper'>
      <Button onClick={airdrop}>GET SOL</Button>
      <span style={{ marginLeft: '0.5rem' }}>
        {formatNumber.format((account?.lamports || 0) / LAMPORTS_PER_SOL)} SOL
      </span>
      <span style={{ marginLeft: '0.5rem' }}>
        {formatNumber.format(balance)} TSOL
      </span>
      <div className='wallet-key'>
        {shortenAddress(`${wallet.publicKey}`)}
        <Identicon
          address={wallet.publicKey.toBase58()}
          style={{ marginLeft: '0.5rem', display: 'flex' }}
        />
      </div>
    </div>
  );
};
