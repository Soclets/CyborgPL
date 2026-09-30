local n = tonumber(os.getenv("N") or "")
if not n or n < 0 or n > 1000000000 then os.exit(2) end
local acc = 0
for i = 1, n do local r = i % 7; acc = acc + (r * r) % 7 end
os.exit(acc % 1000000007 % 256)
