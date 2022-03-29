import {
  Borsh,
  StringPublicKey,
  AnyPublicKey,
  ERROR_INVALID_OWNER,
  Account,
  TokenAccount,
} from '@metaplex-foundation/mpl-core';
import BN from 'bn.js';
import bs58 from 'bs58';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import { PassBookProgram } from '../PassBookProgram';
import { AccountKey, PassState, DurationType } from './constants';

type Args = {
  key: AccountKey;
  mint: StringPublicKey;
  authority: StringPublicKey;
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  passType: DurationType;
  duration: number | null;
  passState: PassState;
};

export class PassData extends Borsh.Data<Args> {
  static readonly SCHEMA = PassData.struct([
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
  passType: DurationType;
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

export class Pass extends Account<PassData> {
  static readonly PASS_PREFIX = 'store';
  constructor(pubkey: AnyPublicKey, info: AccountInfo<Buffer>) {
    super(pubkey, info);
    this.data = PassData.deserialize(this.info.data);
    if (!this.assertOwner(PassBookProgram.PUBKEY)) {
      throw ERROR_INVALID_OWNER();
    }
  }

  static async getPDA(mint: AnyPublicKey) {
    return PassBookProgram.findProgramAddress([
      Buffer.from(PassBookProgram.PREFIX),
      PassBookProgram.PUBKEY.toBuffer(),
      new PublicKey(mint).toBuffer(),
      Buffer.from(Pass.PASS_PREFIX),
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
      (account) => Pass.from(account),
    );
  }

  static async findByMint(connection: Connection, mint: AnyPublicKey): Promise<Pass> {
    const pda = await Pass.getPDA(mint);

    return Pass.load(connection, pda);
  }

  static async findInfoByOwner(
    connection: Connection,
    owner: AnyPublicKey,
  ): Promise<Map<AnyPublicKey, AccountInfo<Buffer>>> {
    const accounts = await TokenAccount.getTokenAccountsByOwner(connection, owner);

    const metadataPdaLookups = accounts.reduce((memo, { data }) => {
      // Only include tokens where amount equal to 1.
      // Note: This is not the same as mint supply.
      // NFTs by definition have supply of 1, but an account balance > 1 implies a mint supply > 1.
      return data.amount?.eq(new BN(1)) ? [...memo, Pass.getPDA(data.mint)] : memo;
    }, []);

    const metadataAddresses = await Promise.all(metadataPdaLookups);

    return Account.getInfos(connection, metadataAddresses);
  }

  static async findPassDataByOwner(
    connection: Connection,
    owner: AnyPublicKey,
  ): Promise<PassData[]> {
    const tokenInfo = await Pass.findInfoByOwner(connection, owner);
    return Array.from(tokenInfo.values()).map((m) => PassData.deserialize(m.data));
  }
}
