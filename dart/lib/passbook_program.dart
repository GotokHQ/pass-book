import 'package:passbook/accounts/store.dart';
import 'package:solana/encoder.dart';
import 'package:solana/solana.dart';

class PassbookProgram {
  static const prefix = 'passbook';
  static const programId = 'passjvPvHQWN4SvBCmHk1gdrtBvoHRERtQK9MKemreQ';

  static Future<String> findProgramAuthority() {
    return findProgramAddress(seeds: [
      Buffer.fromBase58(prefix),
      Buffer.fromBase58(PassbookProgram.programId),
    ], programId: programId);
  }

  static Future<String> findPassStoreAccount(String authority) {
    return findProgramAddress(seeds: [
      Buffer.fromBase58(prefix),
      Buffer.fromBase58(PassbookProgram.programId),
      Buffer.fromBase58(authority),
      Buffer.fromBase58(Store.prefix),
    ], programId: programId);
  }

  static Future<String> findPassBookAccount(String mint) {
    return findProgramAddress(seeds: [
      Buffer.fromBase58(prefix),
      Buffer.fromBase58(PassbookProgram.programId),
      Buffer.fromBase58(mint),
    ], programId: programId);
  }

  static Future<String> findPassAccount(String mint) {
    return findProgramAddress(seeds: [
      Buffer.fromBase58(prefix),
      Buffer.fromBase58(PassbookProgram.programId),
      Buffer.fromBase58(mint),
    ], programId: programId);
  }
}
