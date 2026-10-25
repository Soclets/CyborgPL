# A format width names what it counts (owner, 2026-10-25). `cols` counts screen columns: a Chinese, Japanese or
# Korean character and most emoji take two, an accent added onto a letter none, and an East Asian Ambiguous one
# (`°`, `±`, Greek, Cyrillic) one; `cols.east_asian` counts that last kind as two, for a terminal set to those
# languages. `graphemes`, `points` and `bytes` count as a text's length does. The `|` shows where each hole ends.
MAIN {
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|\n", "tea", 2.5);
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|\n", "日本茶", 3.5);
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|\n", "café", 4.0);
    std::print.format("{align: left, width: 8 cols}|{width: 6 cols, places: 1}|\n", "👍🏽 ok", 1.0);
    std::print.format("{width: 6 cols}|{width: 6 cols.east_asian}|\n", "20°C", "20°C");
    std::print.format("{width: 6 graphemes}|{width: 6 points}|{width: 6 bytes}|\n", "日本", "日本", "日本");
    std::print.format("{align: center, width: 9 cols, fill: `-`}|{base: hex.upper, zeros, width: 6 cols}|{sign}|\n", "mid", 255, 7);
}
