import 'dart:convert';
import 'dart:typed_data';

class StructReader {
  StructReader(this._buffer) : _offset = 0;

  void skip(int length) => _offset += length;

  String nextString() {
    final length = _buffer.asByteData(_offset, 4).getInt32(0, Endian.little);
    final rawBytes = _buffer.asUint8List(_offset + 4, length);

    _offset += length + 4;
    // It is a zero terminated string a'la C
    final lastZero = rawBytes.indexOf(0);
    if (lastZero == -1) {
      return utf8.decode(rawBytes);
    }

    return utf8.decode(rawBytes.sublist(0, lastZero));
  }

  Uint8List nextBytes(int length) {
    final bytes = _buffer.asUint8List(_offset, length);
    _offset += length;

    return bytes;
  }

  final ByteBuffer _buffer;
  int _offset;
}
