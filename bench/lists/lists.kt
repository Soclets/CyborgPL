import kotlin.system.exitProcess
fun main() {
    val n = System.getenv("N")?.toLongOrNull() ?: -1L
    if (n < 0 || n > 100_000_000L) exitProcess(2)
    val xs = ArrayList<Long>()
    var i = 1L
    while (i <= n) { xs.add(i % 1000); i++ }
    var acc = 0L
    repeat(10) { for (j in 0 until xs.size) acc += xs[j] }
    exitProcess((acc % 256).toInt())
}
