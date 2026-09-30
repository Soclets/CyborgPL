n = case Integer.parse(System.get_env("N") || "") do
  {v, ""} when v >= 0 and v <= 1_000_000_000 -> v
  _ -> System.halt(2)
end
acc = Enum.reduce(1..n//1, 0, fn i, a -> rem(a + rem(i * i, 7), 1_000_000_007) end)
System.halt(rem(acc, 256))
