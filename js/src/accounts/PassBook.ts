import {
  Borsh,
  StringPublicKey,
  AnyPublicKey,
  ERROR_INVALID_OWNER,
  Account,
} from '@metaplex-foundation/mpl-core';
import bs58 from 'bs58';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
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
    ['authority', 'pubkeyAsString'],
    ['mint', 'pubkeyAsString'],
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

  static async getPDA(mint: AnyPublicKey) {
    return PassBookProgram.findProgramAddress([
      Buffer.from(PassBookProgram.PREFIX),
      PassBookProgram.PUBKEY.toBuffer(),
      new PublicKey(mint).toBuffer(),
    ]);
  }

  static async findMany(
    connection: Connection,
    filters: {
      mint?: AnyPublicKey;
      authority?: AnyPublicKey;
    } = {},
  ) {
    const baseFilters = [
      // Filter for PassBook by key
      {
        memcmp: {
          offset: 0,
          bytes: bs58.encode(Buffer.from([AccountKey.PassBook])),
        },
      },
      // Filter for assigned to authority
      filters.authority && {
        memcmp: {
          offset: 1,
          bytes: new PublicKey(filters.authority).toBase58(),
        },
      },
      // Filter for assigned to mint
      filters.mint && {
        memcmp: {
          offset: 33,
          bytes: new PublicKey(filters.mint).toBase58(),
        },
      },
    ].filter(Boolean);

    return (await PassBookProgram.getProgramAccounts(connection, { filters: baseFilters })).map(
      (account) => PassBook.from(account),
    );
  }

  static async findByMint(connection: Connection, mint: AnyPublicKey): Promise<PassBook> {
    const pda = await PassBook.getPDA(mint);
    return PassBook.load(connection, pda);
  }

  static async findByAuthority(
    connection: Connection,
    authority: AnyPublicKey,
  ): Promise<Account<PassBook>[]> {
    return await PassBook.findMany(connection, {
      authority,
    });
  }
}
