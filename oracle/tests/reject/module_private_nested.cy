
module a:
    module b:
        pub fn f []:
        end of fn f
    end of module b
end of module a
!MAIN:
    a.b.f();
!end of MAIN
