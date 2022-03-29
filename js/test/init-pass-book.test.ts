import test from 'tape';
import spok from 'spok';
import { Connection, Keypair } from '@solana/web3.js';
import { connectionURL, createMasterEdition, killStuckProcess } from './utils';
import { airdrop, PayerTransactionHandler } from '@metaplex-foundation/amman';

import BN from 'bn.js';

import { logDebug } from './utils';
import { addLabel, isKeyOf } from './utils/address-labels';
import { initPassBook } from './actions';
import { DataV2 } from '@metaplex-foundation/mpl-token-metadata';
import {
  AccountKey,
  DurationType,
  PassBook,
  PassBookData,
  PassBookDataArgs,
  PassState,
  Store,
} from '../src/accounts';

killStuckProcess();

const URI = 'uri';
const NAME = 'my pass book';
const DESCRIPTION = 'test';
const SYMBOL = 'sym';
const SELLER_FEE_BASIS_POINTS = 10;

test('init-pass-book-account: success', async (t) => {
  const payer = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);
  await airdrop(connection, payer.publicKey, 2);

  const initMetadataData = new DataV2({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
    collection: null,
    uses: null,
  });

  const master = await createMasterEdition(
    connection,
    transactionHandler,
    payer,
    initMetadataData,
    100,
  );
  const passBookPDA = await PassBook.getPDA(master.mint.publicKey);
  const store = await Store.getPDA(payer.publicKey);
  const { createTxDetails } = await initPassBook(connection, transactionHandler, payer.publicKey, {
    store,
    passBook: passBookPDA,
    uri: URI,
    name: NAME,
    description: DESCRIPTION,
    source: master.source,
    masterEdition: master.masterEditionPubkey,
    masterMetadata: master.metadata,
    mint: master.mint.publicKey,
    authority: payer.publicKey,
    mutable: true,
    durationType: DurationType.Days,
    duration: new BN(30),
    maxSupply: new BN(100),
  });
  logDebug(createTxDetails.txSummary.logMessages.join('\n'));

  //   assertTransactionSummary(t, createTxDetails.txSummary, {
  //     fee: 5000,
  //     msgRx: [/Program.+metaq/i, /Instruction.+ Create Metadata Accounts/i],
  //   });
  const passBookAccount = await connection.getAccountInfo(passBookPDA);
  logDebug({
    passBookAccountOwner: passBookAccount.owner.toBase58(),
    passBookAccountDataBytes: passBookAccount.data.byteLength,
  });

  // console.log('data', JSON.stringify((<Buffer>passBookAccount.data).toJSON().data));
  const passBookData = PassBookData.deserialize<PassBookData, PassBookDataArgs>(
    <Buffer>passBookAccount.data,
  );
  t.assert(passBookData.duration.eq(new BN(30)), 'Duration invalid');
  t.assert(passBookData.maxSupply.eq(new BN(100)), 'Max supply invalid');
  t.assert(passBookData.totalPasses.eq(new BN(0)), 'total passes should be 0');
  spok(t, passBookData, {
    $topic: 'passBookData',
    key: AccountKey.PassBook,
    authority: isKeyOf(payer),
    mint: isKeyOf(master.mint.publicKey),
    name: NAME,
    description: DESCRIPTION,
    uri: URI,
    mutable: 1,
    durationType: DurationType.Days,
    passState: PassState.NotActivated,
  });
  // console.log('authority ', payer.publicKey.toString());
  // console.log('passBook ', passBookPDA.toString());
  // console.log('mint ', master.mint.publicKey.toString());
  // console.log('mutable ', true);
  // console.log('durationType ', DurationType.Days);
  // console.log('duration ', 30);
  // console.log('maxSupply ', 100);
  // console.log('name ', passBookData.name);
  // console.log('uri ', passBookData.uri);
  // console.log('description ', passBookData.description);
});

test('init-pass-book-account: failure', async (t) => {
  const payer = Keypair.generate();
  addLabel('create:payer', payer);

  const connection = new Connection(connectionURL, 'confirmed');
  const transactionHandler = new PayerTransactionHandler(connection, payer);

  await airdrop(connection, payer.publicKey, 2);

  const initMetadataData = new DataV2({
    uri: URI,
    name: NAME,
    symbol: SYMBOL,
    sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
    creators: null,
    collection: null,
    uses: null,
  });

  const master = await createMasterEdition(
    connection,
    transactionHandler,
    payer,
    initMetadataData,
    0,
  );
  const passBookPDA = await PassBook.getPDA(master.mint.publicKey);
  const fakeAdmin = Keypair.generate();
  const store = await Store.getPDA(fakeAdmin.publicKey);
  const { createTxDetails } = await initPassBook(connection, transactionHandler, payer.publicKey, {
    store,
    passBook: passBookPDA,
    uri: URI,
    name: NAME,
    description: DESCRIPTION,
    source: master.source,
    masterEdition: master.masterEditionPubkey,
    masterMetadata: master.metadata,
    mint: master.mint.publicKey,
    authority: payer.publicKey,
    mutable: true,
    durationType: DurationType.Days,
    duration: new BN(30),
    maxSupply: new BN(50),
  });
  console.log('createTxDetails', createTxDetails);

  logDebug(createTxDetails.txSummary.logMessages.join('\n'));

  t.deepEqual(
    createTxDetails.txSummary.err,
    { InstructionError: [0, { Custom: 3 }] },
    'Init pass account is invalid',
  );
});
