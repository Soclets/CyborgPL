let rec fib n = if n < 2 then n else fib (n - 1) + fib (n - 2)
let () =
  let n = match Sys.getenv_opt "N" with Some s -> (try int_of_string s with _ -> -1) | None -> -1 in
  if n < 0 || n > 92 then exit 2;
  exit (fib n mod 256)
