import { Borsh, StringPublicKey, Transaction } from '@metaplex-foundation/mpl-core';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionCtorFields,
  TransactionInstruction,
} from '@solana/web3.js';
import { PassType } from 'src/accounts/constants';
import { PassBookProgram } from '../PassBookProgram';

type Args = {
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  validityPeriod: number | null;
  collectionMint?: StringPublicKey | null;
  timeValidationAuthority?: StringPublicKey | null;
  passType: PassType;
};

export class InitPassBookArgs extends Borsh.Data<Args> {
  static readonly SCHEMA = InitPassBookArgs.struct([
    ['instruction', 'u8'],
    ['name', 'string'],
    ['description', 'string'],
    ['uri', 'string'],
    ['mutable', 'u8'],
    ['validityPeriod', { kind: 'option', type: 'u32' }],
    ['collectionMint', { kind: 'option', type: 'pubkeyAsString' }],
    ['timeValidationAuthority', { kind: 'option', type: 'pubkeyAsString' }],
    ['passType', 'u8'],
  ]);

  instruction = 0;
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  validityPeriod?: number;
  collectionMint?: StringPublicKey;
  timeValidationAuthority?: StringPublicKey;
  passType: PassType;
}

export type InitPassBookParams = {
  name: string;
  description: string;
  uri: string;
  mutable: boolean;
  authority: PublicKey;
  masterMetadata: PublicKey;
  masterEdition: PublicKey;
  store: PublicKey;
  source: PublicKey;
  passBook: PublicKey;
  mint: PublicKey;
  validityPeriod?: number;
  collectionMint?: PublicKey;
  timeValidationAuthority?: PublicKey;
  passType: PassType;
  tokenAccount: PublicKey;
};

export class InitPassBook extends Transaction {
  constructor(options: TransactionCtorFields, params: InitPassBookParams) {
    super(options);
    const { feePayer } = options;
    const {
      name,
      description,
      uri,
      mutable,
      passBook,
      source,
      store,
      authority,
      masterMetadata,
      masterEdition,
      mint,
      validityPeriod,
      collectionMint,
      timeValidationAuthority,
      passType,
      tokenAccount
    } = params;

    const data = InitPassBookArgs.serialize({
      name,
      description,
      uri,
      mutable,
      validityPeriod,
      collectionMint: collectionMint?.toString(),
      timeValidationAuthority: timeValidationAuthority?.toString(),
      passType,
    });
    this.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: passBook,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: source,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: tokenAccount,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: store,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: authority,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: feePayer,
            isSigner: true,
            isWritable: false,
          },
          {
            pubkey: mint,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: masterMetadata,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: masterEdition,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          },
        ],
        programId: PassBookProgram.PUBKEY,
        data,
      }),
    );
  }
}
