
module a:
    pub fn f []:
        g();
    end of fn f
end of module a
fn g []:
end of fn g
!MAIN:
    a.f();
!end of MAIN
