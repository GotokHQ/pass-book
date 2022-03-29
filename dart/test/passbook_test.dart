import 'package:flutter_test/flutter_test.dart';
import 'package:passbook/accounts/constants.dart';

import 'package:passbook/accounts/passbook.dart';

import 'fixtures/passbook.dart';

void main() {
  test(
    'correctly decodes passbook',
    () {
      final metadata = PassBook.fromBinary(passBookData);
      expect(
        metadata,
        isA<PassBook>()
            .having((m) => m.name, 'name', 'my pass book')
            .having((m) => m.description, 'description', 'test')
            .having(
              (m) => m.passState,
              'passState',
              PassState.notActivated,
            )
            .having(
              (m) => m.mint,
              'mint',
              'Buv9H4c9stsuUvzRWhYVSpT2z2hstF58kDXcUmV6nEhv',
            )
            .having(
              (m) => m.authority,
              'authority',
              '2ENfXBnrwciVC5sbRd3mTZKQvXBa1kHj5boPXajwStSJ',
            )
            .having(
              (m) => m.uri,
              'uri',
              'uri',
            )
            .having(
              (m) => m.durationType,
              'durationType',
              DurationType.days,
            )
            .having(
              (m) => m.duration,
              'duration',
              BigInt.from(30),
            )
            .having(
              (m) => m.totalPasses,
              'totalPasses',
              BigInt.from(0),
            )
            .having(
              (m) => m.maxSupply,
              'maxSupply',
              BigInt.from(100),
            ),
      );
    },
  );
}
