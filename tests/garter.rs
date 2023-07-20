include!("examples.rs");

// Compile time errors
mk_fail_test!(
  garter_err_shadow_prim,
  "garter/compile_errs/err_shadow_prim.garter",
  "Attempting to shadow primitive type"
);
mk_fail_test!(
  garter_err_shadow_prim2,
  "garter/compile_errs/err_shadow_prim_2.garter",
  "Attempting to shadow primitive type"
);
mk_fail_test!(
  garter_err_shadow_prim3,
  "garter/compile_errs/err_shadow_prim_3.garter",
  "Attempting to shadow primitive type"
);
mk_fail_test!(
  garter_err_dup_type_decl,
  "garter/compile_errs/err_dup_type_decl.garter",
  "Custom type Some defined repeatedly"
);
mk_fail_test!(
  garter_err_undefined_type,
  "garter/compile_errs/err_undefined_type.garter",
  "Use of undefined type None"
);
mk_fail_test!(
  garter_err_wrong_type_arity,
  "garter/compile_errs/err_wrong_type_arity.garter",
  "Type Some expected arguments of size 1 but received 2"
);

mk_fail_test!(
  garter_err_dup_match_arm,
  "garter/compile_errs/err_dup_match_arm.garter",
  "Type Some used repeatedly in one match expression"
);

mk_fail_test!(
  garter_err_type_called_no_data,
  "garter/compile_errs/err_type_called_no_data.garter",
  "Type constructor for type Some was incorrectly called"
);

mk_fail_test!(
  garter_err_type_call_data,
  "garter/compile_errs/err_type_call_data.garter",
  "Type constructor for type None was incorrectly called"
);

mk_fail_test!(
  garter_err_dup_match_arm_args,
  "garter/compile_errs/err_dup_match_arm_args.garter",
  "Duplicate arguments in match arm"
);

mk_fail_test!(
  garter_err_type_call_as_function,
  "garter/compile_errs/err_type_call_as_function.garter",
  "Type constructor for type Foo was incorrectly called"
);

// Trivial tests
mk_test!(
  garter_custom_types,
  "garter/trivial_tests/custom_types.garter",
  "true"
);

mk_test!(
  garter_prim_types,
  "garter/trivial_tests/prim_types.garter",
  "6"
);

mk_test!(
  garter_prims_and_custom,
  "garter/trivial_tests/prims_and_custom.garter",
  "5"
);

mk_test!(
  garter_type_shadowing,
  "garter/trivial_tests/type_shadowing.garter",
  "1"
);

mk_test!(
  garter_print_type,
  "garter/trivial_tests/print_type.garter",
  "Some(2)\n0"
);

mk_test!(
  garter_equality_single_variant,
  "garter/trivial_tests/equality_single_variant.garter",
  "true"
);

mk_test!(
  garter_equality_mult_variant,
  "garter/trivial_tests/equality_mult_variant.garter",
  "false"
);

mk_test!(
  garter_equality_single_variant_shadowed,
  "garter/trivial_tests/equality_single_variant_shadowed.garter",
  "false"
);

// Non-trivial tests
mk_test!(
  garter_match_arr,
  "garter/non-trivial_tests/match_arr.garter",
  "[5, 5, 5]"
);

mk_test!(
  garter_match_if,
  "garter/non-trivial_tests/match_if.garter",
  "3"
);

mk_test!(
  garter_match_in_default,
  "garter/non-trivial_tests/match_in_default.garter",
  "10"
);

mk_test!(
  garter_nested_match,
  "garter/non-trivial_tests/nested_match.garter",
  "10"
);

mk_test!(
  garter_print_match,
  "garter/non-trivial_tests/print_match.garter",
  "5
5
5"
);

mk_test!(
  garter_type_func_return,
  "garter/non-trivial_tests/type_func_return.garter",
  "5"
);

mk_test!(
  garter_types_funcs,
  "garter/non-trivial_tests/types_funcs.garter",
  "1"
);

mk_test!(
  garter_equality_single_var_assign,
  "garter/non-trivial_tests/equality_single_var_assign.garter",
  "false"
);
