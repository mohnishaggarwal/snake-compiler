use crate::syntax::{Exp, FunDecl, Prim2};

fn lift_function(
    funcs: &mut Vec<FunDecl<Exp<()>, ()>>,
    env: Vec<&str>,
    name: String,
    params: &Vec<String>,
    body: &Exp<u32>,
    tag: u32,
) {
    // capture all variables in the environment
    let captures: Vec<String> = env.clone().into_iter().map(String::from).collect();
    // create environment for the body
    // it should include all parameters and captured variables
    let mut env_for_body = env;
    env_for_body.extend(params.iter().map(String::as_str));
    let body = Exp::Let {
        bindings: captures
            .iter()
            .enumerate()
            .map(|(idx, captured_name)| {
                (
                    // let x1 = env[0],
                    //     x2 = env[1],
                    //     ...
                    // in
                    captured_name.clone(),
                    Exp::Prim2(
                        Prim2::ArrayGet,
                        Box::new(Exp::Var(format!("#env_{}", tag), ())),
                        Box::new(Exp::Num(idx as i64, ())),
                        (),
                    ),
                )
            })
            .collect(),
        body: Box::new(lambda_lift_help(funcs, &body, env_for_body)),
        ann: (),
    };
    // register the function declaration to the global function list
    funcs.push(FunDecl {
        name: name.clone(),
        // prepend a special parameter called `#env`
        parameters: vec![format!("#env_{}", tag)]
            .into_iter()
            .chain(params.clone().into_iter())
            .collect(),
        body,
        ann: (),
    });
}

