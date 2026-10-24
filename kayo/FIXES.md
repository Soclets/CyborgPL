# Fixes, family by family

The owner, 2026-10-24: every message is its name, `code:` and `fix:` as edited code, and every edit kayo can
write is shown under `or:`; `fix: none suggested` only where the fix lies outside the program's own code; until a
message's fix is written its sentence stays on `about:`. Each fix is proven by `every_fix_clears_its_error`
(tests/conformance.rs): the reject program's message, each fix applied as its replacements, and the same error no
longer fires on that line.

Status: **written** (every site of the name has its fixes), **partly** (some sites; the rest keep `about:`, said
why), **none suggested** (the fix is outside the program, said why), **—** (not yet).

## spelling and syntax (132 names)

| name | sites | status | notes |
|---|---|---|---|
| `any_stands_alone` | 2 | — | |
| `argument_missing` | 2 | — | |
| `asm_binding_form` | 5 | — | |
| `assignment_in_value` | 2 | written | `=` → `equal_to` |
| `belongs_at_top_level` | 6 | — | |
| `belongs_in_body` | 3 | — | |
| `body_missing` | 1 | — | |
| `borrow_group_form` | 4 | — | |
| `borrow_group_needed` | 1 | — | |
| `braces_need_commas` | 2 | written | the `,` put in, or a case's `;` made one |
| `byte_order_needless` | 1 | — | |
| `byte_order_unsaid` | 2 | — | |
| `call_in_brackets` | 3 | written | both brackets swapped |
| `capture_verb_unsaid` | 1 | — | |
| `case_unhandled` | 2 | — | |
| `case_value_form` | 1 | — | |
| `change_unmarked` | 1 | written | `change ` in front |
| `character_empty` | 1 | — | |
| `check_misplaced` | 9 | partly | inside a value: moved in front of the statement; not yet: on a `fn`, at the top level, alone |
| `cnc::c_capitalized` | 4 | written | `C` |
| `cnc::extern_language_unknown` | 1 | — | |
| `cnc::extern_language_unsaid` | 2 | — | |
| `cnc::extern_with_definition` | 2 | — | |
| `cnc::header_not_found` | 1 | — | |
| `cnc::incl_c_form` | 3 | — | |
| `cnc::lent_across_c` | 1 | — | |
| `cnc::unknown_layout` | 1 | — | |
| `collection_form` | 9 | — | |
| `collection_kind_unsaid` | 2 | — | |
| `collection_line_form` | 4 | — | |
| `collection_member_unprefixed` | 6 | — | |
| `comment_form` | 5 | partly | `#x` → `# x`; not yet: the generated marker |
| `comparisons_chained` | 1 | — | |
| `const_changed` | 1 | — | |
| `const_marked_mut` | 1 | written | the word out |
| `conversion_form` | 6 | — | |
| `counter_is_position` | 1 | — | |
| `counting_loop_form` | 3 | — | |
| `declaration_form` | 17 | partly | `let` put in, `mut` / a type moved where it goes, `:=` ↔ `=` by whether a type is written, `let.mut` → `let mut`; not yet: C's variable forms, a generic's one `< >`, a declaration with more than a name |
| `digit_grouping` | 2 | written | one `_` between digits; `1,000` → `1_000` |
| `digit_outside_base` | 1 | — | |
| `enum_empty` | 1 | — | |
| `fail_list_empty` | 1 | — | |
| `field_form` | 7 | partly | the `:`, name first, `;` / `let` out, `:=` → `=`; not yet: `mut` / `immut` placement |
| `foreign_spelling` | 4 | partly | `0x1F` → `hex 1F`, `mod` → `module`, `usize` → `u64` / `u32`, `String` → `str.utf8` / `str.ascii`; not yet: a module include line |
| `function_type_form` | 1 | — | |
| `generated_marker_form` | 1 | — | |
| `generic_bound_missing` | 1 | — | |
| `generic_bound_twice` | 1 | — | |
| `ieee_misused` | 2 | — | |
| `include_form` | 3 | — | |
| `include_path_not_literal` | 2 | — | |
| `inline_block_form` | 4 | — | |
| `inline_language_capitals` | 3 | written | the capitals |
| `interface_bound_needless` | 1 | — | |
| `interface_empty` | 1 | — | |
| `interface_has_code` | 1 | — | |
| `interface_types_one_per_line` | 1 | — | |
| `keyword_name_bare` | 2 | — | |
| `label_needs_colon` | 3 | partly | the `:` put in after the first word; next: every word in the brackets at once |
| `label_order_unsaid` | 2 | — | |
| `lambda_types_unknown` | 2 | — | |
| `leading_zero` | 1 | written | the zeros taken off |
| `lent_at_call` | 1 | — | |
| `level_chain_unsaid` | 2 | — | |
| `level_does_not_chain` | 1 | — | |
| `level_numbered_twice` | 1 | — | |
| `level_order_unsaid` | 2 | — | |
| `limb_not_listed` | 1 | — | |
| `loop_words_form` | 2 | — | |
| `main_form` | 4 | written | `pub` out, `fn MAIN … {` → `MAIN {`, a call to `MAIN` goes |
| `main_parameters_wrong` | 4 | partly | one parameter: `, $argv: ptr[ptr[u8]]` added, or none; not yet: three or more |
| `mcps::seq_and_para` | 1 | — | |
| `mcps::whichever_empty` | 1 | — | |
| `module_name_lowercase` | 1 | written | the name lowercased, where that makes a name |
| `money::exactly_form` | 1 | — | |
| `name_not_english` | 1 | — | |
| `name_quoted` | 13 | written | the name bare |
| `name_reserved` | 10 | — | |
| `needless_lent` | 1 | — | |
| `needs_parentheses` | 7 | partly | both groupings, under `or:`; not yet the `not` next to a comparison (the unary site) |
| `no_starting_value` | 1 | — | |
| `not_built_yet` | 22 | none suggested | kayo has not built it: the fix is kayo's (design claude, 2026-10-24) |
| `not_changeable` | 2 | — | |
| `not_decided` | 7 | none suggested | no ruling yet: the fix is the owner's |
| `not_whole_number` | 4 | — | |
| `old_spelling` | 44 | partly | every replacement the spellings table or the message names, as edits (moves too: `no_cheat for`, `<T: any>`, a slice's `from:`/`to:`); not yet: a `range.` loop, `level.when_chained`, `change *$p =`, `2 ! 3` (no edit means anything there) |
| `on_form` | 5 | — | |
| `operator_cannot_fail` | 1 | — | |
| `operator_cannot_level` | 1 | — | |
| `operator_value_form` | 5 | — | |
| `operators_level_apart` | 1 | — | |
| `otherwise_empty` | 2 | — | |
| `otherwise_misplaced` | 7 | — | |
| `parameter_form` | 6 | partly | `var` out, name first, `lent.str.utf8 $t` → `lent $t: str.utf8`; not yet: `lend` / an owning keyword |
| `parens_for_brackets` | 4 | written | both brackets swapped |
| `preset_wrong` | 4 | — | |
| `print_weird_form` | 3 | — | |
| `quoted_name_form` | 4 | — | |
| `range_as_value` | 2 | — | |
| `return_type_form` | 3 | — | |
| `return_type_missing` | 5 | — | |
| `said_twice` | 5 | — | |
| `shadow_form` | 1 | — | |
| `space_missing` | 7 | partly | the space put in; not yet: `<` standing alone in a type |
| `spelling_removed` | 27 | written | the mark, `#strict` or `lang:` goes (a line of its own whole); `var` → `let` / `const`; the `bare.` sites after `bare_mark` are unreachable since the marks went |
| `step_missing` | 1 | — | |
| `strict_misplaced` | 1 | — | |
| `strict_word_missing` | 2 | — | |
| `table_bytes_form` | 3 | — | |
| `tier_words_order` | 3 | — | |
| `top_level_needs_const` | 4 | — | |
| `two_tiers` | 1 | — | |
| `two_verbs` | 2 | — | |
| `type_incomplete` | 5 | — | |
| `unclosed_block` | 16 | partly | the `}` above the next item, or a last line; not yet: a `[` never closed |
| `unclosed_quote` | 3 | written | closed at the end of its line, before a `;` |
| `unexpected_text` | 53 | — | |
| `unicode::case_mark_kind_unknown` | 2 | — | |
| `unicode::case_mark_misplaced` | 6 | — | |
| `unicode::case_mark_order` | 1 | — | |
| `unknown_handler` | 1 | — | |
| `unknown_inline_language` | 1 | — | |
| `unknown_narrowing` | 3 | — | |
| `unknown_operator` | 6 | — | |
| `unknown_pub_narrowing` | 2 | — | |
| `unknown_target` | 5 | — | |
| `unknown_verb` | 4 | — | |
| `variable_name_form` | 5 | — | |
| `variadic_form` | 2 | — | |
| `versions_outside_collection` | 3 | — | |
| `wrap_needless` | 1 | — | |

## names and paths (62 names)

| name | sites | status | notes |
|---|---|---|---|
| `ai::brain_width` | 1 | — | |
| `bound_given_twice` | 1 | — | |
| `bound_not_taken` | 2 | — | |
| `bound_outside_type` | 6 | — | |
| `bound_unit_misplaced` | 6 | — | |
| `bound_unit_unsaid` | 1 | — | |
| `bound_wrong_type` | 8 | — | |
| `bounds_backwards` | 6 | written | the two sides swapped |
| `builtin_operator_redefined` | 3 | — | |
| `cnc::bit_field_form` | 2 | — | |
| `cnc::bit_field_too_wide` | 1 | — | |
| `cnc::bits_outside_layout` | 1 | — | |
| `cnc::c_cannot_fail` | 1 | — | |
| `cnc::c_enum_carries_data` | 1 | — | |
| `cnc::c_enum_number_missing` | 1 | — | |
| `cnc::c_enum_number_twice` | 1 | — | |
| `cnc::c_str_counted_in_bytes` | 1 | — | |
| `cnc::c_str_zero_misplaced` | 1 | — | |
| `cnc::c_str_zero_unsaid` | 2 | — | |
| `cnc::c_type_unknown` | 1 | — | |
| `cnc::cannot_cross_to_c` | 7 | — | |
| `cnc::not_c_shaped` | 1 | — | |
| `cnc::not_layout_c` | 1 | — | |
| `defined_twice` | 13 | — | |
| `function_owner_wrong` | 1 | — | |
| `kit_name_needs_path` | 2 | written | the kit's prefix in front, or a `use … as …;` line |
| `kit_not_listed` | 2 | — | |
| `lane_count_misplaced` | 1 | — | |
| `length_bound_on_array` | 2 | — | |
| `main_not_at_root` | 1 | — | |
| `mcps::channel_bound_form` | 1 | — | |
| `member_names_itself` | 1 | — | |
| `money::percent_number_type` | 1 | — | |
| `money::percent_unsized` | 1 | — | |
| `money::places_on_bin` | 1 | — | |
| `my_after_kit_path` | 1 | — | |
| `my_needs_path` | 2 | — | |
| `narrowing_not_taken` | 2 | — | |
| `no_such_width` | 1 | — | |
| `not_a_map_key` | 4 | — | |
| `not_a_module` | 3 | — | |
| `not_a_type` | 1 | — | |
| `not_public` | 5 | partly | `pub.kit` / `pub` in front of the declaration; another kit's: none suggested (its code is that kit's) |
| `operator_operand_count` | 2 | — | |
| `parameter_type_missing` | 4 | — | |
| `simd_element_type` | 1 | — | |
| `simd_lane_count_mismatch` | 4 | — | |
| `simd_lane_width` | 1 | — | |
| `std_name_needs_path` | 5 | written | `std::` in front |
| `std_not_listed` | 6 | — | |
| `test_fn_misused` | 5 | — | |
| `type_parameter_count` | 9 | — | |
| `type_parameter_form` | 3 | — | |
| `unannounced` | 3 | — | |
| `unknown_const` | 1 | — | |
| `unknown_function` | 4 | partly | the close name; the declaring fix waits on the owner |
| `unknown_interface` | 3 | — | |
| `unknown_map_variant` | 1 | — | |
| `unknown_text_unit` | 5 | — | |
| `unknown_type` | 1 | partly | the close name |
| `wsmw::no_such_width` | 4 | — | |
| `wsmw::standard_width` | 1 | — | |

