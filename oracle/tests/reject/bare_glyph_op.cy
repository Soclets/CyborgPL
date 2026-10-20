struct V {
    n: i32
}

pub bare.kit_prefix.never op <+> ['a': V, 'b': V] -> V {
    return V[n = 0];
}

MAIN { }
