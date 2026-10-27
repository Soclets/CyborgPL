# module `shapes`; `shapes/circle.cy` and `shapes/square.cy` nest under it.
public function describe [] -> string.utf8:
    return my.helper() concatenate ", " concatenate circle.name() concatenate ", " concatenate square.name();
end of function describe
