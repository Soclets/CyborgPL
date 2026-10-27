# Night trains across Europe: a longer CyborgPL program, written to be read.
#
# It reads a timetable written as text, finds the quickest way between two stations (Dijkstra's
# algorithm, by hand), books travellers onto it, prices each ticket in exact decimals (`d64`, so
# money never picks up a binary float's error), turns minutes into arrival times, prints a
# departure board and ends with a report. The network is made up; the arithmetic is not, and every
# figure it prints can be checked by hand from the timetable below. Along the way it shows:
#
#   structs, enums with data, `match` with `is`     ownership said aloud: `give`, `copy`, `lend`
#   failures a caller must handle, `otherwise`      a handler, told whom it is apologising to
#   range types, `i32[min: 1, max: 4]`              generics with an interface, `Number`
#   maps, looked up with a lent key                 a type's own iterator, `Clock.next`
#   counting loops, `loop`, `break`, `continue`     a short lambda, `fn [$s] with [lend $part] = …`
#   tasks and channels, `concur`                    a writer, `std::print.writer`
#   format holes in words, widths in `cols`         rounding money on purpose, `std::round.near`
#   named bounds, `coll.bound trip`                 guards that let a final build prove every site

# ---------------------------------------------------------------------------------------------------
# The fixed facts
# ---------------------------------------------------------------------------------------------------

constant $network_name := "Nightjar";
constant $departure: integer.signed.32_bit = 1290;        # every train leaves at 21:30, counted in minutes from midnight
constant $change_minutes: integer.signed.32_bit = 10;     # changing trains at a station costs ten minutes
constant $near_rate: float.%decimal.64_bit = 0.12;        # the fare per km for the first 500 km
constant $far_rate: float.%decimal.64_bit = 0.08;         # and per km after that
constant $near_km: integer.signed.32_bit = 500;
constant $by_eight: integer.signed.32_bit = 1920;
constant $most_bookings: integer.unsigned.64_bit = 500;     # a night's trains hold no more travellers than this         # 08:00 the next morning, 1440 + 480

# ---------------------------------------------------------------------------------------------------
# Types
# ---------------------------------------------------------------------------------------------------

# A stretch of line between two stations, run in both directions.
record Link:
    a: string.utf8;
    b: string.utf8;
    minutes: integer.signed.32_bit[minimum: 1, maximum: 1440];
    km: integer.signed.32_bit[minimum: 1, maximum: 3000];
end of record Link

record Network:
    stations: std::list[string.utf8];
    links: std::list[Link];
end of record Network

# The longest trip the network sells: ten days and 100,000 km. A way past either is no way, and that limit is what
# keeps every sum of minutes and km below provably inside an `i32`.
collection.bounded trip:
    trip.minutes = integer.signed.32_bit[minimum: 0, maximum: 14400];
    trip.km = integer.signed.32_bit[minimum: 0, maximum: 100000];
end of collection.bounded trip

# The quickest known way to a station: when it is reached, how far it is, and where it came from.
record Reach:
    minutes: trip.minutes;
    km: trip.km;
    prev: string.utf8;
end of record Reach

record Journey:
    stops: std::list[string.utf8];
    minutes: trip.minutes;
    km: trip.km;
end of record Journey

# Where a traveller sleeps. A sleeper carries how many berths share the compartment.
union.tagged Seat:
    seat;
    couchette;
    sleeper: integer.signed.32_bit;
end of union.tagged Seat

union.tagged Age:
    adult;
    youth;
    senior;
end of union.tagged Age

record Request:
    traveller: string.utf8;
    origin: string.utf8;
    destination: string.utf8;
    seat: Seat;
    age: Age;
end of record Request

record Booking:
    ticket: integer.signed.32_bit;
    traveller: string.utf8;
    journey: Journey;
    seat: Seat;
    fare: float.%decimal.64_bit;
end of record Booking

# ---------------------------------------------------------------------------------------------------
# Small generic helpers
# ---------------------------------------------------------------------------------------------------

interface Number:
    integer.signed.32_bit;
    float.%decimal.64_bit;
end of interface Number

function total_of <T: Number> [lent $xs: std::list[<T>]] -> <T>:
    variable $sum: <T> = 0;
    for $x in lend $xs:
        change $sum = $sum plus $x;
    end of for
    return $sum;
end of function total_of

