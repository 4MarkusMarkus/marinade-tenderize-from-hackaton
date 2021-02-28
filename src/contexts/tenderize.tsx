import React, { useContext, useEffect, useState } from 'react';
import { Tenderize, TenderizeParser } from '../models/lending';
import { STAKE_POOL_ID } from '../utils/ids';
import { ParsedAccount } from './accounts';
import { useConnection } from './connection';

export const TenderizeContext = React.createContext<ParsedAccount<Tenderize> | undefined>(undefined);

export function useTenderize() {
  return useContext(TenderizeContext)!
}

export function TenderizeProvider({ children = undefined as any }) {
  const [tenderizeAccount, setTenderizeAccount] = useState<
    ParsedAccount<Tenderize>
  >();

  const connection = useConnection();

  useEffect(() => {
    connection.getAccountInfo(STAKE_POOL_ID).then((acc) => {
      setTenderizeAccount(TenderizeParser(STAKE_POOL_ID, acc!));
    });
    const listenerId = connection.onAccountChange(STAKE_POOL_ID, (acc) => {
      setTenderizeAccount(TenderizeParser(STAKE_POOL_ID, acc!));
    })

    return () => {
      connection.removeAccountChangeListener(listenerId);
    };
  }, [setTenderizeAccount, connection]);


  return (
    <TenderizeContext.Provider value={tenderizeAccount}>
      {children}
    </TenderizeContext.Provider>
  );
}