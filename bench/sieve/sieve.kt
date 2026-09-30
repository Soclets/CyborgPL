import kotlin.system.exitProcess
fun main() {
    val n = System.getenv("N")?.toLongOrNull() ?: -1L
    if (n < 0 || n > 200_000_000L) exitProcess(2)
    val un = n.toInt()
    val comp = BooleanArray(un + 1)
    var count = 0L
    for (p in 2..un) if (!comp[p]) { count++; var m = p.toLong() * p; while (m <= n) { comp[m.toInt()] = true; m += p } }
    exitProcess((count % 256).toInt())
}