## types and values (161 names)

| name | sites | status | notes |
|---|---|---|---|
| `ai::brain_mixed` | 1 | — | |
| `ai::brain_to_deci_unsaid` | 1 | — | |
| `already_declared` | 5 | — | |
| `argument_label_missing` | 9 | partly | the label put in, or a mistyped one replaced (`Edit::Label`); not yet: a handler's argument, a labelled argument of the own function |
| `argument_label_unknown` | 5 | partly | the close label; not yet: a version's, a handler's |
| `array_cannot_grow` | 2 | — | |
| `array_never_filled` | 1 | — | |
| `borrow_outlives_owner` | 2 | — | |
| `borrow_outside_group` | 3 | — | |
| `bound_not_satisfied` | 3 | — | |
| `bundle_loop_form` | 2 | — | |
| `call_ambiguous` | 1 | — | |
| `call_gives_nothing` | 3 | — | |
| `call_without_parens` | 3 | partly | `()` after it; not yet: a path alone as a statement |
| `cannot_change_immut` | 3 | — | |
| `carries_nothing` | 4 | — | |
| `case_data_form` | 2 | — | |
| `case_handled_twice` | 1 | — | |
| `case_value_misplaced` | 1 | — | |
| `case_value_missing` | 1 | — | |
| `changed_through_lend` | 2 | — | |
| `cnc::c_str_unknown` | 1 | — | |
| `cnc::c_variable_read_only` | 1 | — | |
| `cnc::callback_captures` | 1 | — | |
| `cnc::no_check_missing` | 2 | written | `no_check ` in front of the statement |
| `cnc::numbered_enum_not_c` | 3 | — | |
| `cnc::opaque_struct_held` | 3 | — | |
| `cnc::pointer_target_wrong` | 2 | — | |
| `cnc::spread_into_c` | 1 | — | |
| `concat_needs_text` | 4 | — | |
| `const_not_known` | 5 | — | |
| `const_shadowed` | 4 | — | |
| `conversion_target_unsaid` | 1 | — | |
| `count_negative` | 1 | — | |
| `counter_shadowed` | 1 | — | |
| `data_missing` | 4 | — | |
| `duplicate_map_key` | 1 | — | |
| `element_given_away` | 1 | — | |
| `empty_map_literal` | 1 | — | |
| `empty_untyped` | 3 | — | |
| `failure_case_unhandled` | 1 | — | |
| `failure_handled_twice` | 1 | — | |
| `failure_never_happens` | 3 | — | |
| `failure_not_listed` | 4 | — | |
| `failure_unhandled` | 6 | — | |
| `false_satisfies_claim` | 1 | — | |
| `field_missing` | 1 | — | |
| `field_name_in_call` | 2 | — | |
| `field_unnamed` | 2 | — | |
| `file_not_readable` | 1 | — | |
| `format_holes_mismatch` | 3 | — | |
| `format_text_wrong` | 2 | — | |
| `function_shape_wrong` | 4 | — | |
| `generic_filled_inconsistently` | 2 | — | |
| `generic_size_unknown` | 1 | — | |
| `generic_unfilled` | 2 | — | |
| `given_twice` | 2 | — | |
| `handler_falls_through` | 1 | — | |
| `handler_value_unused` | 1 | — | |
| `include_not_const` | 1 | — | |
| `included_text_invalid` | 5 | — | |
| `index_out_of_range` | 3 | partly | check it before use: the statement under `if ($k at_least 1) and ($k at_most std::list.length(lend $xs)) {`, where the code is a named list's `[pos: …]`; not yet: a list that is not a name, a slice |
| `iterator_has_no_position` | 1 | — | |
| `iterator_next_form` | 1 | — | |
| `label_outside_call` | 4 | — | |
| `labels_out_of_order` | 1 | — | |
| `lend_mut_needed` | 8 | partly | `lend` / a verb → `lend.mut`; not yet: a borrow handed back, a `for` over an iterator |
| `lend_size_mismatch` | 1 | — | |
| `lent_given_away` | 2 | — | |
| `lent_twice_in_call` | 1 | — | |
| `list_array_mismatch` | 4 | — | |
| `loop_variable_typed` | 1 | — | |
| `main_cannot_fail` | 2 | — | |
| `map_entry_form` | 1 | — | |
| `match_needs_enum` | 1 | — | |
| `match_value_unused` | 1 | — | |
| `mcps::channel_form` | 5 | — | |
| `mcps::detached_lend` | 1 | — | |
| `mcps::send_needs_give` | 1 | — | |
| `mcps::task_returns` | 1 | — | |
| `mod_near_on_fraction` | 1 | — | |
| `money::bin_to_deci_unsaid` | 1 | — | |
| `money::deci_mixed` | 2 | — | |
| `money::exact_into_bin` | 1 | — | |
| `money::exactly_needed` | 1 | — | |
| `money::percent_mixed` | 3 | — | |
| `money::power_not_whole` | 2 | — | |
| `money::too_many_digits` | 2 | — | |
| `money::too_many_places` | 3 | — | |
| `must_misused` | 2 | — | |
| `mut_never_changed` | 1 | written | `mut` out of its declaration |
| `needs_integer` | 11 | — | |
| `negative_exponent` | 1 | — | |
| `negative_unsigned` | 2 | — | |
| `no_check_missing` | 5 | partly | `no_check ` in front of the statement; not yet: a format text not written in the call |
| `no_implicit_conversion` | 1 | partly | the side that converts without failing, `to.T(…)`, or the declared type the value's; not yet: a conversion that can fail (asked) |
| `no_such_conversion` | 8 | — | |
| `no_version_fits` | 1 | — | |
| `not_a_function` | 7 | — | |
| `not_a_struct` | 2 | — | |
| `not_a_value` | 7 | — | |
| `not_declared` | 8 | partly | the close name in scope; waiting on design claude: whether a declaration is offered (a variable's value is the writer's) |
| `not_indexable` | 2 | — | |
| `not_one_code_point` | 1 | — | |
| `not_text` | 4 | — | |
| `not_walkable` | 4 | — | |
| `nothing_to_give` | 2 | — | |
| `nothing_to_shadow` | 1 | — | |
| `operator_called_as_function` | 2 | — | |
| `operator_cannot_be_value` | 1 | — | |
| `operator_value_ambiguous` | 1 | — | |
| `operator_wrong_type` | 9 | — | |
| `otherwise_never_needed` | 10 | partly | the `otherwise` clause out; not yet: a list read |
| `outside_loop` | 2 | — | |
| `owned_value_bare` | 9 | partly | `give` / `copy` / `lend` in front, as the message lists; not yet: a `for`'s list, a spread, a conversion |
| `pointer_cast_target` | 2 | — | |
| `pointer_outside_boundary` | 1 | — | |
| `pointer_read_only` | 1 | — | |
| `position_unlabelled` | 7 | partly | `pos: ` in front; not yet: a slice's, an index in the parser |
| `print_form_unsaid` | 4 | partly | `.useful` / `.stored`; not yet: the `.weird` forms |
| `print_into_non_writer` | 1 | — | |
| `returned_borrow_form` | 5 | — | |
| `returned_borrow_misused` | 1 | — | |
| `returned_borrow_unsaid` | 2 | — | |
| `rounding_unsaid` | 1 | written | the way meant first, then each |
| `shadow_sealed` | 1 | — | |
| `shadow_unsaid` | 1 | written | `shadow` for `let` |
| `shuffle_not_fixed` | 2 | — | |
| `simd_lane_count_unknown` | 1 | — | |
| `size_unit_unsaid` | 1 | — | |
| `spread_length_unknown` | 1 | — | |
| `spread_not_variadic` | 1 | — | |
| `spread_outside_call` | 1 | — | |
| `spread_verb_wrong` | 4 | — | |
| `text_not_ascii` | 1 | — | |
| `text_order_undecided` | 2 | — | |
| `text_unit_unsaid` | 6 | partly | each unit; not yet: a `for` over a text, a text's `[…]` |
| `typed_literal_form` | 2 | — | |
| `unicode::no_single_code` | 2 | — | |
| `union_one_field` | 1 | — | |
| `unknown_case` | 2 | written | the close name |
| `unknown_field` | 2 | written | the close name |
| `unknown_operation` | 6 | — | |
| `unknown_pointer_operation` | 1 | — | |
| `unknown_rounding` | 3 | written | the five ways |
| `unknown_std_name` | 4 | — | |
| `unknown_struct` | 1 | — | |
| `unlabelled_after_labelled` | 1 | — | |
| `unordered_map_walked` | 1 | — | |
| `value_does_not_fit` | 11 | partly | a whole number declared too narrow: the narrowest type of its sign that holds it; not yet: worked-out values (bound where read, waits on the root) |
| `value_left_unused` | 4 | — | |
| `verb_on_fresh_value` | 4 | — | |
| `verb_wrong_for_parameter` | 14 | partly | the verb the parameter takes, or the verb out; not yet: C's pointers, a borrow handed back |
| `versions_overlap` | 4 | — | |
| `width_unknown` | 11 | — | |
| `wrap_sizes_mixed` | 1 | — | |
| `writer_size_zero` | 1 | — | |
| `writer_way_unknown` | 1 | — | |
| `written_not_literal` | 1 | — | |
| `wrong_argument_count` | 14 | — | |
| `wrong_type` | 51 | partly | a conversion that cannot fail, or the declared type the value's; not yet: a conversion that can fail (asked), values that are simply wrong |

