import {
  Borsh,
  StringPublicKey,
  AnyPublicKey,
  ERROR_INVALID_OWNER,
  Account,
} from '@metaplex-foundation/mpl-core';
import { AccountInfo } from '@solana/web3.js';
import { PassBookProgram } from '../PassBookProgram';
import { AccountKey, PassState, PassType } from './constants';

type Args = {
  key: AccountKey;
  mint: StringPublicKey;
  authority: StringPublicKey;
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  passType: PassType;
  validityPeriod: number | null;
  collectionMint: StringPublicKey | null;
  timeValidationAuthority: StringPublicKey | null;
  passState: PassState;
};

export class PassBookData extends Borsh.Data<Args> {
  static readonly SCHEMA = PassBookData.struct([
    ['key', 'u8'],
    ['mint', 'pubkeyAsString'],
    ['authority', 'pubkeyAsString'],
    ['name', 'string'],
    ['description', 'string'],
    ['uri', 'string'],
    ['mutable', 'u8'],
    ['passType', 'u8'],
    ['validityPeriod', { kind: 'option', type: 'u32' }],
    ['collectionMint', { kind: 'option', type: 'pubkeyAsString' }],
    ['timeValidationAuthority', { kind: 'option', type: 'pubkeyAsString' }],
    ['passState', 'u8'],
  ]);
  key: AccountKey;
  mint: StringPublicKey;
  authority: StringPublicKey;
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  passType: PassType;
  validityPeriod?: number;
  collectionMint?: StringPublicKey;
  timeValidationAuthority?: StringPublicKey;
  passState: PassState;

  constructor(args: Args) {
    super(args);
    const REPLACE = new RegExp('\u0000', 'g');
    this.key = AccountKey.PassBook;
    this.name = args.name.replace(REPLACE, '');
    this.description = args.description.replace(REPLACE, '');
    this.uri = args.uri.replace(REPLACE, '');
  }
}

export class PassBook extends Account<PassBookData> {
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);
    this.data = PassBookData.deserialize(this.info.data);
    if (!this.assertOwner(PassBookProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }
  }
}
