n <- suppressWarnings(as.numeric(Sys.getenv("N")))
if (is.na(n) || n < 0 || n > 1e8) quit(status = 2)
xs <- numeric(n)
for (i in seq_len(n)) xs[i] <- i %% 1000
acc <- 0
for (r in 1:10) for (j in seq_len(n)) acc <- acc + xs[j]
quit(status = acc %% 256)
