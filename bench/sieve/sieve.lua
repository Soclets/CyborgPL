local n = tonumber(os.getenv("N") or "")
if not n or n < 0 or n > 200000000 then os.exit(2) end
local comp = {}
local count = 0
for p = 2, n do
  if not comp[p] then
    count = count + 1
    local m = p * p
    while m <= n do comp[m] = true; m = m + p end
  end
end
os.exit(count % 256)
