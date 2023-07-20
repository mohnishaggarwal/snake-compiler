use crate::errors::CompileErr;
use crate::syntax::{Exp, SnakeType, SurfProg};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
enum NameType {
    Type(usize),
    Var,
}

#[derive(Clone)]
enum CtorArg {
    // used for trying to call a type with (), ex:
    // type Some in let x = Some()
    InvalidCall,
    // used to tell how many arguments were passed in the type call
    Fields(usize),
    NoField,
}

const PRIM_TYPES: [&str; 4] = ["Func", "Array", "Bool", "Num"];

pub fn check_prog<Span>(p: &SurfProg<Span>) -> Result<(), CompileErr<Span>>
where
    Span: Clone,
{
    fn get_type_name<'exp>(type_used: &'exp SnakeType) -> &'exp str {
        match type_used {
            SnakeType::Array => "Array",
            SnakeType::Bool => "Bool",
            SnakeType::Func => "Func",
            SnakeType::Num => "Num",
            SnakeType::Custom(custom_type) => custom_type.as_str(),
        }
    }

    fn check_prog_help<'exp, Span>(
        expr: &'exp Exp<Span>,
        mut env: HashMap<&'exp str, NameType>,
        type_args: CtorArg,
    ) -> Result<(), CompileErr<Span>>
    where
        Span: Clone,
    {
        match expr {
            Exp::Num(val, ann) => {
                if *val > i64::MAX >> 1 || *val < i64::MIN >> 1 {
                    return Err(CompileErr::Overflow {
                        num: *val,
                        location: ann.clone(),
                    });
                } else {
                    Ok(())
                }
            }
            Exp::Bool(_, _) => Ok(()),
            Exp::Var(name, ann) => match env.get(name.as_str()) {
                None => Err(CompileErr::UnboundVariable {
                    unbound: name.clone(),
                    location: ann.clone(),
                }),
                Some(var) => match var {
                    NameType::Var => Ok(()),
                    NameType::Type(arity) => match type_args {
                        CtorArg::InvalidCall => Err(CompileErr::WrongTypeCall {
                            type_used: name.clone(),
                            location: ann.clone(),
                        }),
                        CtorArg::Fields(num_args) => {
                            if num_args != *arity {
                                Err(CompileErr::WrongTypeCall {
                                    type_used: name.clone(),
                                    location: ann.clone(),
                                })
                            } else {
                                Ok(())
                            }
                        }
                        CtorArg::NoField => {
                            if *arity != 0 {
                                Err(CompileErr::WrongTypeCall {
                                    type_used: name.clone(),
                                    location: ann.clone(),
                                })
                            } else {
                                Ok(())
                            }
                        }
                    },
                },
            },
            Exp::Prim1(_, e, _) => check_prog_help(e, env, CtorArg::NoField),
            Exp::Prim2(_, lhs, rhs, _) => {
                check_prog_help(lhs, env.clone(), CtorArg::NoField)?;
                check_prog_help(rhs, env.clone(), CtorArg::NoField)
            }
            Exp::Let {
                bindings,
                body,
                ann: span,
            } => {
                let mut local_env: HashMap<&str, NameType> = HashMap::new();
                for (name, expr) in bindings {
                    if PRIM_TYPES.contains(&name.as_str()) {
                        return Err(CompileErr::ShadowPrimType {
                            primitive_type: name.clone(),
                            location: span.clone(),
                        });
                    }
                    if local_env.contains_key(name.as_str()) {
                        return Err(CompileErr::DuplicateBinding {
                            duplicated_name: name.clone(),
                            location: span.clone(),
                        });
                    } else {
                        check_prog_help(expr, env.clone(), CtorArg::NoField)?;
                        local_env.insert(name.as_str(), NameType::Var);
                        env.insert(name.as_str(), NameType::Var);
                    }
                }
                check_prog_help(body, env, CtorArg::NoField)
            }
            Exp::If {
                cond,
                thn,
                els,
                ann: _,
            } => {
                check_prog_help(cond, env.clone(), CtorArg::NoField)?;
                check_prog_help(thn, env.clone(), CtorArg::NoField)?;
                check_prog_help(els, env.clone(), CtorArg::NoField)
            }
            Exp::Array(array_values, _) => {
                for array_val in array_values {
                    check_prog_help(array_val, env.clone(), CtorArg::NoField)?;
                }
                Ok(())
            }
            Exp::ArraySet {
                array,
                index,
                new_value,
                ann: _,
            } => {
                check_prog_help(array, env.clone(), CtorArg::NoField)?;
                check_prog_help(index, env.clone(), CtorArg::NoField)?;
                check_prog_help(new_value, env.clone(), CtorArg::NoField)
            }
            Exp::Semicolon { e1, e2, ann: _ } => {
                check_prog_help(e1, env.clone(), CtorArg::NoField)?;
                check_prog_help(e2, env.clone(), CtorArg::NoField)
            }
            Exp::FunDefs {
                decls,
                body,
                ann: span,
            } => {
                // create new environment with function parameters for the function body
                let mut env_with_all_new_funcs = env.clone();
                let mut env_tmp: HashMap<&str, NameType> = HashMap::new();
                for decl in decls {
                    // check function name
                    match env_tmp.get(decl.name.as_str()) {
                        // function name is good
                        None => {
                            if PRIM_TYPES.contains(&decl.name.as_str()) {
                                return Err(CompileErr::ShadowPrimType {
                                    primitive_type: decl.name.clone(),
                                    location: span.clone(),
                                });
                            }
                            let mut params: HashSet<&str> = HashSet::new();
                            for param_name in decl.parameters.iter() {
                                if !params.insert(param_name.as_str()) {
                                    return Err(CompileErr::DuplicateArgName {
                                        duplicated_name: param_name.clone(),
                                        location: span.clone(),
                                    });
                                }
                            }
                        }
                        // function name collides with another function in the same def block
                        // overloading is forbidden, so just report an error
                        Some(_) => {
                            return Err(CompileErr::DuplicateFunName {
                                duplicated_name: decl.name.clone(),
                                location: span.clone(),
                            })
                        }
                    }
                    // add the function to environment
                    env_tmp.insert(decl.name.as_str(), NameType::Var);
                }
                env_with_all_new_funcs.extend(env_tmp.into_iter());
                for decl in decls {
                    let mut env_inside_function = env_with_all_new_funcs.clone();
                    // scan parameter list to look for duplicates
                    let mut params: HashMap<&str, NameType> = HashMap::new();
                    for param in decl.parameters.iter() {
                        if params.contains_key(param.as_str()) {
                            return Err(CompileErr::DuplicateArgName {
                                duplicated_name: param.clone(),
                                location: span.clone(),
                            });
                        } else {
                            params.insert(param.as_str(), NameType::Var);
                        }
                    }
                    env_inside_function.extend(params.into_iter());
                    // check the function body
                    check_prog_help(&decl.body, env_inside_function, CtorArg::NoField)?;
                }
                check_prog_help(body, env_with_all_new_funcs, CtorArg::NoField)
            }
            Exp::Call(call_exp, params, _) => {
                for param in params {
                    check_prog_help(param, env.clone(), CtorArg::NoField)?;
                }
                check_prog_help(
                    call_exp,
                    env.clone(),
                    if params.len() == 0 {
                        CtorArg::InvalidCall
                    } else {
                        CtorArg::Fields(params.len())
                    },
                )
            }
            Exp::Lambda {
                parameters,
                body,
                ann,
            } => {
                // Check for duplicate parameters
                let mut params: HashSet<&str> = HashSet::new();
                for param in parameters {
                    if !params.insert(param.as_str()) {
                        return Err(CompileErr::DuplicateArgName {
                            duplicated_name: param.clone(),
                            location: ann.clone(),
                        });
                    }
                }

                // Evaluate lambda
                let mut env_inside_lambda = env.clone();
                for param in parameters {
                    env_inside_lambda.insert(param, NameType::Var);
                }
                check_prog_help(body, env_inside_lambda, CtorArg::NoField)
            }
            Exp::MakeClosure {
                arity: _,
                label: _,
                env: _,
                ann: _,
            } => unreachable!(),
            Exp::TypeDefs { decls, body, ann } => {
                let mut seen_types: HashSet<&str> = HashSet::new();

                for (name, args) in decls {
                    if seen_types.contains(name.as_str()) {
                        return Err(CompileErr::DuplicateTypeDefs {
                            duplicate_type: name.clone(),
                            location: ann.clone(),
                        });
                    } else if PRIM_TYPES.contains(&name.as_str()) {
                        return Err(CompileErr::ShadowPrimType {
                            primitive_type: name.clone(),
                            location: ann.clone(),
                        });
                    } else {
                        seen_types.insert(name);
                        env.insert(name, NameType::Type(args.len()));
                    }
                }
                check_prog_help(body, env, CtorArg::NoField)
            }
            Exp::Match {
                expr,
                default,
                arms,
                ann,
            } => {
                check_prog_help(expr, env.clone(), CtorArg::NoField)?;
                check_prog_help(default, env.clone(), CtorArg::NoField)?;

                let mut seen_types: HashSet<&str> = HashSet::new();
                for (type_used, type_args, arm_exp) in arms {
                    let type_name = get_type_name(type_used);

                    match env.get(type_name) {
                        Some(var_type) => match var_type {
                            NameType::Var => {
                                return Err(CompileErr::UndefinedType {
                                    undefined_type: String::from(type_name),
                                    location: ann.clone(),
                                });
                            }
                            NameType::Type(arity) => {
                                if seen_types.contains(type_name) {
                                    return Err(CompileErr::DuplicateMatchArms {
                                        type_used: String::from(type_name),
                                        location: ann.clone(),
                                    });
                                }
                                if *arity != type_args.len() {
                                    return Err(CompileErr::WrongTypeArity {
                                        type_used: String::from(type_name),
                                        expected_arity: *arity,
                                        given_arity: type_args.len(),
                                        location: ann.clone(),
                                    });
                                }
                                seen_types.insert(type_name);
                            }
                        },
                        None if PRIM_TYPES.contains(&type_name) => (),
                        None => {
                            return Err(CompileErr::UndefinedType {
                                undefined_type: String::from(type_name),
                                location: ann.clone(),
                            });
                        }
                    };

                    let mut seen_args: HashSet<String> = HashSet::new();
                    for arg in type_args {
                        if seen_args.contains(arg.as_str()) {
                            return Err(CompileErr::DuplicateMatchArmArguments {
                                type_used: arg.clone(),
                                location: ann.clone(),
                            });
                        }
                        seen_args.insert(arg.clone());
                    }

                    let mut local_env = env.clone();
                    for arg in type_args {
                        local_env.insert(arg, NameType::Var);
                    }
                    check_prog_help(arm_exp, local_env, CtorArg::NoField)?;
                }

                Ok(())
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

    let env = HashMap::new();
    check_prog_help(p, env, CtorArg::NoField)
}
