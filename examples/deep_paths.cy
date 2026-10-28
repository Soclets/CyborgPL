# A path reads as deep as it goes (2026-10-25): a field of a field, a field of a list item, an item of a field's
# list. A plain value at the end is read bare; an owned one takes `copy` or `lend`, as one step does.
rec Stop:
    var name: str.utf8;
    var minutes: i32;
;rec Stop
rec Trip:
    var stops: std::ls[Stop];
    var km: i32;
;rec Trip
rec Booking:
    var who: str.utf8;
    var trip: Trip;
;rec Booking

fn shout [lent $s: str.utf8] -> str.utf8:
    ret std::str.upper.full(lend $s);
;fn shout

MAIN:
    let $b := Booking[who = "Ana", trip = Trip[stops = [Stop[name = "Lisboa", minutes = 0], Stop[name = "Madrid", minutes = 540]], km = 625]];
    let $all: std::ls[Booking] := [copy $b,];
    std::print($b.trip.km, " km, ", $b.trip.stops[pos: 2].minutes, " minutes, ", $all[pos: 1].trip.km, " km again", newln);
    let $first := copy $b.trip.stops[pos: 1].name;
    std::print($first, " to ", shout(lend $all[pos: 1].trip.stops[pos: 2].name), ", ", std::ls.len(lend $b.trip.stops), " stops", newln);
;MAIN
