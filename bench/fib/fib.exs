defmodule F do
  def fib(k) when k < 2, do: k
  def fib(k), do: fib(k - 1) + fib(k - 2)
end
n = case Integer.parse(System.get_env("N") || "") do
  {v, ""} when v >= 0 and v <= 92 -> v
  _ -> System.halt(2)
end
System.halt(rem(F.fib(n), 256))