# ---------------------------------------------------------------------------------------------------
# Reading the timetable: one stretch of line per row, "Lisboa - Madrid 540 625", minutes then km
# ---------------------------------------------------------------------------------------------------

function timetable [] -> std::list[string.utf8]:
    return [
        "Lisboa - Madrid 540 625",
        "Madrid - Barcelona 170 621",
        "Barcelona - Paris 395 1035",
        "Paris - Bruxelles 85 312",
        "Bruxelles - Amsterdam 110 210",
        "Bruxelles - Köln 110 220",
        "Amsterdam - Köln 160 260",
        "Paris - Zürich 245 610",
        "Köln - Berlin 260 580",
        "Köln - München 270 590",
        "Zürich - München 230 320",
        "Zürich - Wien 470 750",
        "München - Wien 240 435",
        "Wien - Praha 240 330",
        "Berlin - Praha 260 350",
        "Wien - Kraków 400 440",
        "Praha - Kraków 450 540",
        "Berlin - København 420 440",
        "København - Malmö 40 45",
        "Malmö - Stockholm 270 615",
        "Wien - Budapest two hours",
        "Palermo - Messina 90 230"
    ];
end of function timetable

# "Lisboa - Madrid 540 625" into a link. The two numbers come last, so a station's name may hold spaces.
function parse_link [lent $row: string.utf8] -> Link or fail[bad_row: string.utf8]:
    let $sides := std::string.split(lend $row, at: " - ");
    if std::list.length(lend $sides) not_equal_to 2:
        fail bad_row[copy $row];
    end of if
    let $words := std::string.split(lend $sides[position: 2], at: " ");
    let $n := std::list.length(lend $words);
    if $n less_than 3:
        fail bad_row[copy $row];
    end of if
    let $minutes: integer.signed.32_bit[minimum: 1, maximum: 1440] = to.integer.signed.32_bit[minimum: 1, maximum: 1440]($words[position: $n minus 1]) otherwise:
        case not_a_number:
            fail bad_row[copy $row];
        end of case not_a_number
        case out_of_range:
            fail bad_row[copy $row];
        end of case out_of_range
    end of otherwise;
    let $km: integer.signed.32_bit[minimum: 1, maximum: 3000] = to.integer.signed.32_bit[minimum: 1, maximum: 3000]($words[position: $n]) otherwise:
        case not_a_number:
            fail bad_row[copy $row];
        end of case not_a_number
        case out_of_range:
            fail bad_row[copy $row];
        end of case out_of_range
    end of otherwise;
    let $name_words := std::list.slice(lend $words, from: position 1, before: position $n minus 1);
    let $b := std::string.join(lend $name_words, between: " ");
    return Link[a = copy $sides[position: 1], b = give $b, minutes = $minutes, km = $km];
end of function parse_link

procedure add_station [lent.mutable $stations: std::list[string.utf8], lent $name: string.utf8]:
    for $s in lend $stations:
        if $s equal_to $name:
            return;
        end of if
    end of for
    std::list.append(lend.mutable $stations, value: copy $name);
end of procedure add_station

procedure read_network [lent $rows: std::list[string.utf8]] -> Network:
    variable $stations: std::list[string.utf8] = std::list.empty;
    variable $links: std::list[Link] = std::list.empty;
    for $row in lend $rows:
        let $link := parse_link(lend $row) otherwise:
            !case bad_row $what:
                std::print.shortest("  (skipped a row that does not read as \"A - B minutes km\": \"", $what, "\")", newline);
                continue;
            !end of case bad_row
        end of otherwise;
        add_station(lend.mutable $stations, lend $link.a);
        add_station(lend.mutable $stations, lend $link.b);
        std::list.append(lend.mutable $links, value: give $link);
    end of for
    return Network[stations = give $stations, links = give $links];
end of procedure read_network

function knows [lent $net: Network, lent $name: string.utf8] -> boolean:
    for $s in lend $net.stations:
        if $s equal_to $name:
            return true;
        end of if
    end of for
    return false;
end of function knows

# ---------------------------------------------------------------------------------------------------
# The quickest way: Dijkstra's algorithm. Each round finishes the unfinished station reached soonest,
# then tries every line out of it. Leaving a station the traveller arrived at by another train costs
# a change; leaving the first station does not.
# ---------------------------------------------------------------------------------------------------

