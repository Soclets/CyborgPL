n = case Integer.parse(System.get_env("N") || "") do
  {v, ""} when v >= 0 and v <= 1_000_000_000 -> v
  _ -> System.halt(2)
end
acc = Enum.reduce(1..n//1, 0, fn i, a -> a + rem(i * i, 7) end)
System.halt(rem(rem(acc, 1_000_000_007), 256))
