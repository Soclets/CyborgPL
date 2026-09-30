n = Integer(ENV["N"] || "", exception: false)
exit(2) if n.nil? || n < 0 || n > 1_000_000_000
acc = 0
i = 1
while i <= n
  acc += (i * i) % 7
  i += 1
end
exit(acc % 1_000_000_007 % 256)
