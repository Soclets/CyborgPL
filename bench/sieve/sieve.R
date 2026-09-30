n <- suppressWarnings(as.numeric(Sys.getenv("N")))
if (is.na(n) || n < 0 || n > 2e8) quit(status = 2)
comp <- logical(n + 1)
count <- 0
for (p in 2:n) {
  if (!comp[p + 1]) {
    count <- count + 1
    m <- as.numeric(p) * p
    while (m <= n) { comp[m + 1] <- TRUE; m <- m + p }
  }
}
quit(status = count %% 256)
