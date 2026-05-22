// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Julia the Viper - PEG Rule-Level Tests
// Tests every individual pest grammar rule in isolation.

use jtv_core::parser::{JtvParser, Rule};
use pest::Parser;

// ===== WHITESPACE & COMMENTS =====

#[test]
fn rule_line_comment() {
    let result = JtvParser::parse(Rule::line_comment, "// hello world");
    assert!(result.is_ok(), "Line comment should parse");
}

#[test]
fn rule_block_comment() {
    let result = JtvParser::parse(Rule::block_comment, "/* multi\nline\ncomment */");
    assert!(result.is_ok(), "Block comment should parse");
}

#[test]
fn rule_block_comment_empty() {
    let result = JtvParser::parse(Rule::block_comment, "/**/");
    assert!(result.is_ok(), "Empty block comment should parse");
}

// ===== IDENTIFIERS =====

#[test]
fn rule_identifier_simple() {
    let result = JtvParser::parse(Rule::identifier, "foo");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "foo");
}

#[test]
fn rule_identifier_with_underscore() {
    let result = JtvParser::parse(Rule::identifier, "my_var");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "my_var");
}

#[test]
fn rule_identifier_leading_underscore() {
    let result = JtvParser::parse(Rule::identifier, "_private");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "_private");
}

#[test]
fn rule_identifier_alphanumeric() {
    let result = JtvParser::parse(Rule::identifier, "x42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "x42");
}

#[test]
fn rule_identifier_rejects_keyword_module() {
    let result = JtvParser::parse(Rule::identifier, "module");
    assert!(result.is_err(), "Keywords must not parse as identifiers");
}

#[test]
fn rule_identifier_rejects_keyword_fn() {
    let result = JtvParser::parse(Rule::identifier, "fn");
    assert!(result.is_err(), "Keywords must not parse as identifiers");
}

#[test]
fn rule_identifier_rejects_keyword_return() {
    let result = JtvParser::parse(Rule::identifier, "return");
    assert!(result.is_err());
}

#[test]
fn rule_identifier_rejects_keyword_if() {
    let result = JtvParser::parse(Rule::identifier, "if");
    assert!(result.is_err());
}

#[test]
fn rule_identifier_rejects_keyword_while() {
    let result = JtvParser::parse(Rule::identifier, "while");
    assert!(result.is_err());
}

#[test]
fn rule_identifier_rejects_keyword_for() {
    let result = JtvParser::parse(Rule::identifier, "for");
    assert!(result.is_err());
}

#[test]
fn rule_identifier_rejects_keyword_true() {
    let result = JtvParser::parse(Rule::identifier, "true");
    assert!(result.is_err());
}

#[test]
fn rule_identifier_rejects_keyword_false() {
    let result = JtvParser::parse(Rule::identifier, "false");
    assert!(result.is_err());
}

#[test]
fn rule_identifier_rejects_digit_start() {
    let result = JtvParser::parse(Rule::identifier, "42abc");
    assert!(result.is_err(), "Identifiers must not start with digits");
}

// ===== NUMBER LITERALS =====

#[test]
fn rule_integer_positive() {
    let result = JtvParser::parse(Rule::integer, "42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "42");
}

#[test]
fn rule_integer_negative() {
    let result = JtvParser::parse(Rule::integer, "-7");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "-7");
}

#[test]
fn rule_integer_zero() {
    let result = JtvParser::parse(Rule::integer, "0");
    assert!(result.is_ok());
}

#[test]
fn rule_float_simple() {
    let result = JtvParser::parse(Rule::float, "3.14");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "3.14");
}

#[test]
fn rule_float_negative() {
    let result = JtvParser::parse(Rule::float, "-2.718");
    assert!(result.is_ok());
}

#[test]
fn rule_float_scientific() {
    let result = JtvParser::parse(Rule::float, "1.5e10");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "1.5e10");
}

#[test]
fn rule_float_scientific_negative_exponent() {
    let result = JtvParser::parse(Rule::float, "6.022e-23");
    assert!(result.is_ok());
}

