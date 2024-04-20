use crate::parser::ast::{
    Block, Declaration, Expression, InfixOperator, Literal, Parameter, Program, Statement,
    UnaryOperator,
};

#[derive(Default, Debug)]
struct JavascriptCompilationOutput {
    code: String,
    semicolon_allowed: bool,
    is_block: bool,
    evaluates_to: Option<String>,
}

impl From<&str> for JavascriptCompilationOutput {
    fn from(value: &str) -> Self {
        JavascriptCompilationOutput {
            code: value.to_string(),
            ..Default::default()
        }
    }
}

impl FromIterator<JavascriptCompilationOutput> for JavascriptCompilationOutput {
    fn from_iter<T: IntoIterator<Item = JavascriptCompilationOutput>>(iter: T) -> Self {
        let mut code = String::new();
        for output in iter {
            code.push_str(&output.code);
        }
        JavascriptCompilationOutput {
            code,
            ..Default::default()
        }
    }
}

trait JavascriptCompile {
    fn compile(&self) -> JavascriptCompilationOutput;
}

impl JavascriptCompile for Program {
    fn compile(&self) -> JavascriptCompilationOutput {
        self.iter().map(Statement::compile).collect()
    }
}

impl JavascriptCompile for Statement {
    fn compile(&self) -> JavascriptCompilationOutput {
        let statement = match self {
            Statement::ExpressionStatement {
                expression: expr, ..
            } => expr.compile(),
            Statement::DeclarationStatement(decl) => decl.compile(),
        };
        let code = build_block(&statement, false);
        JavascriptCompilationOutput {
            code: format!(
                "{}{}\n",
                code,
                if statement.semicolon_allowed { ";" } else { "" }
            ),
            ..Default::default()
        }
    }
}

fn build_block(block_output: &JavascriptCompilationOutput, eval: bool) -> String {
    if block_output.is_block {
        if let Some(evaluates_to) = block_output.evaluates_to.as_ref() {
            format!(
                "let return_value = undefined;\n{{\n{}return_value = {};\n}}{}",
                block_output.code,
                evaluates_to,
                eval.then_some(";\nreturn_value").unwrap_or_default()
            )
        } else {
            block_output.code.clone()
        }
    } else {
        block_output.code.clone()
    }
}

impl JavascriptCompile for Expression {
    fn compile(&self) -> JavascriptCompilationOutput {
        match self {
            Expression::IdentifierExpression(ident) => JavascriptCompilationOutput {
                code: ident.0.clone(),
                semicolon_allowed: true,
                ..Default::default()
            },
            Expression::LiteralExpression(literal) => literal.compile(),
            Expression::UnaryExpression(op, arg) => {
                let op = op.compile();
                let arg = arg.compile();
                JavascriptCompilationOutput {
                    code: format!("{}{}", op.code, arg.code),
                    semicolon_allowed: arg.semicolon_allowed,
                    ..Default::default()
                }
            }
            Expression::InfixExpression(op, arg0, arg1) => {
                let op = op.compile();
                let arg0 = arg0.compile();
                let arg1 = arg1.compile();
                JavascriptCompilationOutput {
                    code: format!("{} {} {}", arg0.code, op.code, arg1.code),
                    semicolon_allowed: arg1.semicolon_allowed,
                    ..Default::default()
                }
            }
            Expression::ArrayExpression(exprs) => {
                let exprs = exprs.compile();
                JavascriptCompilationOutput {
                    code: format!("[{}]", exprs.code),
                    semicolon_allowed: true,
                    ..Default::default()
                }
            }
            Expression::CallExpression(ident, args) => {
                let ident = ident.compile();
                let args = args.compile();
                JavascriptCompilationOutput {
                    code: format!("{}({})", ident.code, args.code),
                    semicolon_allowed: true,
                    ..Default::default()
                }
            }
            Expression::MemberAccessExpression(expr, ident) => {
                let expr = expr.compile();
                let ident = ident.0.clone();
                JavascriptCompilationOutput {
                    code: format!("{}.{}", expr.code, ident),
                    semicolon_allowed: true,
                    ..Default::default()
                }
            }
            Expression::IndexExpression(expr, index_expr) => {
                let expr = expr.compile();
                let expr = build_block(&expr, true);
                let index_expr = index_expr.compile();
                JavascriptCompilationOutput {
                    code: format!("{}[{}]", expr, index_expr.code),
                    semicolon_allowed: true,
                    ..Default::default()
                }
            }
            Expression::BlockExpression(block) => {
                let return_value = block.return_value.as_ref().map(Expression::compile);
                if block.statements.is_empty() {
                    return JavascriptCompilationOutput {
                        code: format!(
                            "let return_value = {};",
                            return_value
                                .map(|rv| rv.code)
                                .unwrap_or("undefined".to_string())
                        ),
                        ..Default::default()
                    };
                }
                let block = block.statements.compile();
                JavascriptCompilationOutput {
                    code: block.code,
                    semicolon_allowed: false,
                    is_block: true,
                    evaluates_to: return_value.map(|rv| rv.code),
                }
            }
        }
    }
}

