import 'dart:io';
import 'dart:typed_data';
void main() {
  final n = int.tryParse(Platform.environment['N'] ?? '') ?? -1;
  if (n < 0 || n > 200000000) exit(2);
  final comp = Uint8List(n + 1);
  var count = 0;
  for (var p = 2; p <= n; p++) {
    if (comp[p] == 0) { count++; for (var m = p * p; m <= n; m += p) comp[m] = 1; }
  }
  exit(count % 256);
}