#[test]
fn rule_float_scientific_uppercase() {
    let result = JtvParser::parse(Rule::float, "1.0E5");
    assert!(result.is_ok());
}

#[test]
fn rule_rational() {
    let result = JtvParser::parse(Rule::rational, "3/4");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "3/4");
}

#[test]
fn rule_rational_negative() {
    let result = JtvParser::parse(Rule::rational, "-1/2");
    assert!(result.is_ok());
}

#[test]
fn rule_hex() {
    let result = JtvParser::parse(Rule::hex, "0xFF");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "0xFF");
}

#[test]
fn rule_hex_lowercase() {
    let result = JtvParser::parse(Rule::hex, "0xdeadbeef");
    assert!(result.is_ok());
}

#[test]
fn rule_binary() {
    let result = JtvParser::parse(Rule::binary, "0b1010");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "0b1010");
}

#[test]
fn rule_binary_all_zeros() {
    let result = JtvParser::parse(Rule::binary, "0b0000");
    assert!(result.is_ok());
}

#[test]
fn rule_complex_real_and_imaginary() {
    let result = JtvParser::parse(Rule::complex, "3+4i");
    assert!(result.is_ok());
}

#[test]
fn rule_complex_imaginary_only() {
    let result = JtvParser::parse(Rule::complex, "5i");
    assert!(result.is_ok());
}

#[test]
fn rule_complex_float_parts() {
    let result = JtvParser::parse(Rule::complex, "1.5+2.5i");
    assert!(result.is_ok());
}

#[test]
fn rule_number_dispatches_all_variants() {
    // Integer
    assert!(JtvParser::parse(Rule::number, "42").is_ok());
    // Float
    assert!(JtvParser::parse(Rule::number, "3.14").is_ok());
    // Hex
    assert!(JtvParser::parse(Rule::number, "0xFF").is_ok());
    // Binary
    assert!(JtvParser::parse(Rule::number, "0b1010").is_ok());
    // Rational
    assert!(JtvParser::parse(Rule::number, "1/3").is_ok());
}

// ===== STRING LITERALS =====

#[test]
fn rule_string_simple() {
    let result = JtvParser::parse(Rule::string, "\"hello\"");
    assert!(result.is_ok());
}

#[test]
fn rule_string_empty() {
    let result = JtvParser::parse(Rule::string, "\"\"");
    assert!(result.is_ok());
}

#[test]
fn rule_string_with_escapes() {
    let result = JtvParser::parse(Rule::string, "\"hello\\nworld\"");
    assert!(result.is_ok());
}

#[test]
fn rule_string_with_tab_escape() {
    let result = JtvParser::parse(Rule::string, "\"col1\\tcol2\"");
    assert!(result.is_ok());
}

#[test]
fn rule_string_with_escaped_quote() {
    let result = JtvParser::parse(Rule::string, "\"say \\\"hi\\\"\"");
    assert!(result.is_ok());
}

#[test]
fn rule_string_with_escaped_backslash() {
    let result = JtvParser::parse(Rule::string, "\"path\\\\dir\"");
    assert!(result.is_ok());
}

// ===== TYPE ANNOTATIONS =====

