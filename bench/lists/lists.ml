let () =
  let n = match Sys.getenv_opt "N" with Some s -> (try int_of_string s with _ -> -1) | None -> -1 in
  if n < 0 || n > 100_000_000 then exit 2;
  let xs = ref (Array.make 4 0) and len = ref 0 in
  for i = 1 to n do
    if !len = Array.length !xs then begin
      let bigger = Array.make (2 * !len) 0 in
      Array.blit !xs 0 bigger 0 !len; xs := bigger
    end;
    (!xs).(!len) <- i mod 1000; incr len
  done;
  let acc = ref 0 in
  for _ = 1 to 10 do for j = 0 to !len - 1 do acc := !acc + (!xs).(j) done done;
  exit (!acc mod 256)
