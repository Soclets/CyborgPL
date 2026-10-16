# module `shapes`; `shapes/circle.cy` and `shapes/square.cy` nest under it.
pub fn describe [] -> str.utf8 {
    return my.helper[] concat ", " concat circle.name[] concat ", " concat square.name[];
}
