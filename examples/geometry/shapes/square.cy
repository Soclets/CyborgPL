pub fn area [let $s: float.bin.64] -> float.bin.64:
    ret $side times $s times $s;
end of fn area
pub.kit fn name [] -> str.utf8:
    ret "square";
end of fn name
const $side := 1.0;
