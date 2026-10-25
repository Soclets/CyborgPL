# A path reads as deep as it goes (2026-10-25): a field of a field, a field of a list item, an item of a field's
# list. A plain value at the end is read bare; an owned one takes `copy` or `lend`, as one step does.
struct Stop { name: str.utf8, minutes: i32 }
struct Trip { stops: std::list[Stop], km: i32 }
struct Booking { who: str.utf8, trip: Trip }

fn shout [lent $s: str.utf8] -> str.utf8 { return std::str.upper.full(lend $s); }

MAIN {
    let $b := Booking[who = "Ana", trip = Trip[stops = [Stop[name = "Lisboa", minutes = 0], Stop[name = "Madrid", minutes = 540]], km = 625]];
    let $all: std::list[Booking] = [copy $b,];
    std::print.useful($b.trip.km, " km, ", $b.trip.stops[pos: 2].minutes, " minutes, ", $all[pos: 1].trip.km, " km again", newln);
    let $first := copy $b.trip.stops[pos: 1].name;
    std::print.useful($first, " to ", shout(lend $all[pos: 1].trip.stops[pos: 2].name), ", ", std::list.length(lend $b.trip.stops), " stops", newln);
}
