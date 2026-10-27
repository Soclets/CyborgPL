# A path reads as deep as it goes (2026-10-25): a field of a field, a field of a list item, an item of a field's
# list. A plain value at the end is read bare; an owned one takes `copy` or `lend`, as one step does.
record Stop:
    name: string.utf8; minutes: integer.signed.32_bit;
end of record Stop
record Trip:
    stops: std::list[Stop]; km: integer.signed.32_bit;
end of record Trip
record Booking:
    who: string.utf8; trip: Trip;
end of record Booking

function shout [lent $s: string.utf8] -> string.utf8:
    return std::string.uppercase.full(lend $s);
end of function shout

!MAIN:
    let $b := Booking[who = "Ana", trip = Trip[stops = [Stop[name = "Lisboa", minutes = 0], Stop[name = "Madrid", minutes = 540]], km = 625]];
    let $all: std::list[Booking] = [copy $b,];
    std::print.shortest($b.trip.km, " km, ", $b.trip.stops[position: 2].minutes, " minutes, ", $all[position: 1].trip.km, " km again", newline);
    let $first := copy $b.trip.stops[position: 1].name;
    std::print.shortest($first, " to ", shout(lend $all[position: 1].trip.stops[position: 2].name), ", ", std::list.length(lend $b.trip.stops), " stops", newline);
!end of MAIN
