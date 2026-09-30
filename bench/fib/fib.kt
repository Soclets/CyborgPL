import kotlin.system.exitProcess
fun fib(n: Long): Long = if (n < 2) n else fib(n - 1) + fib(n - 2)
fun main() {
    val n = System.getenv("N")?.toLongOrNull() ?: -1L
    if (n < 0 || n > 92L) exitProcess(2)
    exitProcess((fib(n) % 256).toInt())
}
