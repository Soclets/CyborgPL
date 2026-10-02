-- lists, the proper LuaJIT way: a C array of known size from LuaJIT's built-in FFI,
-- rather than a Lua table, as LuaJIT's own docs advise for large arrays of numbers.
local ffi = require("ffi")
local n = tonumber(os.getenv("N") or "")
if not n or n < 0 or n > 100000000 then os.exit(2) end
local xs = ffi.new("int32_t[?]", n)
for i = 0, n - 1 do xs[i] = (i + 1) % 1000 end
local acc = 0
for _ = 1, 10 do
  for j = 0, n - 1 do acc = acc + xs[j] end
end
os.exit(acc % 256)
