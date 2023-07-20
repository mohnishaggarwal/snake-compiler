include!("examples.rs");
mk_fail_test!(
  adder_parse_error,
  "adder/parse_error.adder",
  "Error parsing input"
);
/* Error produced by interpreter:
Error parsing input: Unrecognized EOF found at 7
Expected one of "!", "(", "[", "add1", "def", "false", "if", "isarray", "isbool", "isfun", "isnum", "lambda", "length", "let", "print", "sub1", "true", "λ", r#"[+-]?[0-9]+"# or r#"[a-zA-Z][a-zA-Z0-9_]*"#
*/
mk_fail_test!(
  boa_comprehensive,
  "boa/comprehensive.boa",
  "if expected a boolean"
);
/* Error produced by interpreter:
Error in interpreter: if expected a boolean, but got 1 in if
*/
mk_fail_test!(
  boa_let_and_branch,
  "boa/let_and_branch.boa",
  "if expected a boolean"
);
/* Error produced by interpreter:
Error in interpreter: if expected a boolean, but got 0 in if
*/
mk_fail_test!(
  boa_multiple_ifs,
  "boa/multiple_ifs.boa",
  "if expected a boolean"
);
/* Error produced by interpreter:
Error in interpreter: if expected a boolean, but got 0 in if
*/
mk_fail_test!(
  boa_simple_branch,
  "boa/simple_branch.boa",
  "if expected a boolean"
);
/* Error produced by interpreter:
Error in interpreter: if expected a boolean, but got 2 in if
*/
mk_fail_test!(
  cobra_err_arith_overflow,
  "cobra/err_arith_overflow.cobra",
  "overflow"
);
/* Error produced by interpreter:
Error in interpreter: Operation 4611686018427387903 + 1 = 4611686018427387904 overflowed
*/
mk_fail_test!(
  cobra_err_arith_type,
  "cobra/err_arith_type.cobra",
  "arithmetic expected a number, but got"
);
/* Error produced by interpreter:
Error in interpreter: arithmetic expected a number, but got true in +
*/
mk_fail_test!(
  cobra_err_logic_type,
  "cobra/err_logic_type.cobra",
  "logic expected a boolean"
);
/* Error produced by interpreter:
Error in interpreter: logic expected a boolean, but got 2 in ||
*/
mk_fail_test!(
  cobra_err_multiply_overflow,
  "cobra/err_multiply_overflow.cobra",
  "overflow"
);
/* Error produced by interpreter:
Error in interpreter: Operation -2305843009213693952 * 3 = -6917529027641081856 overflowed
*/
mk_fail_test!(
  cobra_err_overflow,
  "cobra/err_overflow.cobra",
  "Number literal 4611686018427387904 doesn't fit into 63-bit integer at"
);
/* Error produced by interpreter:
Error generating assembly: Number literal 4611686018427387904 doesn't fit into 63-bit integer at line 1, column 0 to line 1, column 19
*/
mk_fail_test!(
  cobra_logic_error_short_circuit,
  "cobra/logic_error_short_circuit.cobra",
  "logic expected a boolean"
);
/* Error produced by interpreter:
Error in interpreter: logic expected a boolean, but got 1 in ||
*/
mk_fail_test!(
  diamond_err_dup_func,
  "diamond/err_dup_func.diamond",
  "Multiple defined functions named \"func\" at"
);
/* Error produced by interpreter:
Error generating assembly: multiple defined functions named "func" at line 1, column 0 to line 5, column 6
*/
mk_fail_test!(
  diamond_err_duplicate_arg,
  "diamond/err_duplicate_arg.diamond",
  "Multiple arguments named \"a\" at"
);
/* Error produced by interpreter:
Error generating assembly: multiple arguments named "a" at line 1, column 0 to line 2, column 13
*/
mk_fail_test!(
  diamond_err_duplicate_binding,
  "diamond/err_duplicate_binding.diamond",
  "Variable a defined twice in let-expression at"
);
/* Error produced by interpreter:
Error generating assembly: Variable a defined twice in let-expression at line 1, column 0 to line 2, column 1
*/
mk_fail_test!(
  diamond_err_duplicate_funcname,
  "diamond/err_duplicate_funcname.diamond",
  "Multiple defined functions named \"f\" at"
);
/* Error produced by interpreter:
Error generating assembly: multiple defined functions named "f" at line 1, column 0 to line 5, column 3
*/
mk_fail_test!(
  diamond_err_func_as_value,
  "diamond/err_func_as_value.diamond",
  "arithmetic expected a number, but got"
);
/* Error produced by interpreter:
Error in interpreter: arithmetic expected a number, but got closure in add1
*/
mk_fail_test!(
  diamond_err_func_overload,
  "diamond/err_func_overload.diamond",
  "Multiple defined functions named \"func\" at"
);
/* Error produced by interpreter:
Error generating assembly: multiple defined functions named "func" at line 1, column 0 to line 5, column 6
*/
mk_fail_test!(
  diamond_err_unbound_variable,
  "diamond/err_unbound_variable.diamond",
  "Unbound variable a at"
);
/* Error produced by interpreter:
Error generating assembly: Unbound variable a at line 7, column 0 to line 7, column 1
*/
mk_fail_test!(
  diamond_err_undefined_func,
  "diamond/err_undefined_func.diamond",
  "Unbound variable f at"
);
/* Error produced by interpreter:
Error generating assembly: Unbound variable f at line 2, column 0 to line 2, column 1
*/
mk_fail_test!(
  diamond_err_value_as_func,
  "diamond/err_value_as_func.diamond",
  "a closure"
);
/* Error produced by interpreter:
Error in interpreter: Function application expected a closure, but got 1
*/
mk_fail_test!(
  diamond_err_wrong_arity,
  "diamond/err_wrong_arity.diamond",
  "arguments"
);
/* Error produced by interpreter:
Error in interpreter: Function expecting 1 arguments called with 2 arguments
*/
mk_fail_test!(
  egg_err_call_nonfunction,
  "egg/err_call_nonfunction.egg",
  "a closure"
);
/* Error produced by interpreter:
Error in interpreter: Function application expected a closure, but got 2
*/
mk_fail_test!(
  egg_err_index_nonarray,
  "egg/err_index_nonarray.egg",
  "index"
);
/* Error produced by interpreter:
Error in interpreter: Expected an array but got 1 in array index
*/
mk_fail_test!(
  egg_err_length_of_nonarray,
  "egg/err_length_of_nonarray.egg",
  "length"
);
/* Error produced by interpreter:
Error in interpreter: Expected an array but got true in length
*/
mk_fail_test!(
  egg_err_let_recursive_binding,
  "egg/err_let_recursive_binding.egg",
  "Unbound variable arr at"
);
/* Error produced by interpreter:
Error generating assembly: Unbound variable arr at line 1, column 11 to line 1, column 14
*/
mk_fail_test!(
  egg_err_malformed_index,
  "egg/err_malformed_index.egg",
  "a number"
);
/* Error produced by interpreter:
Error in interpreter: index expected a number, but got array in
*/
mk_fail_test!(
  egg_err_out_of_bounds,
  "egg/err_out_of_bounds.egg",
  "index out of bounds"
);
/* Error produced by interpreter:
Error in interpreter: Array index out of bounds
*/
mk_fail_test!(
  egg_err_out_of_bounds_negative,
  "egg/err_out_of_bounds_negative.egg",
  "index out of bounds"
);
/* Error produced by interpreter:
Error in interpreter: Array index out of bounds
*/
mk_test!(adder_add1, "adder/add1.adder", "233");
mk_test!(adder_chained_prim1, "adder/chained_prim1.adder", "232");
mk_test!(adder_comprehensive, "adder/comprehensive.adder", "13");
mk_test!(adder_let_multi, "adder/let_multi.adder", "1");
mk_test!(adder_let_then_add1, "adder/let_then_add1.adder", "2");
mk_test!(adder_number, "adder/number.adder", "233");
mk_test!(adder_simple_let, "adder/simple_let.adder", "1");
mk_test!(adder_some_lets, "adder/some_lets.adder", "2");
mk_test!(boa_arithmetics, "boa/arithmetics.boa", "147");
mk_test!(boa_lots_of_add, "boa/lots_of_add.boa", "10");
mk_test!(boa_simple_add, "boa/simple_add.boa", "3");
mk_test!(cobra_arith1, "cobra/arith1.cobra", "true");
mk_test!(cobra_arith2, "cobra/arith2.cobra", "10");
mk_test!(cobra_comparison, "cobra/comparison.cobra", "true");
mk_test!(
  cobra_err_negative_overflow,
  "cobra/err_negative_overflow.cobra",
  "-4611686018427387904"
);
mk_test!(cobra_logic1, "cobra/logic1.cobra", "false");
mk_test!(cobra_logic2, "cobra/logic2.cobra", "true");
mk_test!(
  cobra_logic3,
  "cobra/logic3.cobra",
  "true
false
false
false"
);
mk_test!(
  cobra_multiply_no_overflow,
  "cobra/multiply_no_overflow.cobra",
  "-4611686018427387904"
);
mk_test!(
  cobra_no_overflow,
  "cobra/no_overflow.cobra",
  "4611686018427387903"
);
mk_test!(
  cobra_print_bool,
  "cobra/print_bool.cobra",
  "true
true"
);
mk_test!(
  cobra_short_circuit,
  "cobra/short_circuit.cobra",
  "true
false"
);
mk_test!(
  cobra_typecheck_functions,
  "cobra/typecheck_functions.cobra",
  "false
false
false
false
false"
);
mk_test!(
  diamond_double_print,
  "diamond/double_print.diamond",
  "1
1
2"
);
mk_test!(
  diamond_eager,
  "diamond/eager.diamond",
  "2
1"
);
mk_test!(
  diamond_fib,
  "diamond/fib.diamond",
  "1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
1
1
2
1
3
1
1
2
5
13
1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
21
1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
1
1
2
1
3
1
1
2
5
13
34
1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
1
1
2
1
3
1
1
2
5
13
1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
21
55
1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
1
1
2
1
3
1
1
2
5
13
1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
21
1
1
2
1
3
1
1
2
5
1
1
2
1
3
8
1
1
2
1
3
1
1
2
5
13
34
89"
);
mk_test!(diamond_func_shadow, "diamond/func_shadow.diamond", "1");
mk_test!(
  diamond_func_shadows_var,
  "diamond/func_shadows_var.diamond",
  "2"
);
mk_test!(
  diamond_func_with_one_param,
  "diamond/func_with_one_param.diamond",
  "6"
);
mk_test!(
  diamond_func_with_params_nontail,
  "diamond/func_with_params_nontail.diamond",
  "252"
);
mk_test!(
  diamond_isPrime,
  "diamond/isPrime.diamond",
  "true
false
false"
);
mk_test!(diamond_max, "diamond/max.diamond", "80");
mk_test!(diamond_mult, "diamond/mult.diamond", "15");
mk_test!(
  diamond_multiple_params,
  "diamond/multiple_params.diamond",
  "1
2
3
4
5
6
7
28"
);
mk_test!(
  diamond_mutual_recursion,
  "diamond/mutual_recursion.diamond",
  "false"
);
mk_test!(
  diamond_nest_funcs_name_collide,
  "diamond/nest_funcs_name_collide.diamond",
  "1"
);
mk_test!(
  diamond_notMultiplesOf,
  "diamond/notMultiplesOf.diamond",
  "true"
);
mk_test!(
  diamond_primes,
  "diamond/primes.diamond",
  "2
3
5
7
11
13
17
19
23
29
31
37
41
43
47
53
59
61
67
71
73
79
83
89
97
101
103
107
109
113
127
131
137
139
149
151
157
163
167
173
179
181
191
193
197
199
211
223
227
229
233
239
241
251
257
263
269
271
277
281
283
293
307
311
313
317
331
337
347
349
353
359
367
373
379
383
389
397
401
409
419
421
431
433
439
443
449
457
461
463
467
479
487
491
499
true"
);
mk_test!(
  diamond_print_func_retval,
  "diamond/print_func_retval.diamond",
  "false
false"
);
mk_test!(
  diamond_print_inside_func,
  "diamond/print_inside_func.diamond",
  "1
2
3"
);
mk_test!(
  diamond_print_tail,
  "diamond/print_tail.diamond",
  "1
1"
);
mk_test!(
  diamond_series_of_funcs,
  "diamond/series_of_funcs.diamond",
  "1"
);
mk_test!(diamond_simple_func, "diamond/simple_func.diamond", "1");
mk_test!(
  diamond_simple_loop,
  "diamond/simple_loop.diamond",
  "0
1
2
3
4
5
5"
);
mk_test!(
  diamond_simple_non_tail_func,
  "diamond/simple_non_tail_func.diamond",
  "2"
);
mk_test!(diamond_simple_tail, "diamond/simple_tail.diamond", "1");
mk_test!(diamond_two_funcs, "diamond/two_funcs.diamond", "1");
mk_test!(diamond_weird, "diamond/weird.diamond", "4");
mk_test!(
  egg_arr_equality,
  "egg/arr_equality.egg",
  "false
true
false
false
true
true"
);
mk_test!(
  egg_arr_get,
  "egg/arr_get.egg",
  "0
[5, <loop>, [[0, 1, 2, 3], true, false, true, false], [10, 11], true, 3]
3
false
true
[10, 11]
[10, 11]"
);
mk_test!(
  egg_arr_length,
  "egg/arr_length.egg",
  "0
2
4"
);
mk_test!(egg_arr_mutate, "egg/arr_mutate.egg", "[9, 2, 3]");
mk_test!(
  egg_arr_semantics,
  "egg/arr_semantics.egg",
  "[[1, 4], [1, 4]]
[1, 4]"
);
mk_test!(egg_capturing_lambda, "egg/capturing_lambda.egg", "3");
mk_test!(
  egg_interesting,
  "egg/interesting.egg",
  "[2, [3, [5, [7, [11, [13, [17, [19, [23, [29, []]]]]]]]]]]"
);
mk_test!(
  egg_loop_print_variant1,
  "egg/loop_print_variant1.egg",
  "1
2
3
4
false"
);
mk_test!(
  egg_loop_print_variant2,
  "egg/loop_print_variant2.egg",
  "1
2
3
4
[]"
);
mk_test!(
  egg_loop_print_variant3,
  "egg/loop_print_variant3.egg",
  "1
2
3
4
[]"
);
mk_test!(
  egg_loop_print_variant4,
  "egg/loop_print_variant4.egg",
  "1
2
3
4
[]"
);
mk_test!(
  egg_mutual_recursive_array,
  "egg/mutual_recursive_array.egg",
  "[0, [true, 1]]
[0, [true, 1]]
[[0, <loop>], 1]
[[0, <loop>], 1]"
);
mk_test!(
  egg_nested_func_definition,
  "egg/nested_func_definition.egg",
  "6"
);
mk_test!(
  egg_print_closure,
  "egg/print_closure.egg",
  "<closure>
<closure>"
);
mk_test!(
  egg_recursive_array,
  "egg/recursive_array.egg",
  "[0, <loop>]"
);
mk_test!(
  egg_rtti,
  "egg/rtti.egg",
  "true
false
false
false
true
false
false
false
true
false
false
false
0"
);
mk_test!(egg_shadow_self, "egg/shadow_self.egg", "2");
mk_test!(egg_simple_array, "egg/simple_array.egg", "[1, 2, 3]");
mk_test!(egg_simple_lambda, "egg/simple_lambda.egg", "1");
mk_test!(egg_zero, "egg/zero.egg", "0");
