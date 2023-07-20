use crate::syntax::{Exp, FunDecl, Prim1, Prim2, SnakeType};
use std::collections::HashMap;

// return the original expression except with every instance of
// var_to_replace replaced with new_var
fn replace_var_name(expr: &Exp<()>, var_to_replace: String, new_var: String) -> Exp<()> {
    match expr {
        Exp::Num(num, _) => Exp::Num(*num, ()),
        Exp::Bool(boolean, _) => Exp::Bool(*boolean, ()),
        Exp::Var(var, _) => {
            if *var == var_to_replace {
                Exp::Var(new_var.clone(), ())
            } else {
                Exp::Var(var.clone(), ())
            }
        }
        Exp::Prim1(op, operand, _) => Exp::Prim1(
            *op,
            Box::new(replace_var_name(operand, var_to_replace, new_var)),
            (),
        ),
        Exp::Prim2(op, operand1, operand2, _) => Exp::Prim2(
            *op,
            Box::new(replace_var_name(
                operand1,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            Box::new(replace_var_name(
                operand2,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            (),
        ),
        Exp::Array(array_values, _) => Exp::Array(
            array_values
                .into_iter()
                .map(|array_val| -> Exp<()> {
                    replace_var_name(array_val, var_to_replace.clone(), new_var.clone())
                })
                .collect(),
            (),
        ),
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => Exp::ArraySet {
            array: Box::new(replace_var_name(
                array,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            index: Box::new(replace_var_name(
                index,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            new_value: Box::new(replace_var_name(
                new_value,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            ann: (),
        },
        Exp::Semicolon { e1, e2, ann: _ } => Exp::Semicolon {
            e1: Box::new(replace_var_name(
                e1,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            e2: Box::new(replace_var_name(
                e2,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            ann: (),
        },
        Exp::Let {
            bindings,
            body,
            ann: _,
        } => Exp::Let {
            bindings: bindings
                .into_iter()
                .map(|(var_name, exp)| {
                    (
                        var_name.clone(),
                        replace_var_name(exp, var_to_replace.clone(), new_var.clone()),
                    )
                })
                .collect(),
            body: Box::new(replace_var_name(
                body,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            ann: (),
        },
        Exp::If {
            cond,
            thn,
            els,
            ann: _,
        } => Exp::If {
            cond: Box::new(replace_var_name(
                cond,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            thn: Box::new(replace_var_name(
                thn,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            els: Box::new(replace_var_name(
                els,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            ann: (),
        },
        Exp::FunDefs {
            decls,
            body,
            ann: _,
        } => Exp::FunDefs {
            decls: decls
                .into_iter()
                .map(|decl| FunDecl {
                    name: decl.name.clone(),
                    parameters: decl.parameters.clone(),
                    body: replace_var_name(&decl.body, var_to_replace.clone(), new_var.clone()),
                    ann: (),
                })
                .collect(),
            body: Box::new(replace_var_name(body, var_to_replace, new_var.clone())),
            ann: (),
        },
        Exp::Call(callee, params, _) => Exp::Call(
            Box::new(replace_var_name(
                callee,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            params
                .into_iter()
                .map(|param| -> Exp<()> {
                    replace_var_name(param, var_to_replace.clone(), new_var.clone())
                })
                .collect(),
            (),
        ),
        Exp::Lambda {
            parameters,
            body,
            ann: _,
        } => Exp::Lambda {
            parameters: parameters.clone(),
            body: Box::new(replace_var_name(
                body,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            ann: (),
        },
        Exp::MakeClosure {
            arity: _,
            label: _,
            env: _,
            ann: _,
        } => unreachable!(),
        Exp::TypeDefs {
            decls,
            body,
            ann: _,
        } => Exp::TypeDefs {
            decls: decls.clone(),
            body: Box::new(replace_var_name(
                body,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            ann: (),
        },
        Exp::Match {
            expr,
            default,
            arms,
            ann: _,
        } => Exp::Match {
            expr: Box::new(replace_var_name(
                expr,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            default: Box::new(replace_var_name(
                default,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            arms: arms
                .into_iter()
                .map(|(snake_type, params, exp)| {
                    (
                        snake_type.clone(),
                        params.clone(),
                        replace_var_name(exp, var_to_replace.clone(), new_var.clone()),
                    )
                })
                .collect(),
            ann: (),
        },
        Exp::MakeTypeInstance {
            typetag,
            fields,
            ann: _,
        } => Exp::MakeTypeInstance {
            typetag: *typetag,
            fields: Box::new(replace_var_name(fields, var_to_replace, new_var)),
            ann: (),
        },
        Exp::MatchType {
            expr,
            typetag,
            ann: _,
        } => Exp::MatchType {
            expr: Box::new(replace_var_name(
                expr,
                var_to_replace.clone(),
                new_var.clone(),
            )),
            typetag: *typetag,
            ann: (),
        },
        Exp::GetTypeFields(exp, _) => replace_var_name(exp, var_to_replace, new_var),
    }
}

fn resolve_types_helper<'exp>(
    expr: &'exp Exp<u32>,
    type_tag_translation: &'exp mut HashMap<String, u64>,
) -> Exp<()> {
    match expr {
        Exp::Num(num, _) => Exp::Num(*num, ()),
        Exp::Bool(boolean, _) => Exp::Bool(*boolean, ()),
        Exp::Var(var, _) => Exp::Var(var.clone(), ()),
        Exp::Prim1(op, operand, _) => Exp::Prim1(
            *op,
            Box::new(resolve_types_helper(operand, type_tag_translation)),
            (),
        ),
        Exp::Prim2(op, operand1, operand2, _) => Exp::Prim2(
            *op,
            Box::new(resolve_types_helper(operand1, type_tag_translation)),
            Box::new(resolve_types_helper(operand2, type_tag_translation)),
            (),
        ),
        Exp::Array(array_values, _) => Exp::Array(
            array_values
                .iter()
                .map(|array_val| -> Exp<()> {
                    resolve_types_helper(&array_val, type_tag_translation)
                })
                .collect(),
            (),
        ),
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => Exp::ArraySet {
            array: Box::new(resolve_types_helper(array, type_tag_translation)),
            index: Box::new(resolve_types_helper(index, type_tag_translation)),
            new_value: Box::new(resolve_types_helper(new_value, type_tag_translation)),
            ann: (),
        },
        Exp::Semicolon { e1, e2, ann: _ } => Exp::Semicolon {
            e1: Box::new(resolve_types_helper(e1, type_tag_translation)),
            e2: Box::new(resolve_types_helper(e2, type_tag_translation)),
            ann: (),
        },
        Exp::Let {
            bindings,
            body,
            ann: _,
        } => Exp::Let {
            bindings: bindings
                .into_iter()
                .map(|(var_name, exp)| {
                    (
                        var_name.clone(),
                        resolve_types_helper(exp, type_tag_translation),
                    )
                })
                .collect(),
            body: Box::new(resolve_types_helper(body, type_tag_translation)),
            ann: (),
        },
        Exp::If {
            cond,
            thn,
            els,
            ann: _,
        } => Exp::If {
            cond: Box::new(resolve_types_helper(cond, type_tag_translation)),
            thn: Box::new(resolve_types_helper(thn, type_tag_translation)),
            els: Box::new(resolve_types_helper(els, type_tag_translation)),
            ann: (),
        },
        Exp::FunDefs {
            decls,
            body,
            ann: _,
        } => Exp::FunDefs {
            decls: decls
                .into_iter()
                .map(|decl| FunDecl {
                    name: decl.name.clone(),
                    parameters: decl.parameters.clone(),
                    body: resolve_types_helper(&decl.body, type_tag_translation),
                    ann: (),
                })
                .collect(),
            body: Box::new(resolve_types_helper(body, type_tag_translation)),
            ann: (),
        },
        Exp::Call(callee, params, _) => Exp::Call(
            Box::new(resolve_types_helper(callee, type_tag_translation)),
            params
                .into_iter()
                .map(|param| -> Exp<()> { resolve_types_helper(param, type_tag_translation) })
                .collect(),
            (),
        ),
        Exp::Lambda {
            parameters,
            body,
            ann: _,
        } => Exp::Lambda {
            parameters: parameters.clone(),
            body: Box::new(resolve_types_helper(body, type_tag_translation)),
            ann: (),
        },
        Exp::MakeClosure {
            arity: _,
            label: _,
            env: _,
            ann: _,
        } => unreachable!(),
        Exp::TypeDefs {
            decls,
            body,
            ann: _,
        } => {
            for (name, _) in decls {
                type_tag_translation.insert(name.clone(), type_tag_translation.len() as u64);
            }
            let mut ret_exp = resolve_types_helper(body, type_tag_translation);
            for (name, args) in decls.iter().rev() {
                if args.len() == 0 {
                    ret_exp = Exp::Let {
                        bindings: vec![(
                            name.clone(),
                            Exp::MakeTypeInstance {
                                typetag: *type_tag_translation.get(name).unwrap(),
                                fields: Box::new(Exp::Array(vec![], ())),
                                ann: (),
                            },
                        )],
                        body: Box::new(ret_exp),
                        ann: (),
                    }
                } else {
                    ret_exp = Exp::FunDefs {
                        decls: vec![FunDecl {
                            name: name.clone(),
                            parameters: args.clone(),
                            body: Exp::MakeTypeInstance {
                                typetag: *type_tag_translation.get(name).unwrap(),
                                fields: Box::new(Exp::Array(
                                    args.into_iter()
                                        .map(|type_arg| Exp::Var(type_arg.clone(), ()))
                                        .collect(),
                                    (),
                                )),
                                ann: (),
                            },
                            ann: (),
                        }],
                        body: Box::new(ret_exp),
                        ann: (),
                    }
                }
            }
            ret_exp
        }
        Exp::Match {
            expr,
            default,
            arms,
            ann: _,
        } => {
            let matchee_var = format!("__matchee_{}", expr.ann());
            let fields_var = format!("__fields_{}", expr.ann());
            let mut ret_exp = resolve_types_helper(default, type_tag_translation);
            for (snake_type, args, arm_exp) in arms.into_iter().rev() {
                match snake_type {
                    SnakeType::Custom(type_name) => {
                        ret_exp = Exp::If {
                            cond: Box::new(Exp::MatchType {
                                expr: Box::new(Exp::Var(matchee_var.clone(), ())),
                                typetag: type_tag_translation.get(type_name).unwrap().clone(),
                                ann: (),
                            }),
                            thn: Box::new(Exp::Let {
                                bindings: args
                                    .into_iter()
                                    .enumerate()
                                    .map(|(idx, param)| {
                                        (
                                            param.clone(),
                                            Exp::Prim2(
                                                Prim2::ArrayGet,
                                                Box::new(Exp::Var(fields_var.clone(), ())),
                                                Box::new(Exp::Num(idx as i64, ())),
                                                (),
                                            ),
                                        )
                                    })
                                    .collect(),
                                body: Box::new(resolve_types_helper(arm_exp, type_tag_translation)),
                                ann: (),
                            }),
                            els: Box::new(ret_exp),
                            ann: (),
                        };
                    }
                    SnakeType::Array => {
                        ret_exp = Exp::If {
                            cond: Box::new(Exp::Prim1(
                                Prim1::IsArray,
                                Box::new(Exp::Var(matchee_var.clone(), ())),
                                (),
                            )),
                            // args will always only have one argument in a non custom snake type
                            thn: Box::new(replace_var_name(
                                &resolve_types_helper(arm_exp, type_tag_translation),
                                args[0].clone(),
                                matchee_var.clone(),
                            )),
                            els: Box::new(ret_exp),
                            ann: (),
                        };
                    }
                    SnakeType::Func => {
                        ret_exp = Exp::If {
                            cond: Box::new(Exp::Prim1(
                                Prim1::IsFun,
                                Box::new(Exp::Var(matchee_var.clone(), ())),
                                (),
                            )),
                            // args will always only have one argument in a non custom snake type
                            thn: Box::new(replace_var_name(
                                &resolve_types_helper(arm_exp, type_tag_translation),
                                args[0].clone(),
                                matchee_var.clone(),
                            )),
                            els: Box::new(ret_exp),
                            ann: (),
                        };
                    }
                    SnakeType::Bool => {
                        ret_exp = Exp::If {
                            cond: Box::new(Exp::Prim1(
                                Prim1::IsBool,
                                Box::new(Exp::Var(matchee_var.clone(), ())),
                                (),
                            )),
                            // args will always only have one argument in a non custom snake type
                            thn: Box::new(replace_var_name(
                                &resolve_types_helper(arm_exp, type_tag_translation),
                                args[0].clone(),
                                matchee_var.clone(),
                            )),
                            els: Box::new(ret_exp),
                            ann: (),
                        };
                    }
                    SnakeType::Num => {
                        ret_exp = Exp::If {
                            cond: Box::new(Exp::Prim1(
                                Prim1::IsNum,
                                Box::new(Exp::Var(matchee_var.clone(), ())),
                                (),
                            )),
                            // args will always only have one argument in a non custom snake type
                            thn: Box::new(replace_var_name(
                                &resolve_types_helper(arm_exp, type_tag_translation),
                                args[0].clone(),
                                matchee_var.clone(),
                            )),
                            els: Box::new(ret_exp),
                            ann: (),
                        };
                    }
                }
            }
            ret_exp = Exp::Let {
                bindings: vec![
                    (
                        matchee_var.clone(),
                        resolve_types_helper(expr, type_tag_translation),
                    ),
                    (
                        fields_var,
                        Exp::GetTypeFields(Box::new(Exp::Var(matchee_var, ())), ()),
                    ),
                ],
                body: Box::new(ret_exp),
                ann: (),
            };
            ret_exp
        }
        Exp::MakeTypeInstance {
            typetag: _,
            fields: _,
            ann: _,
        } => unreachable!(),
        Exp::MatchType {
            expr: _,
            typetag: _,
            ann: _,
        } => unreachable!(),
        Exp::GetTypeFields(_, _) => unreachable!(),
    }
}

pub fn resolve_types(expr: &Exp<u32>) -> (Exp<()>, HashMap<String, u64>) {
    let mut type_tag_translation: HashMap<String, u64> = HashMap::new();
    let ret_exp = resolve_types_helper(expr, &mut type_tag_translation);
    (ret_exp, type_tag_translation)
}
