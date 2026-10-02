-- sum, the proper LuaJIT way: one plain numeric for loop, which the tracing JIT compiles to
-- machine code. LuaJIT comes with no threads, so it runs on one core. Numbers are doubles,
-- exact only to 2^53, so i is reduced mod 7 before squaring ((i*i) mod 7 = (r*r) mod 7).
local n = tonumber(os.getenv("N") or "")
if not n or n < 0 or n > 1000000000 then os.exit(2) end
local acc = 0
for i = 1, n do
  local r = i % 7
  acc = acc + (r * r) % 7
end
os.exit(acc % 1000000007 % 256)
