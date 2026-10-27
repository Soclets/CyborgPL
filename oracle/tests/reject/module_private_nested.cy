
module a:
    module b:
        public function f []:
        end of function f
    end of module b
end of module a
!MAIN:
    a.b.f();
!end of MAIN
