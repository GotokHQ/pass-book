import 'dart:typed_data';

List<int> _convertBytesEndianType(Iterable<int> bytes, int k,
    [Endian endianType = Endian.big]) {
  switch (endianType) {
    case Endian.little:
      var ret = List<int>.from(bytes).reversed.toList();
      ret.addAll(List<int>.filled(k - bytes.length, 0));
      return ret;
    default:
      var ret = List<int>.filled(k - bytes.length, 0, growable: true);
      ret.addAll(bytes);
      return ret;
  }
}

/// Decode a BigInt from bytes
BigInt decodeBigInt(List<int> input, [Endian endianType = Endian.big]) {
  final bytes = _convertBytesEndianType(input, input.length, endianType);
  BigInt result = BigInt.from(0);
  for (int i = 0; i < bytes.length; i++) {
    result += BigInt.from(bytes[bytes.length - i - 1]) << (8 * i);
  }
  return result;
}

var _byteMask = BigInt.from(0xff);

/// Encode a BigInt into bytes using big-endian encoding.
Uint8List encodeBigInt(BigInt number) {
  // Not handling negative numbers. Decide how you want to do that.
  int size = (number.bitLength + 7) >> 3;
  var result = Uint8List(size);
  for (int i = 0; i < size; i++) {
    result[size - i - 1] = (number & _byteMask).toInt();
    number = number >> 8;
  }
  return result;
}