procedure quickest_from [lent $net: Network, lent $from: string.utf8] -> std::map.ordered[string.utf8, Reach]:
    variable $best: std::map.ordered[string.utf8, Reach] = std::map.empty;
    variable $done: std::map[string.utf8, boolean] = std::map.empty;
    std::map.set(lend.mutable $best, key: copy $from, value: Reach[minutes = 0, km = 0, prev = string.empty]);
    loop:
        # the unfinished station reached soonest; the first in the station list wins a tie
        variable $found := false;
        variable $pick := string.empty;
        variable $pick_minutes: trip.minutes = 0;
        variable $pick_km: trip.km = 0;
        for $s in lend $net.stations:
            if std::map.has(lend $done, key: lend $s):
                continue;
            end of if
            if not std::map.has(lend $best, key: lend $s):
                continue;
            end of if
            let $r := std::map.get(lend $best, key: lend $s) otherwise:
                !case missing:
                    continue;
                !end of case missing
            end of otherwise;
            if not $found or.else $r.minutes less_than $pick_minutes:
                change $found = true;
                change $pick = copy $s;
                change $pick_minutes = $r.minutes;
                change $pick_km = $r.km;
            end of if
        end of for
        if not $found:
            break;
        end of if
        std::map.set(lend.mutable $done, key: copy $pick, value: true);
        variable $wait := $change_minutes;
        if $pick equal_to $from:
            change $wait = 0;
        end of if
        for $l in lend $net.links:
            let $minutes := $pick_minutes plus $wait plus $l.minutes;
            let $km := $pick_km plus $l.km;
            if $l.a equal_to $pick:
                try_reach(lend.mutable $best, lend $done, lend $l.b, lend $pick, $minutes, $km);
            end of if
            else if $l.b equal_to $pick:
                try_reach(lend.mutable $best, lend $done, lend $l.a, lend $pick, $minutes, $km);
            end of else if
        end of for
    end of loop
    return give $best;
end of procedure quickest_from

# Reaching 'next' from 'pick' in 'minutes': kept only if no sooner way is known. The map keeps its own copy of
# both names, made here and nowhere else.
procedure try_reach [
    lent.mutable $best: std::map.ordered[string.utf8, Reach],
    lent $done: std::map[string.utf8, boolean],
    lent $next: string.utf8,
    lent $pick: string.utf8,
    $minutes: integer.signed.32_bit,
    $km: integer.signed.32_bit
]:
    if std::map.has(lend $done, key: lend $next):
        return;
    end of if
    # past the longest trip sold, it is no way at all
    if $minutes less_than 0 or $minutes greater_than 14400 or $km less_than 0 or $km greater_than 100000:
        return;
    end of if
    if std::map.has(lend $best, key: lend $next):
        let $old := std::map.get(lend $best, key: lend $next) otherwise:
            !case missing:
                return;
            !end of case missing
        end of otherwise;
        if $old.minutes at_most $minutes:
            return;
        end of if
    end of if
    std::map.set(lend.mutable $best, key: copy $next, value: Reach[minutes = $minutes, km = $km, prev = copy $pick]);
end of procedure try_reach

# The stops from 'from' to 'to', walked back from 'to' along each station's `prev`, then turned round.
procedure journey [lent $net: Network, lent $from: string.utf8, lent $to: string.utf8] -> Journey or fail[unknown_station: string.utf8, same_station, no_route]:
    if not knows(lend $net, lend $from):
        fail unknown_station[copy $from];
    end of if
    if not knows(lend $net, lend $to):
        fail unknown_station[copy $to];
    end of if
    if $from equal_to $to:
        fail same_station;
    end of if
    let $best := quickest_from(lend $net, lend $from);
    let $end := std::map.get(lend $best, key: lend $to) otherwise:
        case missing:
            fail no_route;
        end of case missing
    end of otherwise;
    variable $back: std::list[string.utf8] = std::list.empty;
    variable $at := copy $to;
    loop:
        std::list.append(lend.mutable $back, value: copy $at);
        if $at equal_to $from:
            break;
        end of if
        let $r := std::map.get(lend $best, key: lend $at) otherwise:
            case missing:
                fail no_route;
            end of case missing
        end of otherwise;
        change $at = copy $r.prev;
    end of loop
    variable $stops: std::list[string.utf8] = std::list.empty;
    let $n := std::list.length(lend $back);
    for $k [from: 1, to: $n, by: 1]:
        std::list.append(lend.mutable $stops, value: copy $back[position: $n minus $k plus 1]);
    end of for
    return Journey[stops = give $stops, minutes = $end.minutes, km = $end.km];
