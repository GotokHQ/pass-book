import { clusterApiUrl } from '@solana/web3.js';
import { inspect } from 'util';
import debug from 'debug';
import test from 'tape';
import { LOCALHOST } from '@metaplex-foundation/amman';

export * from './address-labels';
export * from './metadata';
export * from './master-edition';
export * from './collection';

export const logError = debug('pass:tm-test:error');
export const logInfo = debug('pass:tm-test:info');
export const logDebug = debug('pass:tm-test:debug');
export const logTrace = debug('pass:tm-test:trace');

export const programIds = {
  metadata: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  passbook: 'passK9sjcBkUzWu35gf2x4EmpcrkZB9NXgHWtgAzxhB',
};

export const DEVNET = clusterApiUrl('devnet');
export const connectionURL = DEVNET; //process.env.USE_DEVNET != null ? DEVNET : LOCALHOST;

export function dump(x: object) {
  console.log(inspect(x, { depth: 5 }));
}

export function killStuckProcess() {
  // solana web socket keeps process alive for longer than necessary which we
  // "fix" here
  test.onFinish(() => process.exit(0));
}
