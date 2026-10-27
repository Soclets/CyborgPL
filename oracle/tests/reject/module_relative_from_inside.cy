
module a:
    public function f []:
        g();
    end of function f
end of module a
function g []:
end of function g
!MAIN:
    a.f();
!end of MAIN