impl JavascriptCompile for Literal {
    fn compile(&self) -> JavascriptCompilationOutput {
        match self {
            Literal::NumberLiteral(n) => JavascriptCompilationOutput {
                code: n.to_string(),
                semicolon_allowed: true,
                ..Default::default()
            },
            Literal::StringLiteral(s) => JavascriptCompilationOutput {
                code: format!("\"{}\"", s),
                semicolon_allowed: true,
                ..Default::default()
            },
            Literal::BooleanLiteral(b) => JavascriptCompilationOutput {
                code: b.to_string(),
                semicolon_allowed: true,
                ..Default::default()
            },
        }
    }
}

impl JavascriptCompile for UnaryOperator {
    fn compile(&self) -> JavascriptCompilationOutput {
        match self {
            UnaryOperator::Not => "!".into(),
            UnaryOperator::Minus => "-".into(),
            UnaryOperator::Plus => "+".into(),
        }
    }
}

impl JavascriptCompile for InfixOperator {
    fn compile(&self) -> JavascriptCompilationOutput {
        match self {
            InfixOperator::Equal => "==".into(),
            InfixOperator::NotEqual => "!=".into(),
            InfixOperator::GreaterThan => ">".into(),
            InfixOperator::LessThan => "<".into(),
            InfixOperator::GreaterThanEqual => ">=".into(),
            InfixOperator::LessThanEqual => "<=".into(),
            InfixOperator::Plus => "+".into(),
            InfixOperator::Minus => "-".into(),
            InfixOperator::Multiply => "*".into(),
            InfixOperator::Divide => "/".into(),
            InfixOperator::Modulo => "%".into(),
        }
    }
}

impl JavascriptCompile for Declaration {
    fn compile(&self) -> JavascriptCompilationOutput {
        match self {
            Declaration::ConstDeclaration(ident, expr) => {
                let expr = expr.compile();
                let ident = ident.0.clone();
                JavascriptCompilationOutput {
                    code: format!("const {} = {};", ident, expr.code),
                    ..Default::default()
                }
            }
            Declaration::LetDeclaration(ident, expr) => {
                let expr = expr.compile();
                let ident = ident.0.clone();
                JavascriptCompilationOutput {
                    code: format!("let {} = {};", ident, expr.code),
                    ..Default::default()
                }
            }
            Declaration::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                let parameters = parameters.compile();
                let name = name.0.clone();
                let body = body.compile();
                JavascriptCompilationOutput {
                    code: format!("function {}({}) {}", name, parameters.code, body.code),
                    ..Default::default()
                }
            }
        }
    }
}

impl JavascriptCompile for Vec<Parameter> {
    fn compile(&self) -> JavascriptCompilationOutput {
        let parameters = self.iter().map(Parameter::compile).collect::<Vec<_>>();
        JavascriptCompilationOutput {
            code: parameters
                .into_iter()
                .map(|p| p.code)
                .collect::<Vec<_>>()
                .join(", ")
                .to_string(),
            ..Default::default()
        }
    }
}

impl JavascriptCompile for Parameter {
    fn compile(&self) -> JavascriptCompilationOutput {
        JavascriptCompilationOutput {
            code: self.name.0.clone(),
            ..Default::default()
        }
    }
}

