# A shift's width comes from any operand whose type is known — a name, an expression,
# or the type it is being stored into (decided 2026-09-23). Same for not.bit and the wrapping forms.
MAIN {
    let 'v': uint32 = 3;
    let 'a': uint32 = 2 shl 3;
    let 'b': uint32 = ('v' mod.floor 16) shl 2;
    let 'c': uint32 = not.bit ('v' + 1);
    let 'd': uint32 = ('v' + 1) plus.wrap 5;
    std::print.useful['a', " ", 'b', " ", 'c', " ", 'd', newln];
}
