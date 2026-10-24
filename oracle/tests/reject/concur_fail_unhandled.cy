fn f [] -> i32 or fail[bad] { fail bad; }
MAIN { concur { let $v := f() otherwise fail; }; }
