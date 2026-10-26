# C's memory through a pointer (ffi.md): ptr.write, ptr.read, and ptr.at
# moving a whole number of elements. 72 + 105 - 170 = 7.
%extern.C fn exit [$code: i32];
%extern.C fn malloc [$bytes: u64] -> ptr.mut[u8];
%extern.C fn free [$p: ptr.mut[u8]];
MAIN {
    no_check let $buf := malloc(4);
    no_check ptr.write($buf, value: 72);
    no_check ptr.write(ptr.at($buf, by: 1), value: 105);
    no_check let $a := ptr.read($buf);
    no_check let $b := ptr.read(ptr.at($buf, by: 1));
    no_check free($buf);
    no_check my.exit(to.i32($a) plus to.i32($b) minus 170);
}
