# Names in every script, and emoji, come from the limb WSTN, "Wanted Some Thai Names" (owner,
# 2026-10-06: "`a`, `WSTN` (Wanted Some Thai Names) is the name", and "`b`": every script and emoji).
# the base's own names are English only; this kit lists WSTN in its `limbs`.
function ทักทาย [$'ชื่อ': string.utf8] -> string.utf8:
    return "สวัสดี " concatenate $'ชื่อ';
end of function ทักทาย

!MAIN:
    let $'🍕': integer.signed.32_bit = 3;
    let $'नमस्ते' := ทักทาย("โลก");
    std::print.shortest($'नमस्ते', " ", $'🍕', newline);
!end of MAIN
