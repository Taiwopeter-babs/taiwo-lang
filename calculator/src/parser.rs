use pest::{Parser, error::Error as PestError, iterators::Pair};
use pest_derive::Parser as PestDeriveParser;

use crate::ast::{Node, Operator};

#[derive(PestDeriveParser)]
#[grammar = "grammar.pest"]
struct CalcParser;

pub fn parse(source: &str) -> Result<Vec<Node>, PestError<Rule>> {
    let mut ast = vec![];

    let pairs = CalcParser::parse(Rule::Program, source)?;

    for pair in pairs {
        if let Rule::Expr = pair.as_rule() {
            ast.push(build_ast_from_expr(pair))
        }
    }

    Ok(ast)
}

fn build_ast_from_expr(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
        Rule::UnaryExpr => {
            let mut pair = pair.into_inner();

            let operator = pair.next().unwrap();

            let child = pair.next().unwrap();

            let child = build_ast_from_term(child);

            parse_unary_expr(operator, child)
        }
        Rule::BinaryExpr => {
            let mut pair = pair.into_inner();

            let lhs_pair = pair.next().unwrap();

            // The first element can be a UnaryExpr or Term rule
            let mut lhs = match lhs_pair.as_rule() {
                Rule::UnaryExpr => {
                    let mut inner = lhs_pair.into_inner();

                    let operator = inner.next().unwrap();

                    let child = inner.next().unwrap();

                    let child = build_ast_from_term(child);

                    parse_unary_expr(operator, child)
                }
                _ => build_ast_from_term(lhs_pair),
            };

            let operator = pair.next().unwrap();

            let rhs_pair = pair.next().unwrap();

            let mut rhs = build_ast_from_term(rhs_pair);

            let mut final_eval = parse_binary_expr(operator, lhs, rhs);

            loop {
                let pair_buffer = pair.next();

                if let Some(op) = pair_buffer {
                    lhs = final_eval;

                    rhs = build_ast_from_term(pair.next().unwrap());

                    final_eval = parse_binary_expr(op, lhs, rhs);
                } else {
                    return final_eval;
                }
            }
        }
        Rule::Term => build_ast_from_term(pair),
        unknown => panic!("Unknown expr: {:?}", unknown),
    }
}

fn build_ast_from_term(pair: Pair<Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::Term);

    let pair = pair.into_inner().next().unwrap();

    match pair.as_rule() {
        Rule::Int => {
            let int: i32 = pair.as_str().parse().unwrap();

            Node::Int(int)
        }

        Rule::Expr => build_ast_from_expr(pair),

        unknown => panic!("Unknown term: {:?}", unknown),
    }
}

fn parse_unary_expr(operator_pair: Pair<Rule>, child: Node) -> Node {
    let operator = match operator_pair.as_str() {
        "+" => Operator::Plus,
        "-" => Operator::Minus,
        unknown => panic!("Unknown operator {:?}", unknown),
    };

    Node::UnaryExpr {
        op: operator,
        child: Box::new(child),
    }
}

fn parse_binary_expr(operator_pair: Pair<Rule>, lhs: Node, rhs: Node) -> Node {
    let operator = match operator_pair.as_str() {
        "+" => Operator::Plus,
        "-" => Operator::Minus,
        unknown => panic!("Unknown operator {:?}", unknown),
    };

    Node::BinaryExpr {
        op: operator,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let one = parse("1");

        assert!(one.is_ok());

        assert_eq!(one.unwrap()[0], Node::Int(1));

        assert!(parse("b").is_err());
    }
}
