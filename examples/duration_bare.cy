# `duration` bare is std's `std::duration`: std marks everything `bare.kit_prefix.optional` (owner, 2026-10-06:
# "optional for all"). Without std, the type and its literals are refused.
fn twice ['d': duration] -> duration {
    return 'd' + 'd';
}
MAIN {
    let 'w': duration = 2 seconds;
    std::print.useful[twice['w'] equal_to 4 seconds, newln];
}
