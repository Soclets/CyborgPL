import 'dart:io';
void main() {
  final n = int.tryParse(Platform.environment['N'] ?? '') ?? -1;
  if (n < 0 || n > 1000000000) exit(2);
  var acc = 0;
  for (var i = 1; i <= n; i++) acc = (acc + (i * i) % 7) % 1000000007;
  exit(acc % 256);
}