#[test]
fn rule_basic_type_int() {
    let result = JtvParser::parse(Rule::basic_type, "Int");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_float() {
    let result = JtvParser::parse(Rule::basic_type, "Float");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_rational() {
    let result = JtvParser::parse(Rule::basic_type, "Rational");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_complex() {
    let result = JtvParser::parse(Rule::basic_type, "Complex");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_hex() {
    let result = JtvParser::parse(Rule::basic_type, "Hex");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_binary() {
    let result = JtvParser::parse(Rule::basic_type, "Binary");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_symbolic() {
    let result = JtvParser::parse(Rule::basic_type, "Symbolic");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_bool() {
    let result = JtvParser::parse(Rule::basic_type, "Bool");
    assert!(result.is_ok());
}

#[test]
fn rule_basic_type_string() {
    let result = JtvParser::parse(Rule::basic_type, "String");
    assert!(result.is_ok());
}

#[test]
fn rule_list_type() {
    let result = JtvParser::parse(Rule::list_type, "List<Int>");
    assert!(result.is_ok());
}

#[test]
fn rule_list_type_nested() {
    let result = JtvParser::parse(Rule::list_type, "List<List<Int>>");
    assert!(result.is_ok());
}

#[test]
fn rule_tuple_type() {
    let result = JtvParser::parse(Rule::tuple_type, "(Int, Float)");
    assert!(result.is_ok());
}

#[test]
fn rule_tuple_type_triple() {
    let result = JtvParser::parse(Rule::tuple_type, "(Int, Float, String)");
    assert!(result.is_ok());
}

#[test]
fn rule_function_type() {
    let result = JtvParser::parse(Rule::function_type, "Fn(Int, Int) -> Int");
    assert!(result.is_ok());
}

#[test]
fn rule_function_type_no_params() {
    let result = JtvParser::parse(Rule::function_type, "Fn() -> Int");
    assert!(result.is_ok());
}

// ===== DATA EXPRESSIONS =====

#[test]
fn rule_data_expr_single_number() {
    let result = JtvParser::parse(Rule::data_expr, "42");
    assert!(result.is_ok());
}

#[test]
fn rule_data_expr_addition() {
    let result = JtvParser::parse(Rule::data_expr, "1 + 2");
    assert!(result.is_ok());
}

#[test]
fn rule_data_expr_chained_addition() {
    let result = JtvParser::parse(Rule::data_expr, "1 + 2 + 3 + 4");
    assert!(result.is_ok());
}

#[test]
fn rule_data_expr_parenthesised() {
    let result = JtvParser::parse(Rule::data_expr, "(1 + 2) + 3");
    assert!(result.is_ok());
}

#[test]
fn rule_data_expr_negation() {
    let result = JtvParser::parse(Rule::data_expr, "-5");
    assert!(result.is_ok());
}

#[test]
fn rule_data_expr_identifier() {
    let result = JtvParser::parse(Rule::data_expr, "x");
    assert!(result.is_ok());
}

#[test]
fn rule_data_expr_function_call() {
    let result = JtvParser::parse(Rule::data_expr, "add(1, 2)");
    assert!(result.is_ok());
}

#[test]
fn rule_data_expr_qualified_call() {
    let result = JtvParser::parse(Rule::data_expr, "Math.add(1, 2)");
    assert!(result.is_ok());
}

#[test]
fn rule_list_literal_empty() {
    let result = JtvParser::parse(Rule::list_literal, "[]");
    assert!(result.is_ok());
}

#[test]
fn rule_list_literal_elements() {
    let result = JtvParser::parse(Rule::list_literal, "[1, 2, 3]");
    assert!(result.is_ok());
}

#[test]
fn rule_tuple_literal() {
    let result = JtvParser::parse(Rule::tuple_literal, "(10, 20)");
    assert!(result.is_ok());
}

#[test]
fn rule_tuple_literal_triple() {
    let result = JtvParser::parse(Rule::tuple_literal, "(1, 2, 3)");
    assert!(result.is_ok());
}

// ===== CONTROL EXPRESSIONS =====

#[test]
fn rule_comparator_eq() {
    let result = JtvParser::parse(Rule::comparator, "==");
    assert!(result.is_ok());
}

#[test]
fn rule_comparator_ne() {
    let result = JtvParser::parse(Rule::comparator, "!=");
    assert!(result.is_ok());
}

#[test]
fn rule_comparator_lt() {
    let result = JtvParser::parse(Rule::comparator, "<");
    assert!(result.is_ok());
}

#[test]
fn rule_comparator_le() {
    let result = JtvParser::parse(Rule::comparator, "<=");
    assert!(result.is_ok());
}

#[test]
fn rule_comparator_gt() {
    let result = JtvParser::parse(Rule::comparator, ">");
    assert!(result.is_ok());
}

#[test]
fn rule_comparator_ge() {
    let result = JtvParser::parse(Rule::comparator, ">=");
    assert!(result.is_ok());
}

#[test]
fn rule_comparison_expr() {
    let result = JtvParser::parse(Rule::comparison_expr, "x == 5");
    assert!(result.is_ok());
}

#[test]
fn rule_comparison_expr_less_than() {
    let result = JtvParser::parse(Rule::comparison_expr, "a < b");
    assert!(result.is_ok());
}

#[test]
fn rule_control_expr_with_comparison() {
    let result = JtvParser::parse(Rule::control_expr, "x > 0");
    assert!(result.is_ok());
}

#[test]
fn rule_control_expr_data_only() {
    let result = JtvParser::parse(Rule::control_expr, "42");
    assert!(result.is_ok());
}

// ===== CONTROL STATEMENTS =====

#[test]
fn rule_assignment() {
    let result = JtvParser::parse(Rule::assignment, "x = 5");
    assert!(result.is_ok());
}

#[test]
fn rule_assignment_expr() {
    let result = JtvParser::parse(Rule::assignment, "y = 1 + 2 + 3");
    assert!(result.is_ok());
}

#[test]
fn rule_if_stmt_simple() {
    let result = JtvParser::parse(Rule::if_stmt, "if x > 0 { y = 1 }");
    assert!(result.is_ok());
}

#[test]
fn rule_if_stmt_with_else() {
    let result = JtvParser::parse(Rule::if_stmt, "if x > 0 { y = 1 } else { y = 0 }");
    assert!(result.is_ok());
}

#[test]
fn rule_while_stmt() {
    let result = JtvParser::parse(Rule::while_stmt, "while x < 10 { x = x + 1 }");
    assert!(result.is_ok());
}

#[test]
fn rule_for_stmt() {
    let result = JtvParser::parse(Rule::for_stmt, "for i in 0..10 { x = x + i }");
    assert!(result.is_ok());
}

#[test]
fn rule_range_expr_simple() {
    let result = JtvParser::parse(Rule::range_expr, "0..10");
    assert!(result.is_ok());
}

#[test]
fn rule_range_expr_with_step() {
    let result = JtvParser::parse(Rule::range_expr, "0..10..2");
    assert!(result.is_ok());
}

#[test]
fn rule_return_stmt_with_value() {
    let result = JtvParser::parse(Rule::return_stmt, "return 42");
    assert!(result.is_ok());
}

#[test]
fn rule_return_stmt_empty() {
    let result = JtvParser::parse(Rule::return_stmt, "return");
    assert!(result.is_ok());
}

#[test]
fn rule_print_stmt_single() {
    let result = JtvParser::parse(Rule::print_stmt, "print(42)");
    assert!(result.is_ok());
}

#[test]
fn rule_print_stmt_multiple() {
    let result = JtvParser::parse(Rule::print_stmt, "print(1, 2, 3)");
    assert!(result.is_ok());
}

#[test]
fn rule_block() {
    let result = JtvParser::parse(Rule::block, "{ x = 1 }");
    assert!(result.is_ok());
}

#[test]
fn rule_block_empty() {
    let result = JtvParser::parse(Rule::block, "{ }");
    assert!(result.is_ok());
}

#[test]
fn rule_block_multiple_stmts() {
    let result = JtvParser::parse(Rule::block, "{ x = 1 y = 2 }");
    assert!(result.is_ok());
}

// ===== REVERSE BLOCKS =====

#[test]
fn rule_reverse_block() {
    let result = JtvParser::parse(Rule::reverse_block, "reverse { x += 5 }");
    assert!(result.is_ok());
}

#[test]
fn rule_reverse_block_multiple() {
    let result = JtvParser::parse(Rule::reverse_block, "reverse { x += 5 y -= 3 }");
    assert!(result.is_ok());
}

#[test]
fn rule_reversible_op_add() {
    let result = JtvParser::parse(Rule::reversible_op, "+=");
    assert!(result.is_ok());
}

#[test]
fn rule_reversible_op_sub() {
    let result = JtvParser::parse(Rule::reversible_op, "-=");
    assert!(result.is_ok());
}

#[test]
fn rule_reversible_assignment() {
    let result = JtvParser::parse(Rule::reversible_assignment, "counter += 10");
    assert!(result.is_ok());
}

// ===== FUNCTIONS =====

#[test]
fn rule_purity_marker_pure() {
    let result = JtvParser::parse(Rule::purity_marker, "@pure");
    assert!(result.is_ok());
}

#[test]
fn rule_purity_marker_total() {
    let result = JtvParser::parse(Rule::purity_marker, "@total");
    assert!(result.is_ok());
}

#[test]
fn rule_param_untyped() {
    let result = JtvParser::parse(Rule::param, "x");
    assert!(result.is_ok());
}

#[test]
fn rule_param_typed() {
    let result = JtvParser::parse(Rule::param, "x: Int");
    assert!(result.is_ok());
}

#[test]
fn rule_param_list_single() {
    let result = JtvParser::parse(Rule::param_list, "x: Int");
    assert!(result.is_ok());
}

#[test]
fn rule_param_list_multiple() {
    let result = JtvParser::parse(Rule::param_list, "x: Int, y: Float");
    assert!(result.is_ok());
}

#[test]
fn rule_function_decl_simple() {
    let result = JtvParser::parse(Rule::function_decl, "fn add(a: Int, b: Int): Int { return a + b }");
    assert!(result.is_ok());
}

#[test]
fn rule_function_decl_pure() {
    let result = JtvParser::parse(Rule::function_decl, "@pure fn double(x: Int): Int { return x + x }");
    assert!(result.is_ok());
}

#[test]
fn rule_function_decl_total() {
    let result = JtvParser::parse(Rule::function_decl, "@total fn identity(x: Int): Int { return x }");
    assert!(result.is_ok());
}

#[test]
fn rule_function_decl_no_params() {
    let result = JtvParser::parse(Rule::function_decl, "fn zero(): Int { return 0 }");
    assert!(result.is_ok());
}

#[test]
fn rule_function_decl_no_return_type() {
    let result = JtvParser::parse(Rule::function_decl, "fn greet(name) { print(name) }");
    assert!(result.is_ok());
}

#[test]
fn rule_function_call_no_args() {
    let result = JtvParser::parse(Rule::function_call, "zero()");
    assert!(result.is_ok());
}

#[test]
fn rule_function_call_with_args() {
    let result = JtvParser::parse(Rule::function_call, "add(1, 2)");
    assert!(result.is_ok());
}

#[test]
fn rule_qualified_name_simple() {
    let result = JtvParser::parse(Rule::qualified_name, "foo");
    assert!(result.is_ok());
}

#[test]
fn rule_qualified_name_dotted() {
    let result = JtvParser::parse(Rule::qualified_name, "Math.add");
    assert!(result.is_ok());
}

#[test]
fn rule_qualified_name_deeply_nested() {
    let result = JtvParser::parse(Rule::qualified_name, "Lib.Sub.func");
    assert!(result.is_ok());
}

// ===== MODULE & IMPORT =====

#[test]
fn rule_module_decl_empty() {
    let result = JtvParser::parse(Rule::module_decl, "module Empty { }");
    assert!(result.is_ok());
}

#[test]
fn rule_module_decl_with_function() {
    let result = JtvParser::parse(
        Rule::module_decl,
        "module Math { fn add(a: Int, b: Int): Int { return a + b } }",
    );
    assert!(result.is_ok());
}

#[test]
fn rule_import_stmt_simple() {
    let result = JtvParser::parse(Rule::import_stmt, "import Math");
    assert!(result.is_ok());
}

#[test]
fn rule_import_stmt_dotted() {
    let result = JtvParser::parse(Rule::import_stmt, "import Lib.Math");
    assert!(result.is_ok());
}

#[test]
fn rule_import_stmt_with_alias() {
    let result = JtvParser::parse(Rule::import_stmt, "import Math as M");
    assert!(result.is_ok());
}

#[test]
fn rule_module_path_single() {
    let result = JtvParser::parse(Rule::module_path, "Math");
    assert!(result.is_ok());
}

#[test]
fn rule_module_path_dotted() {
    let result = JtvParser::parse(Rule::module_path, "Lib.Math.Utils");
    assert!(result.is_ok());
}

// ===== PROGRAM (Top Level) =====

#[test]
fn rule_program_empty() {
    let result = JtvParser::parse(Rule::program, "");
    assert!(result.is_ok());
}

#[test]
fn rule_program_single_stmt() {
    let result = JtvParser::parse(Rule::program, "x = 42");
    assert!(result.is_ok());
}

#[test]
fn rule_program_mixed() {
    let code = r#"
        module M {
            fn f(x: Int): Int { return x }
        }
        import M
        y = M.f(10)
    "#;
    let result = JtvParser::parse(Rule::program, code);
    assert!(result.is_ok());
}

// ===== KEYWORD RULE =====

#[test]
fn rule_keyword_matches_all() {
    let keywords = [
        "module", "import", "as", "fn", "return", "if", "else", "while", "for", "in",
        "print", "reverse", "Int", "Float", "Rational", "Complex", "Hex", "Binary",
        "Symbolic", "Bool", "String", "List", "Fn", "true", "false",
    ];
    for kw in &keywords {
        let result = JtvParser::parse(Rule::keyword, kw);
        assert!(result.is_ok(), "Keyword '{}' should match", kw);
    }
}

// ===== UNARY OPERATOR =====

#[test]
fn rule_unary_op_negate() {
    let result = JtvParser::parse(Rule::unary_op, "-");
    assert!(result.is_ok());
}

#[test]
fn rule_unary_op_not() {
    let result = JtvParser::parse(Rule::unary_op, "!");
    assert!(result.is_ok());
}

// ===== LOGICAL EXPRESSIONS =====

#[test]
fn rule_logical_expr_or() {
    let result = JtvParser::parse(Rule::logical_expr, "x > 0 || y > 0");
    assert!(result.is_ok());
}

#[test]
fn rule_logical_term_and() {
    let result = JtvParser::parse(Rule::logical_term, "x > 0 && y < 10");
    assert!(result.is_ok());
}

#[test]
fn rule_logical_factor_not() {
    let result = JtvParser::parse(Rule::logical_factor, "!x > 0");
    assert!(result.is_ok());
}

// ===== CONTROL STATEMENT DISPATCH =====

#[test]
fn rule_control_stmt_assignment() {
    let result = JtvParser::parse(Rule::control_stmt, "x = 5");
    assert!(result.is_ok());
}

#[test]
fn rule_control_stmt_if() {
    let result = JtvParser::parse(Rule::control_stmt, "if x > 0 { y = 1 }");
    assert!(result.is_ok());
}

#[test]
fn rule_control_stmt_while() {
    let result = JtvParser::parse(Rule::control_stmt, "while x < 10 { x = x + 1 }");
    assert!(result.is_ok());
}

#[test]
fn rule_control_stmt_for() {
    let result = JtvParser::parse(Rule::control_stmt, "for i in 0..5 { x = x + i }");
    assert!(result.is_ok());
}

#[test]
fn rule_control_stmt_return() {
    let result = JtvParser::parse(Rule::control_stmt, "return 42");
    assert!(result.is_ok());
}

#[test]
fn rule_control_stmt_print() {
    let result = JtvParser::parse(Rule::control_stmt, "print(1, 2)");
    assert!(result.is_ok());
}

#[test]
fn rule_control_stmt_reverse() {
    let result = JtvParser::parse(Rule::control_stmt, "reverse { x += 1 }");
    assert!(result.is_ok());
}

#[test]
fn rule_control_stmt_block() {
    let result = JtvParser::parse(Rule::control_stmt, "{ x = 1 }");
    assert!(result.is_ok());
}
