import {
  Borsh,
  StringPublicKey,
  ERROR_INVALID_OWNER,
  AnyPublicKey,
  Account,
} from '@metaplex-foundation/mpl-core';
import { AccountInfo, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { AccountKey } from './constants';
import { PassBookProgram } from '../PassBookProgram';

type Args = {
  key: AccountKey;
  authority: StringPublicKey;
  redemptionsCount: BN;
  passCount: BN;
  passBookCount: BN;
};

export class StoreData extends Borsh.Data<Args> {
  static readonly SCHEMA = StoreData.struct([
    ['key', 'u8'],
    ['authority', 'pubkeyAsString'],
    ['redemptionsCount', 'u64'],
    ['passCount', 'u64'],
    ['passBookCount', 'u64'],
  ]);
  key: AccountKey;
  mint: StringPublicKey;
  authority: StringPublicKey;
  redemptionsCount: string;
  passCount: string;
  passBookCount: string;

  constructor(args: Args) {
    super(args);
    this.key = AccountKey.PassStore;
  }
}

export class Store extends Account<StoreData> {
  static readonly STORE_PREFIX = 'store';
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);
    if (!this.assertOwner(PassBookProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }
    this.data = StoreData.deserialize(this.info.data);
  }

  static async getPDA(authority: AnyPublicKey) {
    return PassBookProgram.findProgramAddress([
      Buffer.from(PassBookProgram.PREFIX),
      PassBookProgram.PUBKEY.toBuffer(),
      new PublicKey(authority).toBuffer(),
      Buffer.from(Store.STORE_PREFIX),
    ]);
  }
}
