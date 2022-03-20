import test from 'tape';
import spok from 'spok';

import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { defaultSendOptions, TransactionHandler } from '@metaplex-foundation/amman';

import {
  CreateMetadata,
  CreateMetadataV2,
  DataV2,
  Metadata,
  MetadataData,
  MetadataDataData,
} from '@metaplex-foundation/mpl-token-metadata';
import * as spl from '@solana/spl-token';
import { addLabel, logDebug } from '.';

export const URI = 'uri';
export const NAME = 'test';
export const SYMBOL = 'sym';
export const SELLER_FEE_BASIS_POINTS = 10;

type CreateMetadataV2Params = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  mint: PublicKey;
  metadataData: DataV2;
  updateAuthority?: PublicKey;
};

type CreateMetadataParams = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  editionMint: PublicKey;
  metadataData: MetadataDataData;
  updateAuthority?: PublicKey;
};

export async function createMetadata({
  transactionHandler,
  publicKey,
  editionMint,
  metadataData,
  updateAuthority,
}: CreateMetadataParams) {
  const metadata = await Metadata.getPDA(editionMint);
  const createMetadataTx = new CreateMetadata(
    { feePayer: publicKey },
    {
      metadata,
      metadataData,
      updateAuthority: updateAuthority ?? publicKey,
      mint: editionMint,
      mintAuthority: publicKey,
    },
  );

  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(
    createMetadataTx,
    [],
    defaultSendOptions,
  );

  return { metadata, createTxDetails };
}

export async function createMetadataV2({
  transactionHandler,
  publicKey,
  mint,
  metadataData,
  updateAuthority,
}: CreateMetadataV2Params) {
  const metadata = await Metadata.getPDA(mint);
  const createMetadataTx = new CreateMetadataV2(
    { feePayer: publicKey },
    {
      metadata,
      metadataData,
      updateAuthority: updateAuthority ?? publicKey,
      mint: mint,
      mintAuthority: publicKey,
    },
  );

  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(createMetadataTx, [], {
    skipPreflight: false,
  });

  return { metadata, createTxDetails };
}

export async function getMetadataData(
  connection: Connection,
  metadata: PublicKey,
): Promise<MetadataData> {
  const metadataAccount = await connection.getAccountInfo(metadata);
  return MetadataData.deserialize(metadataAccount.data);
}

export async function mintAndCreateMetadataV2(
  connection: Connection,
  transactionHandler: TransactionHandler,
  payer: Keypair,
  args: DataV2,
) {
  const mint = await spl.Token.createMint(
    connection,
    payer,
    payer.publicKey,
    null,
    0,
    spl.TOKEN_PROGRAM_ID,
  );

  const source = await mint.getOrCreateAssociatedAccountInfo(payer.publicKey);

  await mint.mintTo(source.address, payer.publicKey, [], 1);
  addLabel('mint', mint.publicKey);
  const initMetadataData = args;
  const { createTxDetails, metadata } = await createMetadataV2({
    transactionHandler,
    publicKey: payer.publicKey,
    mint: mint.publicKey,
    metadataData: initMetadataData,
  });

  addLabel('metadata', metadata);
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));
  return { mint, metadata, source: source.address };
}

export async function assertMetadataDataUnchanged(
  t: test.Test,
  initial: MetadataData,
  updated: MetadataData,
  except?: keyof MetadataData,
) {
  const x = { ...initial };
  if (except != null) {
    delete x[except];
  }
  delete x.data.creators;
  delete x.tokenStandard;
  delete x.collection;
  delete x.uses;

  const y = { $topic: `no change except '${except}' on metadata`, ...updated };
  if (except != null) {
    delete y[except];
  }
  delete y.data.creators;
  delete y.tokenStandard;
  delete y.collection;
  delete y.uses;

  spok(t, x, y);
}

export async function assertMetadataDataDataUnchanged(
  t: test.Test,
  initial: MetadataDataData,
  updated: MetadataDataData,
  except: (keyof MetadataDataData)[],
) {
  const x = { ...initial };
  except.forEach((f) => delete x[f]);
  delete x.creators;

  const y = { $topic: `no change except '${except}' on metadataData`, ...updated };
  except.forEach((f) => delete y[f]);
  delete y.creators;

  spok(t, x, y);
}