## ownership and lends (2 names)

| name | sites | status | notes |
|---|---|---|---|
| `used_after_give` | 1 | partly | `give $s` → `copy $s` where it was given; none for a `for`'s source (an iterator is walked by giving it) or a give with no `give` word |
| `writer_unflushed` | 1 | written | the flush above the block's `}` |

## checks (Precog) (14 names)

| name | sites | status | notes |
|---|---|---|---|
| `bound_not_kept` | 1 | — | |
| `check_not_enough` | 1 | — | |
| `duration_negative` | 1 | — | |
| `float_too_large` | 1 | — | |
| `hash_disagrees_with_equal` | 1 | — | |
| `length_bound_not_kept` | 1 | — | |
| `mcps::send_never_taken` | 1 | — | |
| `mcps::task_outlives_lend` | 1 | — | |
| `mcps::tasks_wait_on_each_other` | 1 | — | |
| `no_check_lied` | 1 | written | the `no_check` taken out |
| `shift_loses_every_bit` | 1 | — | |
| `shift_overflows` | 1 | — | |
| `slice_out_of_range` | 1 | — | |
| `too_wide_in_between` | 1 | — | |

## kits, files and the manifest (40 names)

| name | sites | status | notes |
|---|---|---|---|
| `build_file_missing` | 1 | — | |
| `cnc::c_lib_name_form` | 2 | — | |
| `file_name_not_module` | 5 | — | |
| `kit_not_found` | 2 | — | |
| `kit_prefix_clash` | 4 | none suggested | the prefixes are the depended-on kits' (owner's example) |
| `kits_depend_in_circle` | 2 | none suggested | the order kits depend in is not one program's code (owner's example) |
| `limb_retired` | 3 | — | |
| `manifest_block_unclosed` | 2 | written | the `}` above the next line, or a last line |
| `manifest_default_unclear` | 3 | — | |
| `manifest_field_misplaced` | 1 | — | |
| `manifest_field_missing` | 6 | — | |
| `manifest_given_twice` | 7 | partly | the second line goes; not yet: a block given twice |
| `manifest_id_form` | 1 | — | |
| `manifest_kit_named_in_block` | 1 | written | the line goes |
| `manifest_limb_block_name` | 1 | — | |
| `manifest_line_form` | 6 | — | |
| `manifest_missing` | 2 | none suggested | the missing manifest is a depended-on kit's |
| `manifest_name_form` | 1 | — | |
| `manifest_section_form` | 2 | — | |
| `manifest_source_unclear` | 4 | — | |
| `manifest_unknown_field` | 4 | written | the close key |
| `manifest_unknown_section` | 1 | written | the close section name |
| `manifest_value_form` | 1 | — | |
| `manifest_value_quoted` | 1 | written | the quotes off |
| `manifest_version_too_long` | 1 | — | |
| `manifest_version_word` | 1 | written | `ver` |
| `name_ambiguous` | 2 | — | |
| `old_file_name` | 8 | — | |
| `operator_level_given_twice` | 1 | — | |
| `path_not_readable` | 3 | — | |
| `power_except_misplaced` | 1 | — | |
| `root_file_missing` | 2 | — | |
| `target_function_wrong` | 5 | — | |
| `target_layout_missing` | 3 | — | |
| `target_not_buildable` | 2 | — | |
| `target_not_listed` | 1 | — | |
| `unicode::case_marks_disagree` | 1 | — | |
| `unicode::names_differ_only_in_case` | 1 | — | |
| `unknown_power` | 1 | — | |
| `version_not_available` | 2 | — | |

## other: templates, uses, sources (12 names)

| name | sites | status | notes |
|---|---|---|---|
| `concise_form` | 1 | — | |
| `division_by_zero` | 1 | partly | check it before use: `if $b not_equal 0 {`, where the divisor is written plainly |
| `generated_code_stale` | 1 | written | what `kayo generate` writes, in place of what is there |
| `mcps::para_cannot_split` | 1 | written | `para` out: a sequential `for` |
| `missing_return` | 4 | — | |
| `no_truthiness` | 5 | — | |
| `step_zero` | 2 | written | a step of `1` |
| `template_runs_forever` | 1 | — | |
| `template_too_many_members` | 1 | — | |
| `template_value_unknown` | 4 | — | |
| `use_form` | 5 | partly | ` as NAME` after the path; a use of this kit's own path goes; not yet: a `use` below other lines (it moves), a missing `;` |
| `use_name_taken` | 1 | — | |
