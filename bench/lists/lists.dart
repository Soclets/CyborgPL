import 'dart:io';
void main() {
  final n = int.tryParse(Platform.environment['N'] ?? '') ?? -1;
  if (n < 0 || n > 100000000) exit(2);
  final xs = <int>[];
  for (var i = 1; i <= n; i++) xs.add(i % 1000);
  var acc = 0;
  for (var r = 0; r < 10; r++) { for (var j = 0; j < xs.length; j++) acc += xs[j]; }
  exit(acc % 256);
}