end of procedure journey

# ---------------------------------------------------------------------------------------------------
# Clock times: minutes since midnight of the departure day as "07:40", with "+1" for the next morning
# ---------------------------------------------------------------------------------------------------

function clock [$minutes: integer.signed.32_bit] -> string.utf8:
    let $day := $minutes divide.floor 1440;
    let $in_day := $minutes modulo.floor 1440;
    let $text := std::format("{zeros, width: 2 columns}:{zeros, width: 2 columns}", $in_day divide.floor 60, $in_day modulo.floor 60);
    if $day equal_to 0:
        return give $text;
    end of if
    return $text concatenate " +" concatenate to.stored.string.utf8($day);
end of function clock

# A clock that hands out the next quarter hour each time it is asked, until it reaches its end.
record Clock:
    now: integer.signed.32_bit[minimum: 0, maximum: 2895];
    stop: integer.signed.32_bit[minimum: 0, maximum: 2880];
end of record Clock

procedure Clock.next [lent.mutable $c: Clock] -> optional[integer.signed.32_bit]:
    # the stop is never past the next day's end, so neither is the last time handed out
    let $t := $c.now;
    if $t at_least $c.stop:
        return optional.none;
    end of if
    if $t at_least 2880:
        return optional.none;
    end of if
    change $c.now = $t plus 15;
    return optional.value($t);
end of procedure Clock.next

# ---------------------------------------------------------------------------------------------------
# Fares, in exact decimals
# ---------------------------------------------------------------------------------------------------

function distance_fare [$km: integer.signed.32_bit] -> float.%decimal.64_bit:
    if $km at_most $near_km:
        return $near_rate times to.float.%decimal.64_bit($km);
    end of if
    let $far := to.float.%decimal.64_bit($km minus $near_km);
    return $near_rate times to.float.%decimal.64_bit($near_km) plus $far_rate times $far;
end of function distance_fare

# The bed costs the same per compartment, shared by however many sleep in it.
function supplement [lent $seat: Seat] -> float.%decimal.64_bit:
    match $seat:
        case seat:
            return 0;
        end of case seat
        case couchette:
            return 25.00;
        end of case couchette
        case sleeper $berths:
            return 120.00 divide.nearest to.float.%decimal.64_bit($berths);
        end of case sleeper
    end of match
end of function supplement

function discount [$age: Age] -> integer.signed.32_bit[minimum: 0, maximum: 50]:
    let $percent: integer.signed.32_bit[minimum: 0, maximum: 50] = match $age:
        case adult:
            is 0;
        end of case adult
        case youth:
            is 25;
        end of case youth
        case senior:
            is 30;
        end of case senior
    end of match;
    return $percent;
end of function discount

# The fare before rounding can hold more than two places (30% of 0.15 is 0.045): it is rounded to the
# cent once, at the end, half to even, and said so.
function fare [$km: integer.signed.32_bit, lent $seat: Seat, $age: Age] -> float.%decimal.64_bit:
    let $full := distance_fare($km) plus supplement(lend $seat);
    let $off := to.float.%decimal.64_bit(discount(copy $age));
    let $exact := $full minus ($full times $off) divide.nearest 100;
    return std::round.nearest($exact, places: 2);
end of function fare

function seat_name [lent $seat: Seat] -> string.utf8:
    match $seat:
        case seat:
            return "seat";
        end of case seat
        case couchette:
            return "couchette";
        end of case couchette
        case sleeper $berths:
            return "sleeper for " concatenate to.stored.string.utf8($berths);
        end of case sleeper
    end of match
end of function seat_name

function age_name [$age: Age] -> string.utf8:
    match $age:
        case adult:
            return "adult";
        end of case adult
        case youth:
            return "youth";
        end of case youth
        case senior:
            return "senior";
        end of case senior
    end of match
end of function age_name

# ---------------------------------------------------------------------------------------------------
# Reading requests: "Ana: Lisboa > Wien, sleeper 2, youth"
# ---------------------------------------------------------------------------------------------------

