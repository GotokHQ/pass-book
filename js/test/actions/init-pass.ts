import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { addLabel, logDebug } from '../utils';
import { defaultSendOptions, TransactionHandler } from '@metaplex-foundation/amman';
import {
  CreateMetadata,
  CreateMasterEditionV3,
  CreateMetadataV2,
  DataV2,
  MasterEdition,
  Metadata,
  MetadataDataData,
} from '@metaplex-foundation/mpl-token-metadata';
import { InitPassBook, PassBook, PassType } from '../../src/pass-book';
import BN from 'bn.js';
import * as spl from '@solana/spl-token';
// -----------------
// Create Metadata
// -----------------
// src/actions/createMetadata.ts
type CreateMetadataParams = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  editionMint: PublicKey;
  metadataData: MetadataDataData;
  updateAuthority?: PublicKey;
};

type CreateMetadataV2Params = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  mint: PublicKey;
  metadataData: DataV2;
  updateAuthority?: PublicKey;
};

type InitPassBookParams = {
  transactionHandler: TransactionHandler;
  publicKey: PublicKey;
  mint: PublicKey;
  masterMetadata: PublicKey;
  masterEdition: PublicKey;
  authority: PublicKey;
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  source: PublicKey;
  tokenAccount: PublicKey;
  store: PublicKey;
  passType: PassType;
};

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

// -----------------
// Prepare Mint and Create Metaata
// -----------------
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
  return { mint, metadata, source };
}

// -----------------
// Create a SPL Token account to receive tokens
// -----------------
export async function createTokenAccount(
  connection: Connection,
  transactionHandler: TransactionHandler,
  payer: Keypair,
) {
  const tokenAccount = Keypair.generate();
  const tx = new Transaction({
    feePayer: payer.publicKey,
  });
  tx.add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: tokenAccount.publicKey,
      lamports: await spl.Token.getMinBalanceRentForExemptAccount(connection),
      space: spl.AccountLayout.span,
      programId: spl.TOKEN_PROGRAM_ID,
    }),
  );
  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(tx, [], {
    skipPreflight: false,
  });
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));
  return tokenAccount.publicKey;
}

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
  const { mint, metadata } = await mintAndCreateMetadataV2(
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

  return { mint, metadata, masterEditionPubkey, createTxDetails };
}

async function createPassBook(
  connection: Connection,
  {
    transactionHandler,
    publicKey,
    name,
    description,
    uri,
    mint,
    masterMetadata,
    masterEdition,
    mutable,
    source,
    store,
    authority,
    passType,
  }: InitPassBookParams,
) {
  const passBook = await PassBook.getPDA(mint);
  const tokenAccount = Keypair.generate();
  const initPassTx = new InitPassBook(
    { feePayer: publicKey },
    {
      authority,
      passBook,
      name,
      description,
      uri,
      mutable,
      mint,
      masterMetadata,
      masterEdition,
      source,
      tokenAccount: tokenAccount.publicKey,
      store,
      passType,
    },
  );
  initPassTx.instructions.unshift(
    SystemProgram.createAccount({
      fromPubkey: publicKey,
      newAccountPubkey: tokenAccount.publicKey,
      lamports: await spl.Token.getMinBalanceRentForExemptAccount(connection),
      space: spl.AccountLayout.span,
      programId: spl.TOKEN_PROGRAM_ID,
    }),
  );
  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(initPassTx, [], {
    skipPreflight: true,
  });
  return { passBook, createTxDetails };
}
