import 'dart:typed_data';

import 'package:passbook/accounts/constants.dart';
import 'package:passbook/utils/endian.dart';
import 'package:passbook/utils/struct_reader.dart';
import 'package:solana/base58.dart';

class PassBook {
  const PassBook(
      {required this.key,
      required this.name,
      required this.description,
      required this.uri,
      required this.authority,
      required this.mint,
      required this.mutable,
      required this.passState,
      required this.durationType,
      required this.duration,
      required this.totalPasses,
      this.maxSupply});

  factory PassBook.fromBinary(List<int> sourceBytes) {
    final bytes = Int8List.fromList(sourceBytes);
    final reader = StructReader(bytes.buffer)..skip(1);
    final authority = base58encode(reader.nextBytes(32));
    final mint = base58encode(reader.nextBytes(32));
    final name = reader.nextString();
    final description = reader.nextString();
    final uri = reader.nextString();
    final mutable = reader.nextBytes(1).first == 1;
    final passState = PassStateExtension.fromId(reader.nextBytes(1).first);
    final durationType =
        DurationTypeExtension.fromId(reader.nextBytes(1).first);
    final BigInt duration = decodeBigInt(reader.nextBytes(8), Endian.little);
    final BigInt totalPasses = decodeBigInt(reader.nextBytes(8), Endian.little);
    final hasMaxSupply = reader.nextBytes(1).first == 1;
    final BigInt? maxSupply =
        hasMaxSupply ? decodeBigInt(reader.nextBytes(8), Endian.little) : null;
    return PassBook(
        key: AccountKey.passBook,
        name: name,
        description: description,
        uri: uri,
        authority: authority,
        mint: mint,
        mutable: mutable,
        passState: passState,
        durationType: durationType,
        duration: duration,
        totalPasses: totalPasses,
        maxSupply: maxSupply);
  }

  final AccountKey key;
  final String name;
  final String description;
  final String uri;
  final String authority;
  final String mint;
  final bool mutable;
  final PassState passState;
  final DurationType durationType;
  final BigInt duration;
  final BigInt totalPasses;
  final BigInt? maxSupply;
}
