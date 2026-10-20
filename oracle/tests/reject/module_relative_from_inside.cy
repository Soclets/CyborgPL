bare.kit_prefix.never [fn];

module a { pub fn f [] { g[]; } }
fn g [] { }
MAIN {
    a.f[];
}
