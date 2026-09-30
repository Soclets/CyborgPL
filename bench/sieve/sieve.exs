n = case Integer.parse(System.get_env("N") || "") do
  {v, ""} when v >= 0 and v <= 200_000_000 -> v
  _ -> System.halt(2)
end
comp = :atomics.new(n + 1, signed: false)
count = Enum.reduce(2..n//1, 0, fn p, c ->
  if :atomics.get(comp, p + 1) == 0 do
    Stream.iterate(p * p, &(&1 + p)) |> Stream.take_while(&(&1 <= n)) |> Enum.each(&:atomics.put(comp, &1 + 1, 1))
    c + 1
  else
    c
  end
end)
System.halt(rem(count, 256))
