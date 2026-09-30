n = Integer(ENV["N"] || "", exception: false)
exit(2) if n.nil? || n < 0 || n > 1_000_000_000
acc = 0
i = 1
while i <= n
  acc = (acc + (i * i) % 7) % 1_000_000_007
  i += 1
end
exit(acc % 256)
