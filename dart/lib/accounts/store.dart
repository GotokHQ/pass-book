import 'dart:typed_data';

import 'package:passbook/accounts/constants.dart';
import 'package:passbook/utils/endian.dart';
import 'package:passbook/utils/struct_reader.dart';
import 'package:solana/base58.dart';

class Store {
  const Store({
    required this.key,
    required this.authority,
    required this.redemptionsCount,
    required this.passCount,
    required this.passBookCount,
  });

  static const prefix = 'store';

  factory Store.fromBinary(List<int> sourceBytes) {
    final bytes = Int8List.fromList(sourceBytes);
    final reader = StructReader(bytes.buffer)..skip(1);
    final authority = base58encode(reader.nextBytes(32));
    final redemptionsCount = decodeBigInt(reader.nextBytes(8), Endian.little);
    final passCount = decodeBigInt(reader.nextBytes(8), Endian.little);
    final passBookCount = decodeBigInt(reader.nextBytes(8), Endian.little);
    return Store(
        key: AccountKey.passStore,
        authority: authority,
        redemptionsCount: redemptionsCount,
        passCount: passCount,
        passBookCount: passBookCount);
  }

  final AccountKey key;
  final String authority;
  final BigInt redemptionsCount;
  final BigInt passCount;
  final BigInt passBookCount;
}