function parse_seat [lent $text: string.utf8] -> Seat or fail[bad_request: string.utf8]:
    if $text equal_to "seat":
        return Seat.seat;
    end of if
    if $text equal_to "couchette":
        return Seat.couchette;
    end of if
    let $words := std::string.split(lend $text, at: " ");
    if std::list.length(lend $words) not_equal_to 2 or.else $words[position: 1] not_equal_to "sleeper":
        fail bad_request[copy $text];
    end of if
    let $berths: integer.signed.32_bit[minimum: 1, maximum: 4] = to.integer.signed.32_bit[minimum: 1, maximum: 4]($words[position: 2]) otherwise:
        case not_a_number:
            fail bad_request[copy $text];
        end of case not_a_number
        case out_of_range:
            fail bad_request[copy $text];
        end of case out_of_range
    end of otherwise;
    return Seat.sleeper($berths);
end of function parse_seat

function parse_age [lent $text: string.utf8] -> Age or fail[bad_request: string.utf8]:
    if $text equal_to "adult":
        return Age.adult;
    end of if
    if $text equal_to "youth":
        return Age.youth;
    end of if
    if $text equal_to "senior":
        return Age.senior;
    end of if
    fail bad_request[copy $text];
end of function parse_age

function parse_request [lent $line: string.utf8] -> Request or fail[bad_request: string.utf8]:
    let $who := std::string.split(lend $line, at: ": ");
    if std::list.length(lend $who) not_equal_to 2:
        fail bad_request[copy $line];
    end of if
    let $parts := std::string.split(lend $who[position: 2], at: ", ");
    if std::list.length(lend $parts) not_equal_to 3:
        fail bad_request[copy $line];
    end of if
    let $ends := std::string.split(lend $parts[position: 1], at: " > ");
    if std::list.length(lend $ends) not_equal_to 2:
        fail bad_request[copy $line];
    end of if
    let $seat := parse_seat(lend $parts[position: 2]) otherwise fail;
    let $age := parse_age(lend $parts[position: 3]) otherwise fail;
    return Request[traveller = copy $who[position: 1], origin = copy $ends[position: 1], destination = copy $ends[position: 2], seat = give $seat, age = give $age];
end of function parse_request

function todays_requests [] -> std::list[string.utf8]:
    return [
        "Ana: Lisboa > Wien, sleeper 2, youth",
        "Bram: Amsterdam > Kraków, couchette, adult",
        "Chloé: Paris > Stockholm, seat, senior",
        "Dario: Madrid > Madrid, seat, adult",
        "Eun: Berlin > Palermo, sleeper 1, adult",
        "Femi: Zürich > Oslo, seat, adult",
        "Gil: Praha > Lisboa, sleeper 4, adult",
        "Hana: Köln > Wien sleeper 2",
        "Ines: München > København, sleeper 1, senior",
        "Jo: Wien > Berlin, sleeper 1, youth",
        "Kai: Bruxelles > Zürich, sleeper 3, youth"
    ];
end of function todays_requests

# ---------------------------------------------------------------------------------------------------
# Booking: a journey, a bed of the kind asked for if one is left, and a fare
# ---------------------------------------------------------------------------------------------------

# How many of each kind of place are left on tonight's trains, by `seat_name`.
function places_left [] -> std::map.ordered[string.utf8, integer.signed.32_bit]:
    variable $left: std::map.ordered[string.utf8, integer.signed.32_bit] = std::map.empty;
    std::map.set(lend.mutable $left, key: "seat", value: 40);
    std::map.set(lend.mutable $left, key: "couchette", value: 6);
    std::map.set(lend.mutable $left, key: "sleeper for 1", value: 1);
    std::map.set(lend.mutable $left, key: "sleeper for 2", value: 4);
    std::map.set(lend.mutable $left, key: "sleeper for 3", value: 3);
    std::map.set(lend.mutable $left, key: "sleeper for 4", value: 8);
    return give $left;
end of function places_left

