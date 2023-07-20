use crate::asm::instrs_to_string;
use crate::errors::CompileErr;
use crate::pipeline_functions::print_prog::{print_prog, print_sprog};
use crate::pipeline_functions::{
  check_prog::check_prog, compile_to_instrs::compile_to_instrs, lambda_lift::lambda_lift,
  resolve_types::resolve_types, sequentialize::sequentialize, uniquify::uniquify,
};
use crate::syntax::{Exp, FunDecl, SeqProg, SurfProg};

fn tag_exp<Ann>(p: &SurfProg<Ann>) -> SurfProg<u32> {
  let mut i = 0;
  p.map_ann(
    &mut (|_| {
      let cur = i;
      i += 1;
      cur
    }),
  )
}

fn tag_prog<Ann>(
  defs: &[FunDecl<Exp<Ann>, Ann>],
  main: &Exp<Ann>,
) -> (Vec<FunDecl<Exp<u32>, u32>>, Exp<u32>) {
  let mut i = 0;
  (
    defs
      .iter()
      .map(|decl| {
        decl.map_ann(
          &mut (|_| {
            let cur = i;
            i += 1;
            cur
          }),
        )
      })
      .collect(),
    main.map_ann(
      &mut (|_| {
        let cur = i;
        i += 1;
        cur
      }),
    ),
  )
}

fn tag_sprog<Ann>(p: &SeqProg<Ann>) -> SeqProg<u32> {
  let mut i = 0;
  p.map_ann(
    &mut (|_| {
      let cur = i;
      i += 1;
      cur
    }),
  )
}

use std::collections::HashMap;
fn generate_typenames_file(lt: &HashMap<String, u64>) -> () {
  use std::fs::File;
  use std::io::prelude::*;
  use std::path::Path;
  let mut v = Vec::new();
  v.resize(lt.len(), String::from(""));
  for (ctype, tag) in lt {
    let ctype = ctype.clone().replacen("__custom_type_", "", 1);
    let ctype = ctype.split_once("_").unwrap().1;
    v[*tag as usize] = format!("\"{}\"", ctype);
  }
  let output = format!("vec![{}]", v.join(", "));
  let path = Path::new("runtime").join("custom_types.txt");
  println!("{}", path.display());
  let mut file = match File::create(&path) {
    Err(why) => panic!("Error opening custom types txt file: {}", why),
    Ok(file) => file,
  };
  match file.write_all(output.as_bytes()) {
    Err(why) => panic!("Error writing custom types txt file: {}", why),
    Ok(_) => (),
  }
}

pub fn compile_to_string<Span>(prog: &SurfProg<Span>) -> Result<String, CompileErr<Span>>
where
  Span: Clone + std::fmt::Debug,
{
  let print_before_seq = true;
  let print_after_seq = false;
  // println!("{:#?}", prog);

  // first check for errors
  check_prog(prog)?;
  // then give all the variables unique names
  let uniq_prog = uniquify(&tag_exp(prog));
  // and tag the program again so that we can name resolve types
  let tagged_uniq_prog = tag_exp(&uniq_prog);
  // now let's resolve those types and get a lookup table for type tags
  let (resolved_types_exp, custom_types) = resolve_types(&tagged_uniq_prog);
  generate_typenames_file(&custom_types);
  // tag the program again to name lambdas in lambda_lift
  let tagged_resolve_type_exp = tag_exp(&resolved_types_exp);
  // lift definitions to the top level
  let (defs, main) = lambda_lift(&tagged_resolve_type_exp);

  if print_before_seq {
    println!(
      "{}",
      format!(
        "{}\nin",
        defs
          .iter()
          .map(|decl| format!(
            "def {}({}):\n{}",
            decl.name,
            decl.parameters.join(", "),
            print_prog(&decl.body, 2)
          ))
          .collect::<Vec<String>>()
          .join("\nand\n"),
      )
    );
    println!("{}\n\n\n", print_prog(&main, 0));
  }

  let (t_defs, t_main) = tag_prog(&defs, &main);
  // then sequentialize
  let seq_p = tag_sprog(&sequentialize(&t_defs, &t_main));

  if print_after_seq {
    println!(
      "{}",
      format!(
        "{}\nin",
        seq_p
          .funs
          .iter()
          .map(|decl| format!(
            "def {}({}):\n{}",
            decl.name,
            decl.parameters.join(", "),
            print_sprog(&decl.body, 3)
          ))
          .collect::<Vec<String>>()
          .join("\nand\n"),
      )
    );
    println!("{}", print_sprog(&seq_p.main, 0));
  }

  // then codegen
  let code = format!(
    "
        section .data
HEAP_START:   times 999999 dq 0
        section .text
        extern snake_error
        extern print_snake_val
        global start_here
start_here:
        push rbp                     ; rbp is callee-saved
        mov rbp, HEAP_START          ; use rbp as heap pointer
        sub rsp, 8                   ; keep stack aligned
        call __snake__main
        add rsp, 8
        pop rbp                      ; restore rbp
        ret
{}
",
    instrs_to_string(&compile_to_instrs(&seq_p))
  );
  println!("{}", code);
  Ok(code)
}
