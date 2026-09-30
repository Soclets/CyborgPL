let () =
  let n = match Sys.getenv_opt "N" with Some s -> (try int_of_string s with _ -> -1) | None -> -1 in
  if n < 0 || n > 1_000_000_000 then exit 2;
  let acc = ref 0 in
  for i = 1 to n do acc := !acc + (i * i) mod 7 done;
  exit (!acc mod 1_000_000_007 mod 256)
