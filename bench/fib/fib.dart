import 'dart:io';
int fib(int n) => n < 2 ? n : fib(n - 1) + fib(n - 2);
void main() {
  final n = int.tryParse(Platform.environment['N'] ?? '') ?? -1;
  if (n < 0 || n > 92) exit(2);
  exit(fib(n) % 256);
}