procedure book [
    lent $net: Network,
    lent.mutable $left: std::map.ordered[string.utf8, integer.signed.32_bit],
    lent $req: Request,
    $ticket: integer.signed.32_bit
] -> Booking or fail[unknown_station: string.utf8, same_station, no_route, sold_out: string.utf8]:
    let $trip := journey(lend $net, lend $req.origin, lend $req.destination) otherwise fail;
    let $kind := seat_name(lend $req.seat);
    # a kind of place the trains do not carry has none left, said here rather than read as a zero
    let $free := std::map.get(lend $left, key: lend $kind) otherwise:
        case missing:
            fail sold_out[give $kind];
        end of case missing
    end of otherwise;
    if $free at_most 0:
        fail sold_out[give $kind];
    end of if
    std::map.set(lend.mutable $left, key: copy $kind, value: $free minus 1);
    let $price := fare($trip.km, lend $req.seat, copy $req.age);
    return Booking[ticket = $ticket, traveller = copy $req.traveller, journey = give $trip, seat = copy $req.seat, fare = $price];
end of procedure book

# A request that cannot be booked is explained the same way wherever it is tried: one handler, told
# whom it is explaining to. Its arms act as if written at the call, so `continue` moves on to the next
# request there (owner, 2026-10-25).
collection.handler booking_failed:
    booking_failed.explain [$who: string.utf8]:
        !case unknown_station $name:
            std::print.shortest("  ", $who, ": no station called ", $name, " on this network", newline);
            continue;
        !end of case unknown_station
        !case same_station:
            std::print.shortest("  ", $who, ": that journey starts where it ends", newline);
            continue;
        !end of case same_station
        !case no_route:
            std::print.shortest("  ", $who, ": no night train joins those two stations", newline);
            continue;
        !end of case no_route
        !case sold_out $kind:
            std::print.shortest("  ", $who, ": no ", $kind, " left tonight", newline);
            continue;
        !end of case sold_out
    end of booking_failed.explain
end of collection.handler booking_failed

procedure show_booking [lent $b: Booking, $age: Age]:
    let $changes := std::list.length(lend $b.journey.stops) minus 2;
    std::print.shortest("  #", $b.ticket, " ", $b.traveller, " (", age_name(copy $age), "), ", seat_name(lend $b.seat), newline);
    std::print.shortest("    ", std::string.join(lend $b.journey.stops, between: " → "), newline);
    std::print.format("    {} km, {} change(s), leaves {}, arrives {}, fare {places: 2}{newline}", $b.journey.km, $changes, clock($departure), clock($departure plus $b.journey.minutes), $b.fare);
end of procedure show_booking

# ---------------------------------------------------------------------------------------------------
# The departure board: everywhere reachable from one station tonight, soonest first
# ---------------------------------------------------------------------------------------------------

record Arrival:
    station: string.utf8;
    minutes: trip.minutes;
    km: trip.km;
end of record Arrival

# Soonest first: each arrival goes in before the first one later than it, so arrivals at the same time keep their
# order. The list is written out anew around each one, which needs no positions.
function soonest_first [lent $xs: std::list[Arrival]] -> std::list[Arrival]:
    variable $out: std::list[Arrival] = std::list.empty;
    for $x in lend $xs:
        variable $next: std::list[Arrival] = std::list.empty;
        variable $placed := false;
        for $y in lend $out:
            if not $placed and $x.minutes less_than $y.minutes:
                std::list.append(lend.mutable $next, value: copy $x);
                change $placed = true;
            end of if
            std::list.append(lend.mutable $next, value: copy $y);
        end of for
        if not $placed:
            std::list.append(lend.mutable $next, value: copy $x);
        end of if
        change $out = give $next;
    end of for
    return give $out;
end of function soonest_first

procedure board [lent $net: Network, lent $from: string.utf8]:
    variable $out := std::print.writer(size: 4096 bytes);
    std::print.shortest(lend.mutable $out, newline, "departures from ", $from, " at ", clock($departure), newline);
    std::print.format(lend.mutable $out, "  {align: left, width: 12 columns} {align: left, width: 8 columns} {align: right, width: 6 columns}{newline}", "to", "arrives", "km");
    let $best := quickest_from(lend $net, lend $from);
    variable $arrivals: std::list[Arrival] = std::list.empty;
    for $e in std::map.pairs(lend $best):
        if $e.key equal_to $from:
            continue;
        end of if
        std::list.append(lend.mutable $arrivals, value: Arrival[station = copy $e.key, minutes = $e.value.minutes, km = $e.value.km]);
    end of for
    let $sorted := soonest_first(lend $arrivals);
    for $a in lend $sorted:
        std::print.format(lend.mutable $out, "  {align: left, width: 12 columns} {width: 8 columns} {width: 6 columns}{newline}", $a.station, clock($departure plus $a.minutes), $a.km);
    end of for
    std::print.flush(lend.mutable $out);
