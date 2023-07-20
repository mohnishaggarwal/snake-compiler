use crate::syntax::{Exp, FunDecl, SnakeType};
use std::collections::HashMap;

pub fn uniquify(e: &Exp<u32>) -> Exp<()> {
  fn uniquify_help(e: &Exp<u32>, translation_table: HashMap<&str, String>) -> Exp<()> {
    match e {
      Exp::Num(val, _) => Exp::Num(*val, ()),
      Exp::Bool(val, _) => Exp::Bool(*val, ()),
      Exp::Var(name, _) => Exp::Var(translation_table.get(name.as_str()).unwrap().clone(), ()),
      Exp::Prim1(op, operand, _) => {
        // just uniquify the operand
        Exp::Prim1(
          *op,
          Box::new(uniquify_help(operand, translation_table.clone())),
          (),
        )
      }
      Exp::Prim2(op, operand1, operand2, _) => Exp::Prim2(
        *op,
        Box::new(uniquify_help(operand1, translation_table.clone())),
        Box::new(uniquify_help(operand2, translation_table.clone())),
        (),
      ),
      Exp::Array(array_values, _) => Exp::Array(
        array_values
          .into_iter()
          .map(|array_val| -> Exp<()> { uniquify_help(array_val, translation_table.clone()) })
          .collect(),
        (),
      ),
      Exp::ArraySet {
        array,
        index,
        new_value,
        ann: _,
      } => Exp::ArraySet {
        array: Box::new(uniquify_help(array, translation_table.clone())),
        index: Box::new(uniquify_help(index, translation_table.clone())),
        new_value: Box::new(uniquify_help(new_value, translation_table.clone())),
        ann: (),
      },
      Exp::Semicolon { e1, e2, ann: _ } => Exp::Semicolon {
        e1: Box::new(uniquify_help(e1, translation_table.clone())),
        e2: Box::new(uniquify_help(e2, translation_table.clone())),
        ann: (),
      },
      Exp::Let {
        bindings,
        body,
        ann: tag,
      } => {
        let mut new_bindings: Vec<(String, Exp<()>)> = Vec::new();
        let mut new_tt = translation_table.clone();
        for (name, definition) in bindings {
          // update the new tag lookup table to include the new binding
          new_bindings.push((
            // attach the tag to the variable name
            format!("__snake_var_{}_{}", tag, name),
            // uniquify the binding definition
            // here we should be careful not to include the current binding in the tag lookup table
            // we provide to the recursive call to prevent breaking such cases:
            // let a = 2 in let b = 1, a = a + b in ...
            // b is already recorded in `new_tt` but the new a is not yet
            uniquify_help(&definition, new_tt.clone()),
          ));
          new_tt.insert(&name, format!("__snake_var_{}_{}", tag, name));
        }
        // uniquify the let body, using the new tag lookup table
        Exp::Let {
          bindings: new_bindings,
          body: Box::new(uniquify_help(body, new_tt)),
          ann: (),
        }
      }
      Exp::If {
        cond,
        thn,
        els,
        ann: _,
      } => Exp::If {
        cond: Box::new(uniquify_help(cond, translation_table.clone())),
        thn: Box::new(uniquify_help(thn, translation_table.clone())),
        els: Box::new(uniquify_help(els, translation_table)),
        ann: (),
      },
      Exp::FunDefs {
        decls,
        body,
        ann: _,
      } => {
        let mut new_decls: Vec<FunDecl<Exp<()>, ()>> = Vec::new();
        let mut tt_with_all_new_funcs = translation_table.clone();
        // uniquify all new functions
        // the function may be used both in its funcbody and in the defs block body
        // and also in other decls in the same fundefs block (!)
        tt_with_all_new_funcs.extend(decls.iter().map(|decl| {
          (
            decl.name.as_str(),
            format!("__snake_function_{}_{}", decl.ann, decl.name),
          )
        }));
        for decl in decls.iter() {
          // this translation table will contain all parameters as well
          let mut tt_for_funcbody = tt_with_all_new_funcs.clone();
          let mut new_decl: FunDecl<Exp<()>, ()> = FunDecl {
            name: format!("__snake_function_{}_{}", decl.ann, decl.name),
            parameters: Vec::new(),
            body: Exp::Num(483, ()), // placeholder
            ann: (),
          };
          // uniquify the parameters and update the tag lookup table for funcbody
          for param in decl.parameters.iter() {
            new_decl
              .parameters
              .push(format!("__snake_param_{}_{}", decl.ann, param));
            tt_for_funcbody.insert(&param, format!("__snake_param_{}_{}", decl.ann, param));
          }
          // process the funcbody and add the updated decl to the new decl list
          new_decl.body = uniquify_help(&decl.body, tt_for_funcbody);
          new_decls.push(new_decl);
        }
        Exp::FunDefs {
          decls: new_decls,
          // uniquify the program block, using the new tag lookup table
          body: Box::new(uniquify_help(body, tt_with_all_new_funcs)),
          ann: (),
        }
      }
      Exp::Call(callee, params, _) => Exp::Call(
        // uniquify the callee expression
        Box::new(uniquify_help(callee, translation_table.clone())),
        params
          .into_iter()
          // uniquify each parameter expression
          .map(|param| -> Exp<()> { uniquify_help(param, translation_table.clone()) })
          .collect(),
        (),
      ),
      Exp::Lambda {
        parameters,
        body,
        ann: tag,
      } => {
        // update tag lookup table to include the parameters
        let mut tt_inside_lambda = translation_table.clone();
        tt_inside_lambda.extend(
          parameters
            .into_iter()
            .map(|param| (param.as_str(), format!("__snake_param_{}_{}", *tag, param))),
        );
        // attach tag to each of the parameters and uniquify the body
        Exp::Lambda {
          parameters: parameters
            .into_iter()
            .map(|param| format!("__snake_param_{}_{}", *tag, param))
            .collect(),
          body: Box::new(uniquify_help(body, tt_inside_lambda)),
          ann: (),
        }
      }
      // we do not have this intermediate form until we desugar lambdas and function decls in
      // `lambda_lift`, so we cannot have it here
      Exp::MakeClosure {
        arity: _,
        label: _,
        env: _,
        ann: _,
      } => unreachable!(),
      Exp::TypeDefs {
        decls,
        body,
        ann: tag,
      } => {
        let mut new_decls: Vec<(String, Vec<String>)> = Vec::new();
        let mut new_tt = translation_table.clone();
        for (name, args) in decls {
          new_decls.push((
            format!("__custom_type_{}_{}", tag, name),
            // arguments are just placeholders, they don't need to be uniquified
            args.clone(),
          ));
          new_tt.insert(name, format!("__custom_type_{}_{}", tag, name));
        }
        Exp::TypeDefs {
          decls: new_decls,
          body: Box::new(uniquify_help(body, new_tt)),
          ann: (),
        }
      }
      Exp::Match {
        expr,
        default,
        arms,
        ann: tag,
      } => {
        let mut new_arms: Vec<(SnakeType, Vec<String>, Exp<()>)> = Vec::new();
        let mut new_tt = translation_table.clone();
        for (type_used, args, exp) in arms {
          let new_snake_type: SnakeType;
          match type_used {
            SnakeType::Custom(name) => {
              new_snake_type =
                SnakeType::Custom(translation_table.get(name.as_str()).unwrap().clone())
            }
            SnakeType::Array => new_snake_type = SnakeType::Array,
            SnakeType::Func => new_snake_type = SnakeType::Func,
            SnakeType::Num => new_snake_type = SnakeType::Num,
            SnakeType::Bool => new_snake_type = SnakeType::Bool,
          };
          let mut new_args: Vec<String> = Vec::new();
          for param in args {
            new_args.push(format!("snake_type_param_{}_{}", tag, param));
            new_tt.insert(&param, format!("snake_type_param_{}_{}", tag, param));
          }
          new_arms.push((new_snake_type, new_args, uniquify_help(exp, new_tt.clone())));
        }

        Exp::Match {
          expr: Box::new(uniquify_help(expr, translation_table.clone())),
          default: Box::new(uniquify_help(default, translation_table.clone())),
          arms: new_arms,
          ann: (),
        }
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

  let translation_table: HashMap<&str, String> = HashMap::from([
    ("Func", String::from("Func")),
    ("Num", String::from("Num")),
    ("Bool", String::from("Bool")),
    ("Array", String::from("Array")),
  ]);
  uniquify_help(e, translation_table)
}
