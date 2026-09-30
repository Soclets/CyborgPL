n <- suppressWarnings(as.numeric(Sys.getenv("N")))
if (is.na(n) || n < 0 || n > 1e9) quit(status = 2)
acc <- 0
for (i in seq_len(n)) { r <- i %% 7; acc <- acc + (r * r) %% 7 }
quit(status = acc %% 1000000007 %% 256)
