use crate::syntax::{Exp, FunDecl, ImmExp, SeqExp, SeqFunDecl, SeqProg};

// Precondition: expressions do not include local function definitions or lambdas
pub fn sequentialize(decls: &[FunDecl<Exp<u32>, u32>], p: &Exp<u32>) -> SeqProg<()> {
  fn seq_help(e: &Exp<u32>) -> SeqExp<()> {
    match e {
      Exp::Num(val, _) => SeqExp::Imm(ImmExp::Num(val.clone()), ()),
      Exp::Bool(val, _) => SeqExp::Imm(ImmExp::Bool(val.clone()), ()),
      Exp::Var(name, _) => SeqExp::Imm(ImmExp::Var(name.clone()), ()),
      Exp::Prim1(op, e1, tag) => {
        let s_e1 = seq_help(&*e1);
        let name = format!("#prim1_{}", tag);
        SeqExp::Let {
          var: name.clone(),
          bound_exp: Box::new(s_e1),
          body: Box::new(SeqExp::Prim1(*op, ImmExp::Var(name.clone()), ())),
          ann: (),
        }
      }
      Exp::Prim2(op, e1, e2, tag) => {
        let s_e1 = seq_help(e1);
        let s_e2 = seq_help(e2);
        let name1 = format!("#prim2_1_{}", tag);
        let name2 = format!("#prim2_2_{}", tag);
        SeqExp::Let {
          var: name1.clone(),
          bound_exp: Box::new(s_e1),
          ann: (),
          body: Box::new(SeqExp::Let {
            var: name2.clone(),
            bound_exp: Box::new(s_e2),
            ann: (),
            body: Box::new(SeqExp::Prim2(
              *op,
              ImmExp::Var(name1),
              ImmExp::Var(name2),
              (),
            )),
          }),
        }
      }
      Exp::If {
        cond,
        thn,
        els,
        ann,
      } => {
        let s_cond = seq_help(cond);
        let name = format!("#if_{}", ann);
        let body = SeqExp::If {
          cond: ImmExp::Var(name.clone()),
          thn: Box::new(seq_help(thn)),
          els: Box::new(seq_help(els)),
          ann: (),
        };
        SeqExp::Let {
          var: name,
          bound_exp: Box::new(s_cond),
          body: Box::new(body),
          ann: (),
        }
      }
      Exp::Let {
        bindings,
        body,
        ann: _,
      } => {
        let mut ret = seq_help(&body);
        for (name, expr) in bindings.iter().rev() {
          ret = SeqExp::Let {
            var: name.clone(),
            bound_exp: Box::new(seq_help(&expr)),
            body: Box::new(ret),
            ann: (),
          };
        }
        ret
      }
      Exp::FunDefs {
        decls: _,
        body: _,
        ann: _,
      } => unreachable!(), // should have been lifted
      Exp::Array(array_vals, tag) => {
        let mut ret = SeqExp::Array(
          array_vals
            .into_iter()
            .map(|array_val| ImmExp::Var(format!("#arr_val_{}_{}", tag, array_val.ann())))
            .collect(),
          (),
        );
        for array_val in array_vals.iter().rev() {
          ret = SeqExp::Let {
            var: format!("#arr_val_{}_{}", tag, array_val.ann()),
            bound_exp: Box::new(seq_help(array_val)),
            body: Box::new(ret),
            ann: (),
          }
        }
        ret
      }
      Exp::ArraySet {
        array,
        index,
        new_value,
        ann: _,
      } => SeqExp::Let {
        var: format!("#arr_id_{}", array.ann()),
        bound_exp: Box::new(seq_help(array)),
        body: Box::new(SeqExp::Let {
          var: format!("#arr_idx_{}", index.ann()),
          bound_exp: Box::new(seq_help(index)),
          body: Box::new(SeqExp::Let {
            var: format!("#arr_new_val_{}", new_value.ann()),
            bound_exp: Box::new(seq_help(new_value)),
            body: Box::new(SeqExp::ArraySet {
              array: ImmExp::Var(format!("#arr_id_{}", array.ann())),
              index: ImmExp::Var(format!("#arr_idx_{}", index.ann())),
              new_value: ImmExp::Var(format!("#arr_new_val_{}", new_value.ann())),
              ann: (),
            }),
            ann: (),
          }),
          ann: (),
        }),
        ann: (),
      },
      Exp::Semicolon { e1, e2, ann } => SeqExp::Let {
        var: format!("#DONT_CARE_{}", ann),
        bound_exp: Box::new(seq_help(e1)),
        body: Box::new(seq_help(e2)),
        ann: (),
      },
      Exp::Lambda {
        parameters: _,
        body: _,
        ann: _,
      } => unreachable!(), // should have been replaced by MakeClosure
      Exp::MakeClosure {
        arity,
        label,
        env,
        ann: _,
      } => match env.as_ref() {
        // we should be coming from `lambda_lift`, which should have created the let
        // binding for `#env` already, so `env` here must be a `Var`
        Exp::Var(env_var_name, _) => SeqExp::MakeClosure {
          arity: *arity,
          label: label.clone(),
          env: ImmExp::Var(env_var_name.clone()),
          ann: (),
        },
        _ => unreachable!(),
      },
      Exp::Call(func_expr, params, tag) => {
        // Here is a very tricky technique!
        // Overview:
        // We make use of the sequentialize helper function to save some manual wrapping
        // of let bindings.
        // We first introduce an "intermediate function name" which starts with a "#", to
        // prevent variable name collision.
        // The first time we hit this arm we should be coming from `lambda_lift`, so the
        // function expression cannot match a `Var` named "#call_func_intermediate".
        //
        // Since we do not fully sequentialize the expression yet, we recursively call the
        // sequentialize helper function to do a second pass. This time the let block will
        // be automatically sequentialized, and we will again hit this arm because we did
        // not change it at all. Our final goal is `SeqExp::CallClosure` but we still have
        // an `Exp::Call` sitting at the core of the let binding after our first pass.
        //
        // However, this time we can recognize that we are actually at the second pass
        // because the function expression has been replaced with a variable named
        // "#call_func_intermediate".
        match func_expr.as_ref() {
          Exp::Var(var_name, tag) if var_name == "#call_func_intermediate" => {
            // Second pass
            // At this point everything has been sequentialized, and we just need to
            // simply convert the `Exp` to `SeqExp`.
            return SeqExp::CallClosure {
              fun: ImmExp::Var(format!("#call_func_{}", tag)),
              args: params
                .iter()
                .map(|expr| match expr {
                  Exp::Var(name, _) => ImmExp::Var(name.clone()),
                  _ => unreachable!(),
                })
                .collect(),
              ann: (),
            };
          }
          _ => {
            // First pass
            //
            // example: tag=5
            // e1(e2, e3)
            // let #call_0_5=e2,
            //     #call_1_5=e3,
            //     #call_func_5=e1
            // in
            // #call_func_intermediate(#call_0_5, #call_1_5)
            // At this point it's still `Exp`, not `SeqExp`!
            //
            // The tag is passed through so that later at the second pass when we
            // recognize the intermediate function name, we can replace it with the
            // actual sequentialized function name.
            seq_help(&Exp::Let {
              bindings: params
                .iter()
                .enumerate()
                .map(|(idx, param)| (format!("#call_{}_{}", idx, tag), param.clone()))
                .chain(vec![(format!("#call_func_{}", tag), *func_expr.clone())])
                .collect(),
              body: Box::new(Exp::Call(
                Box::new(Exp::Var(String::from("#call_func_intermediate"), *tag)),
                (0..params.len())
                  .map(|idx| Exp::Var(format!("#call_{}_{}", idx, *tag), *tag))
                  .collect(),
                *tag,
              )),
              ann: *tag,
            })
          }
        }
      }
      // TypeDefs and Match were resolved in a prior pass
      Exp::TypeDefs { .. } => unreachable!(),
      Exp::Match { .. } => unreachable!(),
      Exp::MakeTypeInstance {
        typetag,
        fields,
        ann: tag,
      } => SeqExp::Let {
        var: format!("fields_exp_{}", tag),
        bound_exp: Box::new(seq_help(fields)),
        body: Box::new(SeqExp::MakeTypeInstance {
          typetag: *typetag,
          fields: ImmExp::Var(format!("fields_exp_{}", tag)),
          ann: (),
        }),
        ann: (),
      },
      Exp::MatchType {
        expr,
        typetag,
        ann: tag,
      } => SeqExp::Let {
        var: format!("match_exp_{}", tag),
        bound_exp: Box::new(seq_help(expr)),
        body: Box::new(SeqExp::MatchType {
          expr: ImmExp::Var(format!("match_exp_{}", tag)),
          typetag: *typetag,
          ann: (),
        }),
        ann: (),
      },
      Exp::GetTypeFields(expr, tag) => SeqExp::Let {
        var: format!("get_type_fields_{}", tag),
        bound_exp: Box::new(seq_help(expr)),
        body: Box::new(SeqExp::GetTypeFields(
          ImmExp::Var(format!("get_type_fields_{}", tag)),
          (),
        )),
        ann: (),
      },
    }
  }
  let main = seq_help(p);
  let funs = decls
    .into_iter()
    .map(|decl| SeqFunDecl {
      name: decl.name.clone(),
      parameters: decl.parameters.clone(),
      body: seq_help(&decl.body),
      ann: (),
    })
    .collect();
  SeqProg {
    funs,
    main,
    ann: (),
  }
}
