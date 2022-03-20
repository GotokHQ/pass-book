import { PublicKey } from '@solana/web3.js';
import { Program } from '@metaplex-foundation/mpl-core';
import { Store } from './accounts';

export class PassBookProgram extends Program {
  static readonly PREFIX = 'passbook';
  static readonly PUBKEY = new PublicKey('passjvPvHQWN4SvBCmHk1gdrtBvoHRERtQK9MKemreQ');

  static async findProgramAuthority(): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [Buffer.from(PassBookProgram.PREFIX, 'utf8'), PassBookProgram.PUBKEY.toBuffer()],
      PassBookProgram.PUBKEY,
    );
  }

  static async findPassStoreAccount(authority: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(PassBookProgram.PREFIX, 'utf8'),
        PassBookProgram.PUBKEY.toBuffer(),
        authority.toBuffer(),
        Buffer.from(Store.STORE_PREFIX, 'utf8'),
      ],
      PassBookProgram.PUBKEY,
    );
  }

  static async findPassBookAccount(mint: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(PassBookProgram.PREFIX, 'utf8'),
        PassBookProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
      ],
      PassBookProgram.PUBKEY,
    );
  }

  static async findPassAccount(mint: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(PassBookProgram.PREFIX, 'utf8'),
        PassBookProgram.PUBKEY.toBuffer(),
        mint.toBuffer(),
      ],
      PassBookProgram.PUBKEY,
    );
  }
}