end of procedure board

# ---------------------------------------------------------------------------------------------------
# Counting at the same time: how many stations each hub reaches by eight in the morning, each hub
# worked out by its own task, which reports back over a channel
# ---------------------------------------------------------------------------------------------------

procedure reached_by [lent $net: Network, lent $hub: string.utf8, $deadline: integer.signed.32_bit] -> integer.unsigned.64_bit:
    let $best := quickest_from(lend $net, lend $hub);
    variable $reached: std::list[string.utf8] = std::list.empty;
    for $e in std::map.pairs(lend $best):
        if $e.key not_equal_to $hub and $departure plus $e.value.minutes at_most $deadline:
            std::list.append(lend.mutable $reached, value: copy $e.key);
        end of if
    end of for
    return std::list.length(lend $reached);
end of procedure reached_by

procedure compare_hubs [lent $net: Network]:
    let $paris_ch: %channel[integer.unsigned.64_bit] = %channel.empty;
    let $koln_ch: %channel[integer.unsigned.64_bit] = %channel.empty;
    let $wien_ch: %channel[integer.unsigned.64_bit] = %channel.empty;
    let $paris := "Paris";
    let $koln := "Köln";
    let $wien := "Wien";
    %spawn.task with [lend $paris_ch, lend $net, lend $paris]:
        %channel.send(lend $paris_ch, value: reached_by(lend $net, lend $paris, $by_eight));
    end of %spawn.task
    %spawn.task with [lend $koln_ch, lend $net, lend $koln]:
        %channel.send(lend $koln_ch, value: reached_by(lend $net, lend $koln, $by_eight));
    end of %spawn.task
    %spawn.task with [lend $wien_ch, lend $net, lend $wien]:
        %channel.send(lend $wien_ch, value: reached_by(lend $net, lend $wien, $by_eight));
    end of %spawn.task
    let $p := %channel.take(lend $paris_ch) otherwise 0;
    let $k := %channel.take(lend $koln_ch) otherwise 0;
    let $w := %channel.take(lend $wien_ch) otherwise 0;
    std::print.shortest(newline, "stations reached by ", clock($by_eight), ": Paris ", $p, ", Köln ", $k, ", Wien ", $w, newline);
end of procedure compare_hubs

# ---------------------------------------------------------------------------------------------------
# Looking things up: the stations whose names hold a given letter
# ---------------------------------------------------------------------------------------------------

function stations_where [lent $net: Network, $keep: function[string.utf8] -> boolean] -> std::list[string.utf8]:
    variable $out: std::list[string.utf8] = std::list.empty;
    for $s in lend $net.stations:
        if $keep(copy $s):
            std::list.append(lend.mutable $out, value: copy $s);
        end of if
    end of for
    return give $out;
end of function stations_where

procedure show_letters [lent $net: Network]:
    std::print.shortest(newline, "stations by letter:", newline);
    let $letters := ["ö", "ü", "ó", "x"];
    for $part in lend $letters:
        let $found := stations_where(lend $net, function [$s] with [lend $part] = std::string.contains(lend $s, part: lend $part));
        if std::list.length(lend $found) equal_to 0:
            std::print.shortest("  none with ", $part, newline);
        end of if
        else:
            std::print.shortest("  with ", $part, ": ", std::string.join(lend $found, between: ", "), newline);
        end of else
    end of for
end of procedure show_letters

# ---------------------------------------------------------------------------------------------------
# The report
# ---------------------------------------------------------------------------------------------------

