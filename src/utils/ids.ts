import { PublicKey } from '@solana/web3.js';

export const TENDERIZE_PROGRAM_ID = new PublicKey(
  'YiLSwwu3yBwaSHoaAooERKNeQ64BSbRuv7wXTZ5Uprf'
);

export const TOKEN_PROGRAM_ID = new PublicKey(
  'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
);

export const TENDERIZED_SOL_MINT_ID = new PublicKey(
  'Do41x7Uza6cQeJ2VhVyJU6QhuRMLxTrLJhpL8MSNFpZm'
);

export let STAKE_POOL_ID = new PublicKey(
  'DKZKU3K8MiBbBfurpaG2ijf6nAGoXvomNysCJZ5jbgiy'
);

export let VALIDATORS_LIST_ID = new PublicKey(
  '7qaWgz27453E5CLPvcvGiLR2GobLPbKWsPmGDNQYRm6U'
);

export const WRAPPED_SOL_MINT = new PublicKey(
  'So11111111111111111111111111111111111111112'
);

export const PROGRAM_IDS = [
  {
    name: 'mainnet-beta',
  },
  {
    name: 'testnet',
  },
  {
    name: 'devnet',
  },
  {
    name: 'localnet',
  },
];

export const setProgramIds = (envName: string) => {
  let instance = PROGRAM_IDS.find((env) => env.name === envName);
  if (!instance) {
    return;
  }
};

export const programIds = () => {
  return {
    tenderize: TENDERIZE_PROGRAM_ID,
    token: TOKEN_PROGRAM_ID,
  };
};
