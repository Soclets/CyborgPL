-- sieve, the proper LuaJIT way: one zeroed byte per number from LuaJIT's built-in FFI,
-- crossing off from p*p.
local ffi = require("ffi")
local n = tonumber(os.getenv("N") or "")
if not n or n < 0 or n > 200000000 then os.exit(2) end
local comp = ffi.new("uint8_t[?]", n + 1)
local p = 2
while p * p <= n do
  if comp[p] == 0 then
    for m = p * p, n, p do comp[m] = 1 end
  end
  p = p + 1
end
local count = 0
for i = 2, n do if comp[i] == 0 then count = count + 1 end end
os.exit(count % 256)
