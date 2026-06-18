// @taxonomy: compiler/parser
// Parser implementation using Pest
use crate::ast::*;
use crate::error::{JtvError, Result};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct JtvParser;

pub fn parse_program(input: &str) -> Result<Program> {
    let mut pairs = JtvParser::parse(Rule::program, input)
        .map_err(|e| JtvError::ParseError(format!("Parse error: {}", e)))?;

    let program_pair = pairs
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected program".to_string()))?;

    let mut statements = Vec::new();

    for pair in program_pair.into_inner() {
        match pair.as_rule() {
            Rule::module_decl => statements.push(parse_module(pair)?),
            Rule::import_stmt => statements.push(parse_import(pair)?),
            Rule::extern_coproc_block => {
                statements.push(TopLevel::ExternCoproc(parse_extern_coproc_block(pair)?))
            }
            Rule::function_decl => statements.push(parse_function(pair)?),
            Rule::control_stmt => statements.push(TopLevel::Control(parse_control_stmt(pair)?)),
            Rule::EOI => break,
            _ => {}
        }
    }

    Ok(Program { statements })
}

fn parse_module(pair: pest::iterators::Pair<Rule>) -> Result<TopLevel> {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected module name".to_string()))?
        .as_str()
        .to_string();

    let mut body = Vec::new();
    for pair in inner {
        match pair.as_rule() {
            Rule::function_decl => body.push(parse_function(pair)?),
            Rule::control_stmt => body.push(TopLevel::Control(parse_control_stmt(pair)?)),
            _ => {}
        }
    }

    Ok(TopLevel::Module(ModuleDecl { name, body }))
}

fn parse_import(pair: pest::iterators::Pair<Rule>) -> Result<TopLevel> {
    let mut inner = pair.into_inner();
    let module_path = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected module path in import".to_string()))?;

    let path: Vec<String> = module_path
        .into_inner()
        .map(|p| p.as_str().to_string())
        .collect();

    let alias = inner.next().map(|p| p.as_str().to_string());

    Ok(TopLevel::Import(ImportStmt { path, alias }))
}

fn parse_function(pair: pest::iterators::Pair<Rule>) -> Result<TopLevel> {
    let mut inner = pair.into_inner();

    let mut purity = Purity::Impure;
    let mut echo_annotation: Option<Echo> = None;
    let mut first = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected function declaration".to_string()))?;

    // Consume any leading markers — `@pure`/`@total` and `@echo(...)` — in any order.
    loop {
        match first.as_rule() {
            Rule::purity_marker => {
                purity = match first.as_str() {
                    "@pure" => Purity::Pure,
                    "@total" => Purity::Total,
                    _ => Purity::Impure,
                };
            }
            Rule::echo_marker => {
                echo_annotation = parse_echo_grade(first.clone());
            }
            _ => break,
        }
        first = inner.next().ok_or_else(|| {
            JtvError::ParseError("Expected function name after marker".to_string())
        })?;
    }

    let name = first.as_str().to_string();

    let mut params = Vec::new();
    let mut return_type = None;
    let mut body = Vec::new();

    for pair in inner {
        match pair.as_rule() {
            Rule::param_list => {
                for param_pair in pair.into_inner() {
                    params.push(parse_param(param_pair)?);
                }
            }
            Rule::return_type => {
                return_type = Some(parse_type_annotation(
                    pair.into_inner().next().ok_or_else(|| {
                        JtvError::ParseError("Expected type annotation in return type".to_string())
                    })?,
                )?);
            }
            Rule::block => {
                for stmt_pair in pair.into_inner() {
                    body.push(parse_control_stmt(stmt_pair)?);
                }
            }
            _ => {}
        }
    }

    Ok(TopLevel::Function(FunctionDecl {
        name,
        params,
        return_type,
        purity,
        echo_annotation,
        body,
    }))
}

/// Extract the Echo grade from an `@echo(...)` marker pair (ADR-0009 D1).
fn parse_echo_grade(marker: pest::iterators::Pair<Rule>) -> Option<Echo> {
    marker.into_inner().next().map(|g| match g.as_str() {
        "Neutral" => Echo::Neutral,
        "Breaking" => Echo::Breaking,
        _ => Echo::Safe,
    })
}

