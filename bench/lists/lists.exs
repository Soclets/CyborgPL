n = case Integer.parse(System.get_env("N") || "") do
  {v, ""} when v >= 0 and v <= 100_000_000 -> v
  _ -> System.halt(2)
end
xs = :array.from_list(Enum.map(1..n//1, fn i -> rem(i, 1000) end))
len = :array.size(xs)
acc = Enum.reduce(1..10, 0, fn _, a -> Enum.reduce(0..(len - 1)//1, a, fn j, b -> b + :array.get(j, xs) end) end)
System.halt(rem(acc, 256))
