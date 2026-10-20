struct V {
    n: i32
}

pub bare.never op <+> ['a': V, 'b': V] -> V {
    return V[n = 0];
}

MAIN { }
