n = Integer(ENV["N"] || "", exception: false)
exit(2) if n.nil? || n < 0 || n > 200_000_000
comp = "\0".b * (n + 1)
count = 0
p = 2
while p <= n
  if comp.getbyte(p) == 0
    count += 1
    m = p * p
    while m <= n
      comp.setbyte(m, 1)
      m += p
    end
  end
  p += 1
end
exit(count % 256)