fn parse_param(pair: pest::iterators::Pair<Rule>) -> Result<Param> {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected parameter name".to_string()))?
        .as_str()
        .to_string();
    let type_annotation = inner.next().map(|p| parse_type_annotation(p)).transpose()?;

    Ok(Param {
        name,
        type_annotation,
    })
}

fn parse_control_stmt(pair: pest::iterators::Pair<Rule>) -> Result<ControlStmt> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected control statement".to_string()))?;

    match inner.as_rule() {
        Rule::assignment => {
            let mut parts = inner.into_inner();
            let target = parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected assignment target".to_string()))?
                .as_str()
                .to_string();
            let value_pair = parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected assignment value".to_string()))?;

            let value = if value_pair.as_rule() == Rule::data_expr {
                Expr::Data(parse_data_expr(value_pair)?)
            } else {
                Expr::Control(parse_control_expr(value_pair)?)
            };

            Ok(ControlStmt::Assignment(Assignment { target, value }))
        }
        Rule::if_stmt => {
            let mut parts = inner.into_inner();
            let condition = parse_control_expr(
                parts
                    .next()
                    .ok_or_else(|| JtvError::ParseError("Expected if condition".to_string()))?,
            )?;

            let then_block = parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected if then block".to_string()))?;
            let mut then_branch = Vec::new();
            for stmt in then_block.into_inner() {
                then_branch.push(parse_control_stmt(stmt)?);
            }

            let else_branch = parts
                .next()
                .map(|else_block| {
                    let mut stmts = Vec::new();
                    for stmt in else_block.into_inner() {
                        stmts.push(parse_control_stmt(stmt)?);
                    }
                    Ok::<Vec<ControlStmt>, JtvError>(stmts)
                })
                .transpose()?;

            Ok(ControlStmt::If(IfStmt {
                condition,
                then_branch,
                else_branch,
            }))
        }
        Rule::while_stmt => {
            let mut parts = inner.into_inner();
            let condition =
                parse_control_expr(parts.next().ok_or_else(|| {
                    JtvError::ParseError("Expected while condition".to_string())
                })?)?;

            let mut body = Vec::new();
            for stmt in parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected while body".to_string()))?
                .into_inner()
            {
                body.push(parse_control_stmt(stmt)?);
            }

            Ok(ControlStmt::While(WhileStmt { condition, body }))
        }
        Rule::for_stmt => {
            let mut parts = inner.into_inner();
            let variable = parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected for loop variable".to_string()))?
                .as_str()
                .to_string();
            let iterable = parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected for loop iterable".to_string()))?;

            // Support both range expressions (0..10) and data expressions (xs)
            let range = if iterable.as_rule() == Rule::range_expr {
                parse_range_expr(iterable)?
            } else {
                // Data expression used as iterable — wrap as a range with
                // the expression as start and a sentinel Unit end
                let expr = parse_data_expr(iterable)?;
                RangeExpr {
                    start: Box::new(expr),
                    end: Box::new(DataExpr::Number(Number::Int(0))),
                    step: None,
                }
            };

            let mut body = Vec::new();
            for stmt in parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected for loop body".to_string()))?
                .into_inner()
            {
                body.push(parse_control_stmt(stmt)?);
            }

            Ok(ControlStmt::For(ForStmt {
                variable,
                range,
                body,
            }))
        }
        Rule::return_stmt => {
            let value = inner
                .into_inner()
                .next()
                .map(|p| parse_data_expr(p))
                .transpose()?;
            Ok(ControlStmt::Return(value))
        }
        Rule::print_stmt => {
            let mut exprs = Vec::new();
            for expr_pair in inner.into_inner() {
                exprs.push(parse_data_expr(expr_pair)?);
            }
            Ok(ControlStmt::Print(exprs))
        }
        Rule::reverse_block => {
            let mut body = Vec::new();
            for stmt_pair in inner.into_inner() {
                body.push(parse_reversible_stmt(stmt_pair)?);
            }
            Ok(ControlStmt::ReverseBlock(ReverseBlock { body }))
        }
        Rule::reversible_block => {
            // `reversible { reversible_stmt* } ( -> identifier )?`
            // Body statements are `reversible_stmt`; a trailing `identifier`
            // (if present) is the token binding.
            let mut body = Vec::new();
            let mut token_binding = None;
            for sub in inner.into_inner() {
                match sub.as_rule() {
                    Rule::reversible_stmt => body.push(parse_reversible_stmt(sub)?),
                    Rule::identifier => token_binding = Some(sub.as_str().to_string()),
                    other => {
                        return Err(JtvError::ParseError(format!(
                            "Unexpected item in reversible block: {:?}",
                            other
                        )))
                    }
                }
            }
            Ok(ControlStmt::ReversibleBlock(ReversibleBlockStmt {
                body,
                token_binding,
            }))
        }
        Rule::reverse_token => {
            let tok = inner
                .into_inner()
                .next()
                .ok_or_else(|| {
                    JtvError::ParseError("Expected reversal token name after `reverse`".to_string())
                })?
                .as_str()
                .to_string();
            Ok(ControlStmt::ReverseToken(tok))
        }
        Rule::abandon_token => {
            let tok = inner
                .into_inner()
                .next()
                .ok_or_else(|| {
                    JtvError::ParseError("Expected reversal token name after `abandon`".to_string())
                })?
                .as_str()
                .to_string();
            Ok(ControlStmt::AbandonToken(tok))
        }
        Rule::block => {
            let mut stmts = Vec::new();
            for stmt_pair in inner.into_inner() {
                stmts.push(parse_control_stmt(stmt_pair)?);
            }
            Ok(ControlStmt::Block(stmts))
        }
        _ => Err(JtvError::ParseError(format!(
            "Unexpected control statement: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_reversible_stmt(pair: pest::iterators::Pair<Rule>) -> Result<ReversibleStmt> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected reversible statement".to_string()))?;

    match inner.as_rule() {
        Rule::reversible_assignment => {
            let mut parts = inner.into_inner();
            let target = parts
                .next()
                .ok_or_else(|| {
                    JtvError::ParseError("Expected reversible assignment target".to_string())
                })?
                .as_str()
                .to_string();
            let op = parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected reversible operator".to_string()))?
                .as_str();
            let expr = parse_data_expr(parts.next().ok_or_else(|| {
                JtvError::ParseError("Expected reversible assignment expression".to_string())
            })?)?;

            match op {
                "+=" => Ok(ReversibleStmt::AddAssign(target, expr)),
                "-=" => Ok(ReversibleStmt::SubAssign(target, expr)),
                _ => Err(JtvError::ParseError(format!(
                    "Invalid reversible operator: {}",
                    op
                ))),
            }
        }
        Rule::if_stmt => {
            // Parse as regular if statement
            let mut parts = inner.into_inner();
            let condition = parse_control_expr(parts.next().ok_or_else(|| {
                JtvError::ParseError("Expected reversible if condition".to_string())
            })?)?;

            let then_block = parts.next().ok_or_else(|| {
                JtvError::ParseError("Expected reversible if then block".to_string())
            })?;
            let mut then_branch = Vec::new();
            for stmt in then_block.into_inner() {
                then_branch.push(parse_control_stmt(stmt)?);
            }

            let else_branch = parts
                .next()
                .map(|else_block| {
                    let mut stmts = Vec::new();
                    for stmt in else_block.into_inner() {
                        stmts.push(parse_control_stmt(stmt)?);
                    }
                    Ok::<Vec<ControlStmt>, JtvError>(stmts)
                })
                .transpose()?;

            Ok(ReversibleStmt::If(IfStmt {
                condition,
                then_branch,
                else_branch,
            }))
        }
        _ => Err(JtvError::ParseError(format!(
            "Unexpected reversible statement: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_data_expr(pair: pest::iterators::Pair<Rule>) -> Result<DataExpr> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected data expression".to_string()))?;
    parse_additive_expr(inner)
}

fn parse_additive_expr(pair: pest::iterators::Pair<Rule>) -> Result<DataExpr> {
    let mut inner = pair.into_inner();
    let mut left =
        parse_term(inner.next().ok_or_else(|| {
            JtvError::ParseError("Expected additive expression term".to_string())
        })?)?;

    while let Some(right_pair) = inner.next() {
        let right = parse_term(right_pair)?;
        left = DataExpr::add(left, right);
    }

    Ok(left)
}

fn parse_term(pair: pest::iterators::Pair<Rule>) -> Result<DataExpr> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected term".to_string()))?;
    parse_factor(inner)
}

fn parse_factor(pair: pest::iterators::Pair<Rule>) -> Result<DataExpr> {
    match pair.as_rule() {
        Rule::number => {
            let num = parse_number(
                pair.into_inner()
                    .next()
                    .ok_or_else(|| JtvError::ParseError("Expected number".to_string()))?,
            )?;
            Ok(DataExpr::Number(num))
        }
        Rule::string => {
            // String literal: extract the inner content (without quotes)
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected string content".to_string()))?;
            Ok(DataExpr::StringLit(inner.as_str().to_string()))
        }
        Rule::identifier => Ok(DataExpr::Identifier(pair.as_str().to_string())),
        Rule::function_call => {
            let mut parts = pair.into_inner();
            let qualified_name = parts
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected function name".to_string()))?;

            // Parse qualified name: Module.submodule.function
            let name_parts: Vec<String> = qualified_name
                .into_inner()
                .map(|p| p.as_str().to_string())
                .collect();

            let (module, name) = if name_parts.len() > 1 {
                // Has module path: ["Module", "submod", "func"] -> module=["Module", "submod"], name="func"
                let last = name_parts.len() - 1;
                (Some(name_parts[..last].to_vec()), name_parts[last].clone())
            } else {
                // No module path
                (None, name_parts[0].clone())
            };

            let mut args = Vec::new();
            if let Some(arg_list) = parts.next() {
                for arg in arg_list.into_inner() {
                    args.push(parse_data_expr(arg)?);
                }
            }

            Ok(DataExpr::FunctionCall(FunctionCall { module, name, args }))
        }
        Rule::list_literal => {
            let mut elements = Vec::new();
            for elem in pair.into_inner() {
                elements.push(parse_data_expr(elem)?);
            }
            Ok(DataExpr::List(elements))
        }
        Rule::tuple_literal => {
            let mut elements = Vec::new();
            for elem in pair.into_inner() {
                elements.push(parse_data_expr(elem)?);
            }
            Ok(DataExpr::Tuple(elements))
        }
        Rule::factor => {
            let mut inner = pair.into_inner();
            let first = inner
                .next()
                .ok_or_else(|| JtvError::ParseError("Expected factor".to_string()))?;

            if first.as_rule() == Rule::unary_op {
                let op = first.as_str();
                let expr = parse_factor(inner.next().ok_or_else(|| {
                    JtvError::ParseError("Expected factor after unary operator".to_string())
                })?)?;

                match op {
                    "-" => Ok(DataExpr::Negate(Box::new(expr))),
                    _ => Err(JtvError::ParseError(format!(
                        "Unknown unary operator: {}",
                        op
                    ))),
                }
            } else {
                parse_factor(first)
            }
        }
        Rule::data_expr => parse_data_expr(pair),
        _ => Err(JtvError::ParseError(format!(
            "Unexpected factor: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_number(pair: pest::iterators::Pair<Rule>) -> Result<Number> {
    match pair.as_rule() {
        Rule::integer => {
            let n = pair
                .as_str()
                .parse::<i64>()
                .map_err(|e| JtvError::ParseError(format!("Invalid integer: {}", e)))?;
            Ok(Number::Int(n))
        }
        Rule::float => {
            let n = pair
                .as_str()
                .parse::<f64>()
                .map_err(|e| JtvError::ParseError(format!("Invalid float: {}", e)))?;
            Ok(Number::Float(n))
        }
        Rule::rational => {
            let parts: Vec<&str> = pair.as_str().split('/').collect();
            let num = parts[0]
                .parse::<i64>()
                .map_err(|e| JtvError::ParseError(format!("Invalid rational numerator: {}", e)))?;
            let den = parts[1].parse::<i64>().map_err(|e| {
                JtvError::ParseError(format!("Invalid rational denominator: {}", e))
            })?;
            Ok(Number::Rational(num, den))
        }
        Rule::complex => {
            // Simplified complex parsing
            let s = pair.as_str();
            if s.ends_with('i') {
                let real_part = &s[..s.len() - 1];
                if let Some(plus_pos) = real_part.rfind('+') {
                    let real = real_part[..plus_pos].parse::<f64>().unwrap_or(0.0);
                    let imag = real_part[plus_pos + 1..].parse::<f64>().unwrap_or(0.0);
                    Ok(Number::Complex(real, imag))
                } else {
                    let imag = real_part.parse::<f64>().unwrap_or(0.0);
                    Ok(Number::Complex(0.0, imag))
                }
            } else {
                Err(JtvError::ParseError("Invalid complex number".to_string()))
            }
        }
        Rule::hex => Ok(Number::Hex(pair.as_str().to_string())),
        Rule::binary => Ok(Number::Binary(pair.as_str().to_string())),
        Rule::symbolic => {
            // Strip the '#' prefix to get the symbolic name
            let name = pair.as_str().trim_start_matches('#').to_string();
            Ok(Number::Symbolic(name))
        }
        _ => Err(JtvError::ParseError(format!(
            "Unknown number type: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_control_expr(pair: pest::iterators::Pair<Rule>) -> Result<ControlExpr> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected control expression".to_string()))?;

    match inner.as_rule() {
        Rule::logical_expr => parse_logical_expr(inner),
        Rule::comparison_expr => parse_comparison_expr(inner),
        Rule::data_expr => Ok(ControlExpr::Data(parse_data_expr(inner)?)),
        _ => Err(JtvError::ParseError(format!(
            "Unexpected control expression: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_logical_expr(pair: pest::iterators::Pair<Rule>) -> Result<ControlExpr> {
    let mut inner = pair.into_inner();
    let mut left = parse_logical_term(
        inner
            .next()
            .ok_or_else(|| JtvError::ParseError("Expected logical term".to_string()))?,
    )?;

    for right_pair in inner {
        let right = parse_logical_term(right_pair)?;
        left = ControlExpr::Logical(Box::new(left), LogicalOp::Or, Box::new(right));
    }

    Ok(left)
}

fn parse_logical_term(pair: pest::iterators::Pair<Rule>) -> Result<ControlExpr> {
    let mut inner = pair.into_inner();
    let mut left = parse_logical_factor(
        inner
            .next()
            .ok_or_else(|| JtvError::ParseError("Expected logical factor".to_string()))?,
    )?;

    for right_pair in inner {
        let right = parse_logical_factor(right_pair)?;
        left = ControlExpr::Logical(Box::new(left), LogicalOp::And, Box::new(right));
    }

    Ok(left)
}

fn parse_logical_factor(pair: pest::iterators::Pair<Rule>) -> Result<ControlExpr> {
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected logical factor".to_string()))?;

    match first.as_str() {
        "!" => {
            let expr = parse_logical_factor(inner.next().ok_or_else(|| {
                JtvError::ParseError("Expected expression after negation".to_string())
            })?)?;
            Ok(ControlExpr::Not(Box::new(expr)))
        }
        _ => match first.as_rule() {
            Rule::comparison_expr => parse_comparison_expr(first),
            Rule::data_expr => Ok(ControlExpr::Data(parse_data_expr(first)?)),
            Rule::control_expr => parse_control_expr(first),
            _ => parse_logical_factor(first),
        },
    }
}

fn parse_comparison_expr(pair: pest::iterators::Pair<Rule>) -> Result<ControlExpr> {
    let mut inner = pair.into_inner();
    let left =
        parse_data_expr(inner.next().ok_or_else(|| {
            JtvError::ParseError("Expected comparison left operand".to_string())
        })?)?;
    let op = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected comparison operator".to_string()))?
        .as_str();
    let right =
        parse_data_expr(inner.next().ok_or_else(|| {
            JtvError::ParseError("Expected comparison right operand".to_string())
        })?)?;

    let comparator = match op {
        "==" => Comparator::Eq,
        "!=" => Comparator::Ne,
        "<" => Comparator::Lt,
        "<=" => Comparator::Le,
        ">" => Comparator::Gt,
        ">=" => Comparator::Ge,
        _ => return Err(JtvError::ParseError(format!("Unknown comparator: {}", op))),
    };

    Ok(ControlExpr::Comparison(
        Box::new(left),
        comparator,
        Box::new(right),
    ))
}

fn parse_range_expr(pair: pest::iterators::Pair<Rule>) -> Result<RangeExpr> {
    let mut inner = pair.into_inner();
    let start =
        Box::new(parse_data_expr(inner.next().ok_or_else(|| {
            JtvError::ParseError("Expected range start".to_string())
        })?)?);
    let end =
        Box::new(parse_data_expr(inner.next().ok_or_else(|| {
            JtvError::ParseError("Expected range end".to_string())
        })?)?);
    let step = inner
        .next()
        .map(|p| parse_data_expr(p))
        .transpose()?
        .map(Box::new);

    Ok(RangeExpr { start, end, step })
}

fn parse_type_annotation(pair: pest::iterators::Pair<Rule>) -> Result<TypeAnnotation> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| JtvError::ParseError("Expected type annotation".to_string()))?;

    match inner.as_rule() {
        Rule::basic_type => {
            let type_str = inner.as_str();
            let basic = match type_str {
                "Int" => BasicType::Int,
                "Float" => BasicType::Float,
                "Rational" => BasicType::Rational,
                "Complex" => BasicType::Complex,
                "Hex" => BasicType::Hex,
                "Binary" => BasicType::Binary,
                "Symbolic" => BasicType::Symbolic,
                "Bool" => BasicType::Bool,
                "String" => BasicType::String,
                _ => return Err(JtvError::ParseError(format!("Unknown type: {}", type_str))),
            };
            Ok(TypeAnnotation::Basic(basic))
        }
        Rule::list_type => {
            let elem_type =
                parse_type_annotation(inner.into_inner().next().ok_or_else(|| {
                    JtvError::ParseError("Expected list element type".to_string())
                })?)?;
            Ok(TypeAnnotation::List(Box::new(elem_type)))
        }
        Rule::tuple_type => {
            let mut types = Vec::new();
            for type_pair in inner.into_inner() {
                types.push(parse_type_annotation(type_pair)?);
            }
            Ok(TypeAnnotation::Tuple(types))
        }
        Rule::function_type => {
            let parts = inner.into_inner();
            let mut param_types = Vec::new();

            // Collect all but the last (which is return type)
            let all_types: Vec<_> = parts.collect();
            let return_type = all_types
                .last()
                .ok_or_else(|| JtvError::ParseError("Expected function return type".to_string()))?;

            for i in 0..all_types.len() - 1 {
                param_types.push(parse_type_annotation(all_types[i].clone())?);
            }

            let ret = Box::new(parse_type_annotation(return_type.clone())?);
            Ok(TypeAnnotation::Function(param_types, ret))
        }
        _ => Err(JtvError::ParseError(format!(
            "Unknown type annotation: {:?}",
            inner.as_rule()
        ))),
    }
}

// ──────────────────────────────────────────────
// extern coproc block parser
// ──────────────────────────────────────────────

fn parse_extern_coproc_block(pair: pest::iterators::Pair<Rule>) -> Result<ExternCoprocBlock> {
    let mut inner = pair.into_inner();
    let gate_name = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("expected gate name in extern coproc".into()))?
        .as_str()
        .to_string();

    let mut items = Vec::new();
    for item_pair in inner {
        if item_pair.as_rule() == Rule::coproc_item {
            items.push(parse_coproc_item(item_pair)?);
        }
    }

    Ok(ExternCoprocBlock {
        gate_name,
        items,
        resolved: None,
    })
}

fn parse_coproc_item(pair: pest::iterators::Pair<Rule>) -> Result<CoprocItem> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| JtvError::ParseError("expected coproc item".into()))?;
    match inner.as_rule() {
        Rule::intrinsic_decl => Ok(CoprocItem::Intrinsic(parse_intrinsic_decl(inner)?)),
        Rule::insn_decl => Ok(CoprocItem::Insn(parse_insn_decl(inner)?)),
        r => Err(JtvError::ParseError(format!(
            "unexpected coproc item rule: {r:?}"
        ))),
    }
}

fn parse_intrinsic_decl(pair: pest::iterators::Pair<Rule>) -> Result<CoprocIntrinsic> {
    let mut inner = pair.into_inner();
    let mut purity = Purity::Pure; // intrinsics default to pure
    let mut first = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("expected intrinsic declaration".into()))?;

    if first.as_rule() == Rule::purity_marker {
        purity = match first.as_str() {
            "@pure" => Purity::Pure,
            "@total" => Purity::Total,
            _ => Purity::Impure,
        };
        first = inner.next().ok_or_else(|| {
            JtvError::ParseError("expected intrinsic name after purity marker".into())
        })?;
    }

    let name = first.as_str().to_string();
    let mut params = Vec::new();
    let mut return_type = TypeAnnotation::Basic(BasicType::Int);

    for pair in inner {
        match pair.as_rule() {
            Rule::param_list => {
                for p in pair.into_inner() {
                    params.push(parse_param(p)?);
                }
            }
            Rule::return_type => {
                return_type = parse_type_annotation(
                    pair.into_inner()
                        .next()
                        .ok_or_else(|| JtvError::ParseError("expected return type".into()))?,
                )?;
            }
            _ => {}
        }
    }

    Ok(CoprocIntrinsic {
        name,
        params,
        return_type,
        purity,
    })
}

fn parse_insn_decl(pair: pest::iterators::Pair<Rule>) -> Result<CoprocInsn> {
    let mut inner = pair.into_inner();
    let mut purity = Purity::Pure;
    let mut first = inner
        .next()
        .ok_or_else(|| JtvError::ParseError("expected insn declaration".into()))?;

    if first.as_rule() == Rule::purity_marker {
        purity = match first.as_str() {
            "@pure" => Purity::Pure,
            "@total" => Purity::Total,
            _ => Purity::Impure,
        };
        first = inner
            .next()
            .ok_or_else(|| JtvError::ParseError("expected insn name after purity marker".into()))?;
    }

    let name = first.as_str().to_string();
    let mut params = Vec::new();
    let mut return_type = TypeAnnotation::Basic(BasicType::Int);
    let mut encoding: Option<String> = None;

    for pair in inner {
        match pair.as_rule() {
            Rule::param_list => {
                for p in pair.into_inner() {
                    params.push(parse_param(p)?);
                }
            }
            Rule::return_type => {
                return_type = parse_type_annotation(
                    pair.into_inner()
                        .next()
                        .ok_or_else(|| JtvError::ParseError("expected return type".into()))?,
                )?;
            }
            Rule::encoding_clause => {
                let enc_pair = pair
                    .into_inner()
                    .next()
                    .ok_or_else(|| JtvError::ParseError("expected encoding string".into()))?;
                // String literal — strip quotes via the inner string_inner rule.
                let raw = enc_pair
                    .into_inner()
                    .next()
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_default();
                encoding = Some(raw);
            }
            _ => {}
        }
    }

    Ok(CoprocInsn {
        name,
        params,
        return_type,
        purity,
        encoding,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_addition() {
        let code = "x = 5 + 3";
        let result = parse_program(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function() {
        let code = r#"
            fn add(a: Int, b: Int): Int {
                return a + b
            }
        "#;
        let result = parse_program(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pure_function() {
        let code = r#"
            @pure fn square(x: Int): Int {
                return x + x
            }
        "#;
        let result = parse_program(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_echo_annotation() {
        let program = parse_program("@echo(Neutral) fn f(): Int { return 0 }").unwrap();
        match &program.statements[0] {
            TopLevel::Function(func) => assert_eq!(func.echo_annotation, Some(Echo::Neutral)),
            _ => panic!("expected a function"),
        }
    }

    #[test]
    fn test_parse_echo_with_purity_either_order() {
        for code in [
            "@pure @echo(Breaking) fn f(): Int { return 0 }",
            "@echo(Breaking) @pure fn f(): Int { return 0 }",
        ] {
            let program = parse_program(code).unwrap();
            match &program.statements[0] {
                TopLevel::Function(func) => {
                    assert_eq!(func.purity, Purity::Pure);
                    assert_eq!(func.echo_annotation, Some(Echo::Breaking));
                }
                _ => panic!("expected a function"),
            }
        }
    }

    #[test]
    fn test_parse_symbolic_literal() {
        let code = "x = #pi + #e";
        let result = parse_program(code);
        assert!(
            result.is_ok(),
            "Failed to parse symbolic literals: {:?}",
            result.err()
        );
        let program = result.unwrap();
        // Verify the AST contains symbolic numbers
        if let TopLevel::Control(ControlStmt::Assignment(ref assign)) = program.statements[0] {
            if let Expr::Data(DataExpr::Add(ref left, ref right)) = assign.value {
                assert_eq!(**left, DataExpr::Number(Number::Symbolic("pi".to_string())));
                assert_eq!(**right, DataExpr::Number(Number::Symbolic("e".to_string())));
            } else {
                panic!("Expected Add expression");
            }
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_symbolic_with_underscore() {
        let code = "y = #my_var";
        let result = parse_program(code);
        assert!(
            result.is_ok(),
            "Failed to parse symbolic with underscore: {:?}",
            result.err()
        );
    }
}