impl JavascriptCompile for Vec<Expression> {
    fn compile(&self) -> JavascriptCompilationOutput {
        let expressions = self.iter().map(Expression::compile).collect::<Vec<_>>();
        JavascriptCompilationOutput {
            code: expressions
                .into_iter()
                .map(|e| e.code)
                .collect::<Vec<_>>()
                .join(", ")
                .to_string(),
            ..Default::default()
        }
    }
}

impl JavascriptCompile for Block {
    fn compile(&self) -> JavascriptCompilationOutput {
        let statements = self
            .statements
            .iter()
            .map(|statement| statement.compile())
            .collect::<JavascriptCompilationOutput>();
        let return_value = self
            .return_value
            .as_ref()
            .map(|return_value| return_value.compile())
            .map(|return_value| format!("return {};\n", return_value.code))
            .unwrap_or("".into());
        JavascriptCompilationOutput {
            code: format!("{{\n{}{}}}", statements.code, return_value,),
            ..Default::default()
        }
    }
}

pub struct JavascriptCompiler;

impl JavascriptCompiler {
    pub fn compile(program: Program) -> String {
        let compiled = program.compile();
        compiled.code
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::{Identifier, Program};

    use super::*;

    #[test]
    fn literals() {
        let program: Program = vec![
            Statement::ExpressionStatement {
                expression: Expression::LiteralExpression(Literal::NumberLiteral("5".to_string())),
                has_semicolon: true,
            },
            Statement::ExpressionStatement {
                expression: Expression::LiteralExpression(Literal::StringLiteral(
                    "foo".to_string(),
                )),
                has_semicolon: true,
            },
            Statement::ExpressionStatement {
                expression: Expression::LiteralExpression(Literal::BooleanLiteral(true)),
                has_semicolon: true,
            },
        ];

        assert_eq!("5;\n\"foo\";\ntrue;\n".to_string(), program.compile().code);
    }

    #[test]
    fn expressions() {
        let program: Program = vec![
            Statement::ExpressionStatement {
                expression: Expression::IdentifierExpression(Identifier("test".to_string())),
                has_semicolon: true,
            },
            Statement::ExpressionStatement {
                expression: Expression::LiteralExpression(Literal::NumberLiteral("5".to_string())),
                has_semicolon: true,
            },
            Statement::ExpressionStatement {
                expression: Expression::UnaryExpression(
                    UnaryOperator::Minus,
                    Box::new(Expression::LiteralExpression(Literal::NumberLiteral(
                        "5".to_string(),
                    ))),
                ),
                has_semicolon: true,
            },
            Statement::ExpressionStatement {
                expression: Expression::InfixExpression(
                    InfixOperator::Plus,
                    Box::new(Expression::LiteralExpression(Literal::NumberLiteral(
                        "5".to_string(),
                    ))),
                    Box::new(Expression::LiteralExpression(Literal::NumberLiteral(
                        "5".to_string(),
                    ))),
                ),
                has_semicolon: true,
            },
            Statement::ExpressionStatement {
                expression: Expression::ArrayExpression(vec![
                    Expression::LiteralExpression(Literal::NumberLiteral("5".to_string())),
                    Expression::LiteralExpression(Literal::NumberLiteral("10".to_string())),
                ]),
                has_semicolon: true,
            },
        ];
        assert_eq!(
            "test;\n5;\n-5;\n5 + 5;\n[5, 10];\n".to_string(),
            program.compile().code
        );
    }

    #[test]
    fn declarations() {
        let program: Program = vec![
            Statement::DeclarationStatement(Declaration::ConstDeclaration(
                Identifier("test".to_string()),
                Expression::LiteralExpression(Literal::NumberLiteral("5".to_string())),
            )),
            Statement::DeclarationStatement(Declaration::LetDeclaration(
                Identifier("test".to_string()),
                Expression::LiteralExpression(Literal::NumberLiteral("5".to_string())),
            )),
            Statement::DeclarationStatement(Declaration::FunctionDeclaration {
                name: Identifier("test".to_string()),
                parameters: vec![],
                body: Block {
                    statements: vec![Statement::DeclarationStatement(
                        Declaration::LetDeclaration(
                            Identifier("test".to_string()),
                            Expression::LiteralExpression(Literal::NumberLiteral("5".to_string())),
                        ),
                    )],
                    return_value: None,
                },
            }),
            Statement::DeclarationStatement(Declaration::FunctionDeclaration {
                name: Identifier("test".to_string()),
                parameters: vec![
                    Parameter {
                        name: Identifier("foo".to_string()),
                        type_: Identifier("string".to_string()),
                    },
                    Parameter {
                        name: Identifier("bar".to_string()),
                        type_: Identifier("number".to_string()),
                    },
                ],
                body: Block {
                    statements: vec![Statement::DeclarationStatement(
                        Declaration::LetDeclaration(
                            Identifier("baz".to_string()),
                            Expression::LiteralExpression(Literal::NumberLiteral("5".to_string())),
                        ),
                    )],
                    return_value: Some(Expression::IdentifierExpression(Identifier(
                        "baz".to_string(),
                    ))),
                },
            }),
        ];
        assert_eq!(
            "const test = 5;\nlet test = 5;\nfunction test() {\nlet test = 5;\n}\nfunction test(foo, bar) {\nlet baz = 5;\nreturn baz;\n}\n".to_string(),
            program.compile().code
        );
    }

    #[test]
    fn code_snippet() {
        let program: Program = vec![
            Statement::DeclarationStatement(Declaration::FunctionDeclaration {
                name: Identifier("foo".into()),
                parameters: vec![
                    Parameter {
                        name: Identifier("bar".into()),
                        type_: Identifier("number".into()),
                    },
                    Parameter {
                        name: Identifier("baz".into()),
                        type_: Identifier("number".into()),
                    },
                ],
                body: Block {
                    statements: vec![],
                    return_value: Some(Expression::InfixExpression(
                        InfixOperator::Plus,
                        Box::new(Expression::IdentifierExpression(Identifier("bar".into()))),
                        Box::new(Expression::IdentifierExpression(Identifier("baz".into()))),
                    )),
                },
            }),
            Statement::ExpressionStatement {
                expression: Expression::CallExpression(
                    Box::new(Expression::IdentifierExpression(Identifier("foo".into()))),
                    vec![
                        Expression::LiteralExpression(Literal::NumberLiteral("20".into())),
                        Expression::InfixExpression(
                            InfixOperator::Minus,
                            Box::new(Expression::LiteralExpression(Literal::NumberLiteral(
                                "30".into(),
                            ))),
                            Box::new(Expression::LiteralExpression(Literal::NumberLiteral(
                                "2".into(),
                            ))),
                        ),
                    ],
                ),
                has_semicolon: true,
            },
        ];
        // TODO: add function call expression and more

        assert_eq!(
            "function foo(bar, baz) {\nreturn bar + baz;\n}\nfoo(20, 30 - 2);\n".to_string(),
            program.compile().code
        );
    }

    #[test]
    fn block_expression_without_statements() {
        let program: Program = vec![Statement::ExpressionStatement {
            expression: Expression::BlockExpression(Box::new(Block {
                statements: vec![],
                return_value: Some(Expression::LiteralExpression(Literal::NumberLiteral(
                    "5".into(),
                ))),
            })),
            has_semicolon: true,
        }];

        assert_eq!(
            "let return_value = 5;\n".to_string(),
            program.compile().code
        );
    }

    #[test]
    fn block_expression_with_statements() {
        let program: Program = vec![Statement::ExpressionStatement {
            expression: Expression::BlockExpression(Box::new(Block {
                statements: vec![Statement::DeclarationStatement(
                    Declaration::ConstDeclaration(
                        Identifier("foo".into()),
                        Expression::LiteralExpression(Literal::NumberLiteral("5".into())),
                    ),
                )],
                return_value: Some(Expression::IdentifierExpression(Identifier("foo".into()))),
            })),
            has_semicolon: true,
        }];

        assert_eq!(
            "let return_value = undefined;\n{\nconst foo = 5;\nreturn_value = foo;\n}\n"
                .to_string(),
            program.compile().code
        );
    }
}