fn lambda_lift_help(
    funcs: &mut Vec<FunDecl<Exp<()>, ()>>,
    e: &Exp<u32>,
    env: Vec<&str>,
) -> Exp<()> {
    match e {
        Exp::FunDefs {
            decls,
            body,
            ann: tag,
        } => {
            // add all functions to the environment
            let env_with_new_funcs: Vec<&str> = env
                .clone()
                .into_iter()
                .chain(
                    decls
                        .iter()
                        .map(|decl| decl.name.as_str())
                        .collect::<Vec<&str>>(),
                )
                .collect();
            // lift each function to the top level, inserting necessary let bindings into their
            // bodies and prepending the special parameter `#env`
            for fundecl in decls {
                lift_function(
                    funcs,
                    env_with_new_funcs.clone(),
                    fundecl.name.clone(),
                    &fundecl.parameters,
                    &fundecl.body,
                    fundecl.ann,
                );
            }
            // next we want to replace the definition with a let binding
            // def f(): 1 in ...
            // becomes
            // let f = make_closure(0, f, env) in ...
            // where the raw function pointer f has been lifted
            // first create the env array
            let env_expr = Exp::Array(
                // we use the env without all these new functions because we will later append
                // placeholders for them
                env.iter()
                    .map(|name| Exp::Var(String::from(*name), ()))
                    .chain(
                        // Landin's Knot placeholders
                        (0..decls.len())
                            .into_iter()
                            .map(|_| Exp::Num(0, ()))
                            .collect::<Vec<Exp<()>>>(),
                    )
                    .collect(),
                (),
            );
            // let #env = [x1, x2, x3, ...],
            let mut bindings = vec![(format!("#env_{}", tag), env_expr)];
            //     f = make_closure(f_arity, f, env),
            //     g = make_closure(g_arity, g, env),
            //     ...
            bindings.extend(decls.iter().map(|decl| {
                (
                    decl.name.clone(),
                    Exp::MakeClosure {
                        arity: decl.parameters.len(),
                        label: decl.name.clone(),
                        env: Box::new(Exp::Var(format!("#env_{}", tag), ())),
                        ann: (),
                    },
                )
            }));
            let mut defblock_body = lambda_lift_help(funcs, body, env_with_new_funcs.clone());
            // in
            // env[3] := f;
            // env[4] := g;
            // ...
            for idx in 0..decls.len() {
                defblock_body = Exp::Semicolon {
                    e1: Box::new(Exp::ArraySet {
                        array: Box::new(Exp::Var(format!("#env_{}", tag), ())),
                        // skip the first `env.len()` elements: these are captured variables
                        // excluding the new functions
                        // the rest are placeholder entries we want to update
                        index: Box::new(Exp::Num((env.len() + idx) as i64, ())),
                        new_value: Box::new(Exp::Var(decls[idx].name.clone(), ())),
                        ann: (),
                    }),
                    e2: Box::new(defblock_body),
                    ann: (),
                }
            }
            Exp::Let {
                bindings,
                body: Box::new(defblock_body),
                ann: (),
            }
        }
        Exp::Num(val, _) => Exp::Num(*val, ()),
        Exp::Bool(val, _) => Exp::Bool(*val, ()),
        Exp::Var(name, _) => Exp::Var(name.clone(), ()),
        Exp::Prim1(op, operand, _) => {
            Exp::Prim1(*op, Box::new(lambda_lift_help(funcs, operand, env)), ())
        }
        Exp::Prim2(op, operand1, operand2, _) => Exp::Prim2(
            *op,
            Box::new(lambda_lift_help(funcs, operand1, env.clone())),
            Box::new(lambda_lift_help(funcs, operand2, env)),
            (),
        ),
        Exp::Let {
            bindings,
            body,
            ann: _,
        } => {
            // create new env that includes all new bindings
            let mut env_new = env.clone();
            env_new.extend(bindings.iter().map(|(name, _)| name.as_str()));
            // process the body, lifting functions defined inside
            let body = lambda_lift_help(funcs, body, env_new);
            // process each binding
            // order does not matter because we already performed uniquification
            let mut bindings_new: Vec<(String, Exp<()>)> = Vec::new();
            for (bname, bdef) in bindings {
                let bdef = lambda_lift_help(funcs, &bdef, env.clone());
                // the processed version of the binding body
                bindings_new.push((bname.clone(), bdef));
            }
            Exp::Let {
                bindings: bindings_new,
                body: Box::new(body), // body has been processed
                ann: (),
            }
        }
        Exp::If {
            cond,
            thn,
            els,
            ann: _,
        } => Exp::If {
            cond: Box::new(lambda_lift_help(funcs, cond, env.clone())),
            thn: Box::new(lambda_lift_help(funcs, thn, env.clone())),
            els: Box::new(lambda_lift_help(funcs, els, env)),
            ann: (),
        },
        Exp::Call(func_expr, params, _) => Exp::Call(
            // we don't change anything here, and leave the work to `sequentialize`
            Box::new(lambda_lift_help(funcs, func_expr, env.clone())),
            params
                .into_iter()
                .map(|param| lambda_lift_help(funcs, param, env.clone()))
                .collect(),
            (),
        ),
        Exp::Array(array_values, _) => Exp::Array(
            array_values
                .into_iter()
                .map(|arr_val| -> Exp<()> { lambda_lift_help(funcs, arr_val, env.clone()) })
                .collect(),
            (),
        ),
        Exp::ArraySet {
            array,
            index,
            new_value,
            ann: _,
        } => Exp::ArraySet {
            array: Box::new(lambda_lift_help(funcs, array, env.clone())),
            index: Box::new(lambda_lift_help(funcs, index, env.clone())),
            new_value: Box::new(lambda_lift_help(funcs, new_value, env)),
            ann: (),
        },
        Exp::Semicolon { e1, e2, ann: _ } => Exp::Semicolon {
            e1: Box::new(lambda_lift_help(funcs, e1, env.clone())),
            e2: Box::new(lambda_lift_help(funcs, e2, env)),
            ann: (),
        },
        Exp::Lambda {
            parameters,
            body,
            ann: tag,
        } => {
            // give the lambda a unique name
            let lambda_name = format!("__snake_lambda_{}", tag);
            // lift it to the top level
            lift_function(
                funcs,
                env.clone(),
                lambda_name.clone(),
                parameters,
                body,
                *tag,
            );
            // example: tag=2
            //
            // let a = 1 in
            // lambda x: x end
            //
            // becomes
            //
            // let #env = [a] in
            // make_closure(1, __snake_lambda_2, #env)
            //
            // note: here `#env` is a `Var`!
            Exp::Let {
                bindings: vec![(
                    format!("#env_{}", tag),
                    Exp::Array(
                        env.iter()
                            .map(|name| Exp::Var(String::from(*name), ()))
                            .collect(),
                        (),
                    ),
                )],
                body: Box::new(Exp::MakeClosure {
                    arity: parameters.len(),
                    label: lambda_name,
                    env: Box::new(Exp::Var(format!("#env_{}", tag), ())),
                    ann: (),
                }),
                ann: (),
            }
        }
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
            body: Box::new(lambda_lift_help(funcs, body, env)),
            ann: (),
        },
        Exp::Match {
            expr,
            default,
            arms,
            ann: _,
        } => Exp::Match {
            expr: Box::new(lambda_lift_help(funcs, expr, env.clone())),
            default: Box::new(lambda_lift_help(funcs, default, env.clone())),
            arms: arms
                .into_iter()
                .map(|(snake_type, params, exp)| {
                    (
                        snake_type.clone(),
                        params.clone(),
                        lambda_lift_help(funcs, exp, env.clone()),
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
            fields: Box::new(lambda_lift_help(funcs, fields, env)),
            ann: (),
        },
        Exp::MatchType {
            expr,
            typetag,
            ann: _,
        } => Exp::MatchType {
            expr: Box::new(lambda_lift_help(funcs, expr, env)),
            typetag: *typetag,
            ann: (),
        },
        Exp::GetTypeFields(expr, _) => {
            Exp::GetTypeFields(Box::new(lambda_lift_help(funcs, expr, env)), ())
        }
    }
}

// Precondition: all names are uniquified
pub fn lambda_lift(p: &Exp<u32>) -> (Vec<FunDecl<Exp<()>, ()>>, Exp<()>) {
    let mut funcs = Vec::new();
    let prog = lambda_lift_help(&mut funcs, p, Vec::new());
    (funcs, prog)
}
