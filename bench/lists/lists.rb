n = Integer(ENV["N"] || "", exception: false)
exit(2) if n.nil? || n < 0 || n > 100_000_000
xs = []
i = 1
while i <= n
  xs << i % 1000
  i += 1
end
acc = 0
10.times { j = 0; while j < xs.size; acc += xs[j]; j += 1; end }
exit(acc % 256)
