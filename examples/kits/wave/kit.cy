# Another kit with a `bare.kit_prefix.optional` `hello`, so a bare `hello` beside `greet` is ambiguous.

pub bare.kit_prefix.optional fn hello [] -> str.utf8 {
    return "hi";
}
