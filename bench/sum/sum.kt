import kotlin.system.exitProcess
fun main() {
    val n = System.getenv("N")?.toLongOrNull() ?: -1L
    if (n < 0 || n > 1_000_000_000L) exitProcess(2)
    var acc = 0L
    var i = 1L
    while (i <= n) { acc = (acc + (i * i) % 7) % 1_000_000_007L; i++ }
    exitProcess((acc % 256).toInt())
}
