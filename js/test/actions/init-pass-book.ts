import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { logDebug } from '../utils';
import { TransactionHandler } from '@metaplex-foundation/amman';
import * as spl from '@solana/spl-token';
import { InitPassBook, PassBook, PassType } from '../../src/pass-book';

type InitPassBookParams = {
  passBook: PublicKey;
  mint: PublicKey;
  masterMetadata: PublicKey;
  masterEdition: PublicKey;
  authority: PublicKey;
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  source: PublicKey;
  store: PublicKey;
  passType: PassType;
  validityPeriod?: number;
  collectionMint?: PublicKey;
  timeValidationAuthority?: PublicKey;
};

// -----------------
// Create a SPL Token account to receive tokens
// -----------------
export async function createTokenAccount(
  connection: Connection,
  transactionHandler: TransactionHandler,
  payer: PublicKey,
) {
  const tokenAccount = Keypair.generate();
  const tx = new Transaction({
    feePayer: payer,
  });
  tx.add(
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: tokenAccount.publicKey,
      lamports: await spl.Token.getMinBalanceRentForExemptAccount(connection),
      space: spl.AccountLayout.span,
      programId: spl.TOKEN_PROGRAM_ID,
    }),
  );
  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(tx, [tokenAccount], {
    skipPreflight: false,
  });
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));
  return tokenAccount.publicKey;
}

export async function initPassBook(
  connection: Connection,
  transactionHandler: TransactionHandler,
  publicKey: PublicKey,
  {
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
    validityPeriod,
    collectionMint,
    timeValidationAuthority,
  }: InitPassBookParams,
) {
  const passBook = await PassBook.getPDA(mint);
  const tokenAccount = await createTokenAccount(connection, transactionHandler, publicKey);
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
      store,
      passType,
      tokenAccount,
      validityPeriod,
      collectionMint,
      timeValidationAuthority,
    },
  );
  const createTxDetails = await transactionHandler.sendAndConfirmTransaction(initPassTx, [], {
    skipPreflight: true,
  });
  return { passBook, createTxDetails };
}
