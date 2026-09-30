local n = tonumber(os.getenv("N") or "")
if not n or n < 0 or n > 100000000 then os.exit(2) end
local xs, len = {}, 0
for i = 1, n do len = len + 1; xs[len] = i % 1000 end
local acc = 0
for _ = 1, 10 do for j = 1, len do acc = acc + xs[j] end end
os.exit(acc % 256)
