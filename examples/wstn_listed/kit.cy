# Names in every script, and emoji, come from the limb WSTN, "Wanted Some Thai Names" (owner,
# 2026-10-06: "`a`, `WSTN` (Wanted Some Thai Names) is the name", and "`b`": every script and emoji).
# the base's own names are English only; this kit lists WSTN in its `limbs`.
fn ทักทาย [let $'ชื่อ': str.utf8] -> str.utf8:
    ret "สวัสดี " concat $'ชื่อ';
end of fn ทักทาย

!MAIN:
    let $'🍕': int.sign.32 = 3;
    let $'नमस्ते' := ทักทาย("โลก");
    std::print.shortest($'नमस्ते', " ", $'🍕', newln);
!end of MAIN