procedure report [lent $bookings: std::list[Booking][maximum: 500], lent $left: std::map.ordered[string.utf8, integer.signed.32_bit]]:
    variable $out := std::print.writer(size: 4096 bytes);
    std::print.shortest(lend.mutable $out, newline, "=== ", $network_name, ", tonight ===", newline);
    variable $fares: std::list[float.%decimal.64_bit][maximum: 500] = std::list.empty;
    variable $kms: std::list[integer.signed.32_bit][maximum: 500] = std::list.empty;
    variable $longest := 0;
    variable $longest_who := "nobody";
    for $b in lend $bookings:
        std::list.append(lend.mutable $fares, value: $b.fare);
        std::list.append(lend.mutable $kms, value: $b.journey.km);
        if $b.journey.km greater_than $longest:
            change $longest = $b.journey.km;
            change $longest_who = copy $b.traveller;
        end of if
    end of for
    let $count := std::list.length(lend $bookings);
    let $takings := total_of(lend $fares);
    std::print.shortest(lend.mutable $out, "tickets sold: ", $count, newline);
    std::print.format(lend.mutable $out, "takings: {places: 2}{newline}", $takings);
    if $count greater_than 0:
        let $average := std::round.nearest($takings divide.nearest to.float.%decimal.64_bit(to.integer.signed.32_bit($count)), places: 2);
        std::print.format(lend.mutable $out, "average fare: {places: 2}{newline}", $average);
    end of if
    std::print.shortest(lend.mutable $out, "km travelled: ", total_of(lend $kms), ", the longest ", $longest, " km (", $longest_who, ")", newline);
    std::print.shortest(lend.mutable $out, "places left:", newline);
    for $e in std::map.pairs(lend $left):
        std::print.format(lend.mutable $out, "  {align: left, width: 14 columns} {width: 3 columns}{newline}", $e.key, $e.value);
    end of for
    std::print.flush(lend.mutable $out);
end of procedure report

# Every request of the day, each with the next ticket number; the ones that cannot be read or booked are explained.
procedure book_all [lent $net: Network, lent.mutable $left: std::map.ordered[string.utf8, integer.signed.32_bit], lent $lines: std::list[string.utf8]] -> std::list[Booking][maximum: 500]:
    std::print.shortest(newline, "bookings:", newline);
    variable $bookings: std::list[Booking][maximum: 500] = std::list.empty;
    variable $ticket := 500;
    for $line in lend $lines:
        if std::list.length(lend $bookings) at_least $most_bookings:
            std::print.shortest("  (tonight's trains are full)", newline);
            break;
        end of if
        let $req := parse_request(lend $line) otherwise:
            !case bad_request $what:
                if $what equal_to $line:
                    std::print.shortest("  (cannot read \"", $line, "\")", newline);
                end of if
                else:
                    std::print.shortest("  (cannot read \"", $what, "\" in \"", $line, "\")", newline);
                end of else
                continue;
            !end of case bad_request
        end of otherwise;
        let $b := book(lend $net, lend.mutable $left, lend $req, $ticket plus 1) otherwise booking_failed.!explain(who: copy $req.traveller);
        change $ticket = $ticket plus 1;
        show_booking(lend $b, copy $req.age);
        std::list.append(lend.mutable $bookings, value: give $b);
    end of for
    return give $bookings;
end of procedure book_all

# The night's quarter hours, from departure to midnight, walked by the clock's own iterator.
procedure show_quarter_hours []:
    std::print.shortest(newline, "the last quarter hours before midnight:");
    variable $quarters := Clock[now = $departure, stop = 1440];
    for $t in lend.mutable $quarters:
        std::print.shortest(" ", clock($t));
    end of for
    std::print.shortest(newline);
end of procedure show_quarter_hours

# Exact decimals: why a fare is a `d64`. A fare of 0.10 three times is 0.30 exactly.
procedure show_exact_money []:
    let $dime: float.%decimal.64_bit = 0.10;
    let $three := $dime plus $dime plus $dime;
    let $float: float.binary.64_bit = 0.1 plus 0.1 plus 0.1;
    std::print.shortest("three fares of 0.10 as d64: ");
    std::print.stored($three, newline);
    std::print.shortest("the same in f64: ");
    std::print.stored($float, newline);
end of procedure show_exact_money

# ---------------------------------------------------------------------------------------------------
# The night: read the timetable, then each part in turn
# ---------------------------------------------------------------------------------------------------

!MAIN:
    std::print.shortest("the ", $network_name, " night trains", newline, newline, "reading the timetable:", newline);
    let $rows := timetable();
    let $net := read_network(lend $rows);
    std::print.shortest("  ", std::list.length(lend $net.links), " lines between ", std::list.length(lend $net.stations), " stations", newline);
    board(lend $net, "Wien");
    variable $left := places_left();
    let $requests := todays_requests();
    let $bookings := book_all(lend $net, lend.mutable $left, lend $requests);
    report(lend $bookings, lend $left);
    compare_hubs(lend $net);
    show_letters(lend $net);
    show_quarter_hours();
    show_exact_money();
    std::print.shortest("good night from ", $network_name, newline);
!end of MAIN
