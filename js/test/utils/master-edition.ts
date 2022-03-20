import { Connection, Keypair } from '@solana/web3.js';
import { TransactionHandler } from '@metaplex-foundation/amman';

import BN from 'bn.js';
import {
  CreateMasterEditionV3,
  DataV2,
  MasterEdition,
} from '@metaplex-foundation/mpl-token-metadata';
import { mintAndCreateMetadataV2 } from './metadata';

// -----------------
// Create A Master Edition
// -----------------
export async function createMasterEdition(
  connection: Connection,
  transactionHandler: TransactionHandler,
  payer: Keypair,
  args: DataV2,
  maxSupply: number,
) {
  const { mint, metadata, source } = await mintAndCreateMetadataV2(
    connection,
    transactionHandler,
    payer,
    args,
  );

  const masterEditionPubkey = await MasterEdition.getPDA(mint.publicKey);
  const createMev3 = new CreateMasterEditionV3(
    { feePayer: payer.publicKey },
    {
      edition: masterEditionPubkey,
      metadata: metadata,
      updateAuthority: payer.publicKey,
      mint: mint.publicKey,
      mintAuthority: payer.publicKey,
      maxSupply: new BN(maxSupply),
    },
  );

  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(createMev3, [], {
    skipPreflight: true,
  });

  return { mint, metadata, masterEditionPubkey, source, createTxDetails };
}
