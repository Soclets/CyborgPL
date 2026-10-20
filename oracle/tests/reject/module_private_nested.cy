bare.kit_prefix.never [fn];

module a { module b { pub fn f [] { } } }
MAIN {
    a.b.f[];
}
