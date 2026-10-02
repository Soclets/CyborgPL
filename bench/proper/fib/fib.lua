-- fib, the proper LuaJIT way: a local recursive function (naive recursion is the program).
local n = tonumber(os.getenv("N") or "")
if not n or n < 0 or n > 92 then os.exit(2) end
local function fib(k)
  if k < 2 then return k end
  return fib(k - 1) + fib(k - 2)
end
os.exit(fib(n) % 256)
