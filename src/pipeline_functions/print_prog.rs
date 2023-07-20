use crate::syntax::{Exp, ImmExp, Prim1, Prim2, SeqExp, SnakeType};

pub fn print_prog<Span>(expr: &Exp<Span>, indent: usize) -> String
where
    Span: Clone,
{
    format!(
        "{}{}",
        " ".repeat(indent),
        match expr {
            Exp::Num(n, _) => format!("{}", n),
            Exp::Bool(b, _) => format!("{}", b),
            Exp::Var(v, _) => format!("{}", v),
            Exp::Prim1(op, a, _) => format!(
                "{}({})",
                match op {
                    Prim1::Add1 => "add1",
                    Prim1::Sub1 => "sub1",
                    Prim1::Not => "!",
                    Prim1::Print => "print",
                    Prim1::IsBool => "isbool",
                    Prim1::IsNum => "isnum",
                    Prim1::Length => "length",
                    Prim1::IsArray => "isarray",
                    Prim1::IsFun => "isfun",
                },
                print_prog(a, 0)
            ),
            Exp::Prim2(op, a, b, _) => match op {
                Prim2::ArrayGet => format!("{}[{}]", print_prog(a, 0), print_prog(b, 0)),
                _ => {
                    format!(
                        "{} {} {}",
                        print_prog(a, 0),
                        match op {
                            Prim2::Add => "+",
                            Prim2::Sub => "-",
                            Prim2::Mul => "*",
                            Prim2::And => "&&",
                            Prim2::Or => "||",
                            Prim2::Lt => "<",
                            Prim2::Gt => ">",
                            Prim2::Le => "<=",
                            Prim2::Ge => ">=",
                            Prim2::Eq => "==",
                            Prim2::Neq => "!=",
                            Prim2::ArrayGet => unreachable!(),
                        },
                        print_prog(b, 0)
                    )
                }
            },
            Exp::Let {
                bindings,
                body,
                ann: _,
            } => {
                format!(
                    "let {} in\n{}",
                    bindings
                        .iter()
                        .map(|(name, expr)| format!("{} = {}", name, print_prog(expr, 0)))
                        .collect::<Vec<String>>()
                        .join(", "),
                    print_prog(body, indent)
                )
            }
            Exp::If {
                cond,
                thn,
                els,
                ann: _,
            } => format!(
                "if {}:\n{}\n{}else:\n{}",
                print_prog(cond, 0),
                print_prog(thn, indent + 2),
                " ".repeat(indent),
                print_prog(els, indent + 2)
            ),
            Exp::Array(elts, _) => format!(
                "[{}]",
                elts.iter()
                    .map(|elt| print_prog(elt, 0))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Exp::ArraySet {
                array,
                index,
                new_value,
                ann: _,
            } => format!(
                "{}[{}] := {}",
                print_prog(array, 0),
                print_prog(index, 0),
                print_prog(new_value, 0)
            ),
            Exp::Semicolon { e1, e2, ann: _ } => {
                format!("{};\n{}", print_prog(e1, 0), print_prog(e2, indent))
            }
            Exp::FunDefs {
                decls,
                body,
                ann: _,
            } => format!(
                "{}\nin\n{}",
                decls
                    .iter()
                    .map(|decl| format!(
                        "def {}({}):\n{}",
                        decl.name,
                        decl.parameters.join(", "),
                        print_prog(&decl.body, indent + 2)
                    ))
                    .collect::<Vec<String>>()
                    .join("\nand\n"),
                print_prog(body, indent)
            ),
            Exp::Call(callee, params, _) => format!(
                "{}({})",
                print_prog(callee, 0),
                params
                    .iter()
                    .map(|param| print_prog(param, 0))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Exp::Lambda {
                parameters,
                body,
                ann: _,
            } => format!(
                "lambda {}: {} end",
                parameters.join(", "),
                print_prog(body, 0)
            ),
            Exp::MakeClosure {
                arity,
                label,
                env,
                ann: _,
            } => format!("make_closure({}, {}, {})", arity, label, print_prog(env, 0)),
            Exp::MakeTypeInstance {
                typetag,
                fields,
                ann: _,
            } => {
                format!("make_type_instance(type{}, {})", typetag, print_prog(fields, 0))
            }
            Exp::MatchType {
                expr,
                typetag,
                ann: _,
            } => {
                format!("{}.ofType({})", print_prog(expr, 0), typetag)
            }
            Exp::GetTypeFields(expr, _) => format!("{}.fields", print_prog(expr, 0)),
            Exp::TypeDefs {
                decls,
                body,
                ann: _,
            } => {
                format!(
                    "type {} in\n{}",
                    decls
                        .iter()
                        .map(|(name, fields)| format!("{}({})", name, fields.join(", ")))
                        .collect::<Vec<String>>()
                        .join(", "),
                    print_prog(body, indent)
                )
            }
            Exp::Match {
                expr,
                default,
                arms,
                ann: _,
            } => format!(
                "match {} default {}:\n{}{}",
                print_prog(expr, 0),
                print_prog(default, 0),
                " ".repeat(indent + 2),
                arms.iter()
                    .map(|(ctor, fields, expr)| format!(
                        "case {}({}) =>\n{},",
                        print_ctor(ctor),
                        fields.join(", "),
                        print_prog(expr, indent + 4)
                    ))
                    .collect::<Vec<String>>()
                    .join((" ".repeat(indent + 2) + "\n").as_str())
            ),
        }
    )
}

fn print_ctor(ctor: &SnakeType) -> String {
    match ctor {
        SnakeType::Custom(name) => name.clone(),
        _ => format!("{:?}", ctor),
    }
}

pub fn print_imm(expr: &ImmExp) -> String {
    match expr {
        ImmExp::Num(n) => format!("{}", n),
        ImmExp::Bool(b) => format!("{}", b),
        ImmExp::Var(v) => format!("{}", v),
    }
}

pub fn print_sprog<Span>(expr: &SeqExp<Span>, indent: usize) -> String
where
    Span: Clone,
{
    format!(
        "{}{}",
        " ".repeat(indent),
        match expr {
            SeqExp::Imm(imm, _) => print_imm(imm),
            SeqExp::Prim1(op, a, _) => format!(
                "{}({})",
                match op {
                    Prim1::Add1 => "add1",
                    Prim1::Sub1 => "sub1",
                    Prim1::Not => "!",
                    Prim1::Print => "print",
                    Prim1::IsBool => "isbool",
                    Prim1::IsNum => "isnum",
                    Prim1::Length => "length",
                    Prim1::IsArray => "isarray",
                    Prim1::IsFun => "isfun",
                },
                print_imm(a)
            ),
            SeqExp::Prim2(op, a, b, _) => match op {
                Prim2::ArrayGet => format!("{}[{}]", print_imm(a), print_imm(b)),
                _ => {
                    format!(
                        "{} {} {}",
                        print_imm(a),
                        match op {
                            Prim2::Add => "+",
                            Prim2::Sub => "-",
                            Prim2::Mul => "*",
                            Prim2::And => "&&",
                            Prim2::Or => "||",
                            Prim2::Lt => "<",
                            Prim2::Gt => ">",
                            Prim2::Le => "<=",
                            Prim2::Ge => ">=",
                            Prim2::Eq => "==",
                            Prim2::Neq => "!=",
                            Prim2::ArrayGet => unreachable!(),
                        },
                        print_imm(b)
                    )
                }
            },
            SeqExp::ArraySet {
                array,
                index,
                new_value,
                ann: _,
            } => format!(
                "{}[{}] := {}",
                print_imm(array),
                print_imm(index),
                print_imm(new_value)
            ),
            SeqExp::Array(elts, _) => format!(
                "[{}]",
                elts.iter()
                    .map(|elt| print_imm(elt))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            SeqExp::MakeClosure {
                arity,
                label,
                env,
                ann: _,
            } => format!("make_closure({}, {}, {})", arity, label, print_imm(env)),
            SeqExp::CallClosure { fun, args, ann: _ } => format!(
                "{}({})",
                print_imm(fun),
                args.iter()
                    .map(|param| print_imm(param))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            SeqExp::Let {
                var,
                bound_exp,
                body,
                ann: _,
            } => match bound_exp.as_ref() {
                SeqExp::Let { .. } => {
                    format!(
                        "let {} =\n{}\n{}in\n{}",
                        var,
                        print_sprog(bound_exp, indent + 2),
                        " ".repeat(indent),
                        print_sprog(body, indent)
                    )
                }
                _ => format!(
                    "let {} = {} in\n{}",
                    var,
                    print_sprog(bound_exp, 0),
                    print_sprog(body, indent)
                ),
            },
            SeqExp::If {
                cond,
                thn,
                els,
                ann: _,
            } => format!(
                "if {}:\n{}\n{}else:\n{}",
                print_imm(cond),
                print_sprog(thn, indent + 2),
                " ".repeat(indent),
                print_sprog(els, indent + 2)
            ),
            SeqExp::MakeTypeInstance {
                typetag: _,
                fields: _,
                ann: _,
            } => todo!(),
            SeqExp::MatchType { expr: _, typetag: _, ann: _ } => todo!(),
            SeqExp::GetTypeFields(_, _) => todo!(),
        }
    )
}
