let () =
  let n = match Sys.getenv_opt "N" with Some s -> (try int_of_string s with _ -> -1) | None -> -1 in
  if n < 0 || n > 200_000_000 then exit 2;
  let comp = Bytes.make (n + 1) '\000' in
  let count = ref 0 in
  for p = 2 to n do
    if Bytes.get comp p = '\000' then begin
      incr count;
      let m = ref (p * p) in
      while !m <= n do Bytes.set comp !m '\001'; m := !m + p done
    end
  done;
  exit (!count mod 256)
