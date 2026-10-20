# Names in every script, and emoji, come from the limb WSTN, "Wanted Some Thai Names" (owner,
# 2026-10-06: "`a`, `WSTN` (Wanted Some Thai Names) is the name", and "`b`": every script and emoji).
# the base's own names are English only; this kit lists WSTN in its `limbs`.
fn ทักทาย ['ชื่อ': str.utf8] -> str.utf8 {
    return "สวัสดี " concat 'ชื่อ';
}

MAIN {
    let '🍕': i32 = 3;
    let 'नमस्ते' := ทักทาย["โลก"];
    std::print.useful['नमस्ते', " ", '🍕', newln];
}
