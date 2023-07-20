use crate::asm::{Arg32, Arg64, BinArgs, Instr, JmpArg, MemRef, MovArgs, Offset, Reg, Reg32};
use crate::pipeline_functions::space_needed::space_needed;
use crate::syntax::{ImmExp, Prim1, Prim2, SeqExp, SeqProg};
use std::collections::HashMap;

type ErrorCode = u64;
static ARITH_ERROR: ErrorCode = 0;
static COMPARISON_ERROR: ErrorCode = 1;
static IF_ERROR: ErrorCode = 2;
static LOGIC_ERROR: ErrorCode = 3;
static OVERFLOW_ERROR: ErrorCode = 4;
static NOT_ARRAY: ErrorCode = 5;
static INDEX_OUT_OF_BOUNDS: ErrorCode = 6;
static INDEX_NOT_NUMBER: ErrorCode = 7;
static CALLED_NON_FUNCTION: ErrorCode = 8;
static WRONG_ARITY: ErrorCode = 9;
static LENGTH_NON_ARRAY: ErrorCode = 10;

/*
  Numbers: 0 in the least significant bit
  Booleans: 111 in the three least significant bits
  Arrays: 001 in the three least significant bits
  Closures: 011 in the three least significant bits
*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnakeType {
    Number,
    Boolean,
    Array,
    Closure,
}

#[derive(Clone, Copy)]
struct RawSnakeVal(u64);
impl From<bool> for RawSnakeVal {
    fn from(b: bool) -> RawSnakeVal {
        if b {
            return SNAKE_TRU;
        } else {
            return SNAKE_FLS;
        }
    }
}
impl From<i64> for RawSnakeVal {
    fn from(n: i64) -> RawSnakeVal {
        RawSnakeVal(signed_to_unsigned(n))
    }
}

static TAG_CHECKER: u64 = 0b111;
static BOOL_TAG: u32 = 0b111;
static ARRAY_TAG: u32 = 0b001;
static CLOSURE_TAG: u32 = 0b011;

static SNAKE_TRU: RawSnakeVal = RawSnakeVal(0xFF_FF_FF_FF_FF_FF_FF_FF);
static SNAKE_FLS: RawSnakeVal = RawSnakeVal(0x7F_FF_FF_FF_FF_FF_FF_FF);

fn usize_to_i32(x: usize) -> i32 {
    use std::convert::TryInto;
    x.try_into().unwrap()
}

fn signed_to_unsigned(x: i64) -> u64 {
    u64::from_le_bytes(x.to_le_bytes())
}

enum CallingConvention {
    SystemV,
    Snake,
}

fn stack_align(sz: u32, cc: CallingConvention) -> u32 {
    // we will later do a `call` which pushes a value onto the stack
    match cc {
        // Snake calling convention: upon entry into callee, RSP should be divisible by 16
        // so here make RSP+8 divisible by 16
        CallingConvention::Snake => match sz % 16 {
            8 => sz,
            0 => sz + 8,
            _ => panic!("stack size {} does not look right", sz),
        },
        // SystemV calling convention: upon entry into callee, RSP+8 should be divisible by 16
        // so here make RSP divisible by 16
        CallingConvention::SystemV => match sz % 16 {
            8 => sz + 8,
            0 => sz,
            _ => panic!("stack size {} does not look right", sz),
        },
    }
}

pub fn compile_to_instrs(p: &SeqProg<u32>) -> Vec<Instr> {
    fn alloc<Ann>(stack_lt: &HashMap<&str, Ann>) -> i32 {
        usize_to_i32(stack_lt.len() + 1)
    }
    fn get_mem(mem_addr: i32) -> MemRef {
        MemRef {
            reg: Reg::Rsp,
            offset: Offset::Constant(-8 * mem_addr),
        }
    }
    fn compile_immediate_help(e: &ImmExp, stack_lt: &HashMap<&str, i32>, reg: Reg) -> Instr {
        match e {
            ImmExp::Num(val) => Instr::Mov(MovArgs::ToReg(reg, Arg64::Signed(*val * 2))),
            ImmExp::Bool(val) => Instr::Mov(MovArgs::ToReg(
                reg,
                Arg64::Unsigned(RawSnakeVal::from(*val).0),
            )),
            ImmExp::Var(name) => {
                let mem_addr = stack_lt.get(name.as_str()).unwrap();
                Instr::Mov(MovArgs::ToReg(reg, Arg64::Mem(get_mem(*mem_addr))))
            }
        }
    }
    fn compile_to_instrs_help(
        expr: &SeqExp<u32>,
        stack_lt: HashMap<&str, i32>,
        sf_size: u32,
        is_tail: bool,
    ) -> Vec<Instr> {
        fn generate_type_check(
            reg: Reg,
            expected_type: SnakeType,
            error_code: ErrorCode,
            tag: &str,
        ) -> Vec<Instr> {
            let scratch_reg = Reg::R10;
            if scratch_reg == reg {
                panic!(
          "Cannot generate type check for R10 as it is used as scratch register in type check"
        );
            }
            let mut instrs: Vec<Instr> = vec![Instr::Comment(format!(
                "type check: {:?} is {:?}",
                reg, expected_type
            ))];
            let type_pass_label = format!("typecheck_pass_{}", tag);

            match expected_type {
                SnakeType::Number => instrs.extend(
                    [
                        Instr::Mov(MovArgs::ToReg(scratch_reg, Arg64::Unsigned(1))),
                        Instr::Test(BinArgs::ToReg(reg, Arg32::Reg(scratch_reg))),
                        Instr::Jz(JmpArg::Label(type_pass_label.clone())),
                    ]
                    .to_vec(),
                ),
                _ => {
                    let expected_tag = match expected_type {
                        SnakeType::Number => unreachable!(),
                        SnakeType::Boolean => BOOL_TAG,
                        SnakeType::Array => ARRAY_TAG,
                        SnakeType::Closure => CLOSURE_TAG,
                    };
                    instrs.extend(
                        [
                            Instr::Mov(MovArgs::ToReg(scratch_reg, Arg64::Unsigned(TAG_CHECKER))),
                            Instr::And(BinArgs::ToReg(scratch_reg, Arg32::Reg(reg))),
                            Instr::Cmp(BinArgs::ToReg(scratch_reg, Arg32::Unsigned(expected_tag))),
                            Instr::Je(JmpArg::Label(type_pass_label.clone())),
                        ]
                        .to_vec(),
                    );
                }
            }

            instrs.extend(
                [
                    Instr::Mov(MovArgs::ToReg(Reg::Rdi, Arg64::Unsigned(error_code))),
                    Instr::Mov(MovArgs::ToReg(Reg::Rsi, Arg64::Reg(reg))),
                    Instr::Call(JmpArg::Label("snake_error".to_string())),
                    Instr::Label(type_pass_label.clone()),
                ]
                .to_vec(),
            );

            instrs
        }
        fn put_snakeval_in_rax(sv: RawSnakeVal) -> Instr {
            Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Unsigned(sv.0)))
        }
        fn generate_overflow_check(tag: u32) -> Vec<Instr> {
            ([
                Instr::Comment(String::from("overflow check")),
                Instr::Jno(JmpArg::Label(format!("overflowcheck_pass_{}", tag))),
                Instr::Mov(MovArgs::ToReg(Reg::Rdi, Arg64::Unsigned(OVERFLOW_ERROR))),
                Instr::Call(JmpArg::Label("snake_error".to_string())),
                Instr::Label(format!("overflowcheck_pass_{}", tag)),
            ])
            .to_vec()
        }

        // index_reg is where index is stored
        // array_size_reg is the register storing the address on the heap where array size is
        fn generate_array_index_checks(
            index_reg: Reg,
            array_size_reg: Reg,
            tag: u32,
        ) -> Vec<Instr> {
            let index_out_of_bounds = format!("index_out_of_bounds_{}", tag);
            let index_in_bounds = format!("index_in_bounds_{}", tag);
            [
                // Check index < 0
                Instr::Cmp(BinArgs::ToReg(index_reg, Arg32::Unsigned(0))),
                Instr::Jl(JmpArg::Label(index_out_of_bounds.clone())),
                Instr::Cmp(BinArgs::ToReg(
                    index_reg,
                    Arg32::Mem(MemRef {
                        reg: array_size_reg,
                        offset: Offset::Constant(0),
                    }),
                )),
                Instr::Jge(JmpArg::Label(index_out_of_bounds.clone())),
                Instr::Jmp(JmpArg::Label(index_in_bounds.clone())),
                Instr::Label(index_out_of_bounds).clone(),
                Instr::Mov(MovArgs::ToReg(
                    Reg::Rdi,
                    Arg64::Unsigned(INDEX_OUT_OF_BOUNDS),
                )),
                Instr::Mov(MovArgs::ToReg(Reg::Rsi, Arg64::Reg(index_reg))),
                Instr::Call(JmpArg::Label("snake_error".to_string())),
                Instr::Label(index_in_bounds.clone()),
            ]
            .to_vec()
        }

        fn generate_arity_check(func_ptr_reg: Reg, num_args: u32, tag: u32) -> Vec<Instr> {
            let done_label = format!("arity_check_passed_{}", tag);
            [
                Instr::Comment(format!(
                    "arity check: {:?} receives {} args",
                    func_ptr_reg, num_args
                )),
                // move arity into R11
                Instr::Mov(MovArgs::ToReg(
                    Reg::R11,
                    Arg64::Mem(MemRef {
                        reg: func_ptr_reg,
                        offset: Offset::Constant(0),
                    }),
                )),
                Instr::Cmp(BinArgs::ToReg(Reg::R11, Arg32::Unsigned(num_args))),
                Instr::Je(JmpArg::Label(done_label.clone())),
                Instr::Mov(MovArgs::ToReg(Reg::Rdi, Arg64::Unsigned(WRONG_ARITY))),
                Instr::Mov(MovArgs::ToReg(Reg::Rsi, Arg64::Reg(Reg::R11))),
                Instr::Mov(MovArgs::ToReg(Reg::Rdx, Arg64::Unsigned(num_args as u64))),
                Instr::Call(JmpArg::Label("snake_error".to_string())),
                Instr::Label(done_label.clone()),
            ]
            .to_vec()
        }

        let mut is = Vec::new();

        match expr {
            SeqExp::Imm(imm, _) => {
                is.push(compile_immediate_help(imm, &stack_lt, Reg::Rax));
            }
            SeqExp::Prim1(op, e1, tag) => {
                is.push(compile_immediate_help(e1, &stack_lt, Reg::Rax));
                match op {
                    Prim1::Add1 | Prim1::Sub1 => {
                        is.extend(generate_type_check(
                            Reg::Rax,
                            SnakeType::Number,
                            ARITH_ERROR,
                            &format!("1_{}", *tag),
                        ));
                    }
                    Prim1::Not => {
                        is.extend(generate_type_check(
                            Reg::Rax,
                            SnakeType::Boolean,
                            LOGIC_ERROR,
                            &format!("2_{}", *tag),
                        ));
                    }
                    Prim1::Length => is.extend(generate_type_check(
                        Reg::Rax,
                        SnakeType::Array,
                        LENGTH_NON_ARRAY,
                        &format!("1_{}", *tag),
                    )),
                    _ => (),
                }
                match op {
                    Prim1::Add1 => {
                        is.push(Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Signed(2))));
                    }
                    Prim1::Sub1 => {
                        is.push(Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Signed(2))));
                    }
                    Prim1::Not => is.extend([
                        Instr::Mov(MovArgs::ToReg(Reg::R11, Arg64::Unsigned(SNAKE_TRU.0))),
                        Instr::Cmp(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R11))),
                        Instr::Je(JmpArg::Label(format!("not_true_{}", tag))),
                        put_snakeval_in_rax(SNAKE_TRU),
                        Instr::Jmp(JmpArg::Label(format!("not_end_{}", tag))),
                        Instr::Label(format!("not_true_{}", tag)),
                        put_snakeval_in_rax(SNAKE_FLS),
                        Instr::Label(format!("not_end_{}", tag)),
                    ]),
                    Prim1::IsBool => is.extend([
                        Instr::Mov(MovArgs::ToReg(Reg::R10, Arg64::Unsigned(TAG_CHECKER))),
                        Instr::And(BinArgs::ToReg(Reg::R10, Arg32::Reg(Reg::Rax))),
                        Instr::Cmp(BinArgs::ToReg(Reg::R10, Arg32::Unsigned(BOOL_TAG))),
                        put_snakeval_in_rax(SNAKE_TRU),
                        Instr::Je(JmpArg::Label(format!("isbool_true_{}", tag))),
                        put_snakeval_in_rax(SNAKE_FLS),
                        Instr::Label(format!("isbool_true_{}", tag)),
                    ]),
                    Prim1::IsNum => is.extend([
                        Instr::Mov(MovArgs::ToReg(Reg::R11, Arg64::Unsigned(1))),
                        Instr::Test(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R11))),
                        Instr::Jz(JmpArg::Label(format!("isnum_true_{}", tag))),
                        put_snakeval_in_rax(SNAKE_FLS),
                        Instr::Jmp(JmpArg::Label(format!("isnum_end_{}", tag))),
                        Instr::Label(format!("isnum_true_{}", tag)),
                        put_snakeval_in_rax(SNAKE_TRU),
                        Instr::Label(format!("isnum_end_{}", tag)),
                    ]),
                    Prim1::Print => {
                        let stack_offset = stack_align(sf_size, CallingConvention::SystemV);
                        is.extend([
                            Instr::Mov(MovArgs::ToReg(Reg::Rdi, Arg64::Reg(Reg::Rax))),
                            Instr::Sub(BinArgs::ToReg(Reg::Rsp, Arg32::Unsigned(stack_offset))),
                            Instr::Call(JmpArg::Label("print_snake_val".to_string())),
                            Instr::Add(BinArgs::ToReg(Reg::Rsp, Arg32::Unsigned(stack_offset))),
                        ])
                    }
                    Prim1::Length => {
                        is.extend(
                            [
                                // untag array
                                Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(1))),
                                // return length
                                Instr::Mov(MovArgs::ToReg(
                                    Reg::Rax,
                                    Arg64::Mem(MemRef {
                                        reg: Reg::Rax,
                                        offset: Offset::Constant(0),
                                    }),
                                )),
                                // tag length value
                                Instr::Shl(BinArgs::ToReg(Reg::Rax, Arg32::Signed(1))),
                            ]
                            .to_vec(),
                        )
                    }
                    Prim1::IsArray => is.extend(
                        [
                            Instr::Mov(MovArgs::ToReg(Reg::R10, Arg64::Unsigned(TAG_CHECKER))),
                            Instr::And(BinArgs::ToReg(Reg::R10, Arg32::Reg(Reg::Rax))),
                            Instr::Cmp(BinArgs::ToReg(Reg::R10, Arg32::Unsigned(ARRAY_TAG))),
                            put_snakeval_in_rax(SNAKE_TRU),
                            Instr::Je(JmpArg::Label(format!("isarray_true_{}", tag))),
                            put_snakeval_in_rax(SNAKE_FLS),
                            Instr::Label(format!("isarray_true_{}", tag)),
                        ]
                        .to_vec(),
                    ),
                    Prim1::IsFun => is.extend(
                        [
                            Instr::Mov(MovArgs::ToReg(Reg::R10, Arg64::Unsigned(TAG_CHECKER))),
                            Instr::And(BinArgs::ToReg(Reg::R10, Arg32::Reg(Reg::Rax))),
                            Instr::Cmp(BinArgs::ToReg(Reg::R10, Arg32::Unsigned(CLOSURE_TAG))),
                            put_snakeval_in_rax(SNAKE_TRU),
                            Instr::Je(JmpArg::Label(format!("isfun_true_{}", tag))),
                            put_snakeval_in_rax(SNAKE_FLS),
                            Instr::Label(format!("isfun_true_{}", tag)),
                        ]
                        .to_vec(),
                    ),
                };
                match op {
                    // check for overflow
                    Prim1::Add1 | Prim1::Sub1 => {
                        is.extend(generate_overflow_check(*tag));
                    }
                    _ => (),
                }
            }
            SeqExp::Prim2(op, e1, e2, tag) => {
                is.push(compile_immediate_help(e1, &stack_lt, Reg::Rax));
                is.push(compile_immediate_help(e2, &stack_lt, Reg::R11));
                match op {
                    // first do type checking
                    Prim2::Add | Prim2::Sub | Prim2::Mul => {
                        is.extend(generate_type_check(
                            Reg::Rax,
                            SnakeType::Number,
                            ARITH_ERROR,
                            &format!("1_{}", *tag),
                        ));
                        is.extend(generate_type_check(
                            Reg::R11,
                            SnakeType::Number,
                            ARITH_ERROR,
                            &format!("2_{}", *tag),
                        ));
                    }
                    Prim2::Lt | Prim2::Gt | Prim2::Le | Prim2::Ge => {
                        is.extend(generate_type_check(
                            Reg::Rax,
                            SnakeType::Number,
                            COMPARISON_ERROR,
                            &format!("1_{}", *tag),
                        ));
                        is.extend(generate_type_check(
                            Reg::R11,
                            SnakeType::Number,
                            COMPARISON_ERROR,
                            &format!("2_{}", *tag),
                        ));
                    }
                    Prim2::And | Prim2::Or => {
                        is.extend(generate_type_check(
                            Reg::Rax,
                            SnakeType::Boolean,
                            LOGIC_ERROR,
                            &format!("1_{}", *tag),
                        ));
                        is.extend(generate_type_check(
                            Reg::R11,
                            SnakeType::Boolean,
                            LOGIC_ERROR,
                            &format!("2_{}", *tag),
                        ));
                    }
                    Prim2::Eq | Prim2::Neq => (),
                    Prim2::ArrayGet => {
                        is.extend(generate_type_check(
                            Reg::Rax,
                            SnakeType::Array,
                            NOT_ARRAY,
                            &format!("1_{}", *tag),
                        ));
                        is.extend(generate_type_check(
                            Reg::R11,
                            SnakeType::Number,
                            INDEX_NOT_NUMBER,
                            &format!("2_{}", *tag),
                        ));
                    }
                }
                match op {
                    // then generate the instructions
                    Prim2::Add => {
                        is.push(Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R11))));
                    }
                    Prim2::Sub => {
                        is.push(Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R11))));
                    }
                    Prim2::Mul => {
                        is.push(Instr::Sar(BinArgs::ToReg(Reg::Rax, Arg32::Signed(1))));
                        is.push(Instr::IMul(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R11))));
                    }
                    Prim2::And => is.extend([
                        Instr::Mov(MovArgs::ToReg(Reg::R10, Arg64::Unsigned(SNAKE_TRU.0))),
                        Instr::Cmp(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))),
                        Instr::Je(JmpArg::Label(format!("and_operand1_true_{}", tag))),
                        put_snakeval_in_rax(SNAKE_FLS),
                        Instr::Jmp(JmpArg::Label(format!("and_end_{}", tag))),
                        Instr::Label(format!("and_operand1_true_{}", tag)),
                        Instr::Cmp(BinArgs::ToReg(Reg::R11, Arg32::Reg(Reg::R10))),
                        Instr::Je(JmpArg::Label(format!("and_operand2_true_{}", tag))),
                        put_snakeval_in_rax(SNAKE_FLS),
                        Instr::Jmp(JmpArg::Label(format!("and_end_{}", tag))),
                        Instr::Label(format!("and_operand2_true_{}", tag)),
                        put_snakeval_in_rax(SNAKE_TRU),
                        Instr::Label(format!("and_end_{}", tag)),
                    ]),
                    Prim2::Or => is.extend([
                        Instr::Mov(MovArgs::ToReg(Reg::R10, Arg64::Unsigned(SNAKE_TRU.0))),
                        Instr::Cmp(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R10))),
                        Instr::Je(JmpArg::Label(format!("or_true_{}", tag))),
                        Instr::Cmp(BinArgs::ToReg(Reg::R11, Arg32::Reg(Reg::R10))),
                        Instr::Je(JmpArg::Label(format!("or_true_{}", tag))),
                        put_snakeval_in_rax(SNAKE_FLS),
                        Instr::Jmp(JmpArg::Label(format!("or_end_{}", tag))),
                        Instr::Label(format!("or_true_{}", tag)),
                        put_snakeval_in_rax(SNAKE_TRU),
                        Instr::Label(format!("or_end_{}", tag)),
                    ]),
                    Prim2::Lt | Prim2::Gt | Prim2::Le | Prim2::Ge | Prim2::Eq | Prim2::Neq => {
                        // shift right both operands while preserving signs
                        is.extend([
                            Instr::Sar(BinArgs::ToReg(Reg::Rax, Arg32::Signed(1))),
                            Instr::Sar(BinArgs::ToReg(Reg::R11, Arg32::Signed(1))),
                            Instr::Cmp(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R11))),
                            match op {
                                Prim2::Lt => Instr::Jl(JmpArg::Label(format!("cmp_true_{}", tag))),
                                Prim2::Gt => Instr::Jg(JmpArg::Label(format!("cmp_true_{}", tag))),
                                Prim2::Le => Instr::Jle(JmpArg::Label(format!("cmp_true_{}", tag))),
                                Prim2::Ge => Instr::Jge(JmpArg::Label(format!("cmp_true_{}", tag))),
                                Prim2::Eq => Instr::Je(JmpArg::Label(format!("cmp_true_{}", tag))),
                                Prim2::Neq => {
                                    Instr::Jne(JmpArg::Label(format!("cmp_true_{}", tag)))
                                }
                                _ => unreachable!(),
                            },
                            put_snakeval_in_rax(SNAKE_FLS),
                            Instr::Jmp(JmpArg::Label(format!("cmp_end_{}", tag))),
                            Instr::Label(format!("cmp_true_{}", tag)),
                            put_snakeval_in_rax(SNAKE_TRU),
                            Instr::Label(format!("cmp_end_{}", tag)),
                        ])
                    }
                    Prim2::ArrayGet => {
                        // untag array
                        is.push(Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(1))));
                        // untag index
                        is.push(Instr::Sar(BinArgs::ToReg(Reg::R11, Arg32::Signed(1))));
                        // confirm index is in bounds
                        is.extend(generate_array_index_checks(Reg::R11, Reg::Rax, *tag));
                        is.push(Instr::Mov(MovArgs::ToReg(
                            Reg::Rax,
                            Arg64::Mem(MemRef {
                                reg: Reg::Rax,
                                offset: Offset::Computed {
                                    reg: Reg::R11,
                                    factor: 8,
                                    constant: 8,
                                },
                            }),
                        )));
                    }
                }
                match op {
                    // check for overflow
                    Prim2::Add | Prim2::Sub | Prim2::Mul => {
                        is.extend(generate_overflow_check(*tag));
                    }
                    _ => (),
                }
            }
            SeqExp::Let {
                var,
                bound_exp,
                body,
                ann: _,
            } => {
                // evaluate the expression first, and store the result in Rax
                is.extend(compile_to_instrs_help(
                    bound_exp,
                    stack_lt.clone(),
                    sf_size,
                    false,
                ));
                // create a new environment for the let block
                let mut new_stack_lt = stack_lt.clone();
                // allocate an address for the new variable
                let mem_addr = alloc(&new_stack_lt);
                is.push(Instr::Comment(format!(
                    "storing {} into [rsp - {}]",
                    var,
                    mem_addr * 8
                )));
                // actually save the value in memory
                is.push(Instr::Mov(MovArgs::ToMem(
                    get_mem(mem_addr),
                    Reg32::Reg(Reg::Rax),
                )));
                // register the new variable in the stack lookup table
                new_stack_lt.insert(var.as_str(), mem_addr);
                // normally compile the body with the new environment
                is.extend(compile_to_instrs_help(body, new_stack_lt, sf_size, is_tail));
            }
            SeqExp::If {
                cond,
                thn,
                els,
                ann: tag,
            } => {
                is.push(compile_immediate_help(cond, &stack_lt, Reg::Rax));
                is.extend(generate_type_check(
                    Reg::Rax,
                    SnakeType::Boolean,
                    IF_ERROR,
                    &tag.to_string(),
                ));
                is.push(Instr::Mov(MovArgs::ToReg(
                    Reg::R11,
                    Arg64::Unsigned(SNAKE_FLS.0),
                )));
                is.push(Instr::Cmp(BinArgs::ToReg(Reg::Rax, Arg32::Reg(Reg::R11))));
                is.push(Instr::Je(JmpArg::Label(format!("if_false_{}", tag))));
                is.extend(compile_to_instrs_help(
                    thn,
                    stack_lt.clone(),
                    sf_size,
                    is_tail,
                ));
                is.push(Instr::Jmp(JmpArg::Label(format!("if_end_{}", tag))));
                is.push(Instr::Label(format!("if_false_{}", tag)));
                is.extend(compile_to_instrs_help(els, stack_lt, sf_size, is_tail));
                is.push(Instr::Label(format!("if_end_{}", tag)));
            }
            SeqExp::Array(array_values, _) => {
                let array_size = array_values.len();
                // Push the array size to the heap
                is.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rbp,
                        offset: Offset::Constant(0),
                    },
                    Reg32::Unsigned(array_size as u32),
                )));

                // Push each array element to the heap
                for (i, array_val) in array_values.iter().enumerate() {
                    is.push(compile_immediate_help(array_val, &stack_lt, Reg::Rax));
                    is.push(Instr::Mov(MovArgs::ToMem(
                        MemRef {
                            reg: Reg::Rbp,
                            offset: Offset::Constant(8 * usize_to_i32(i + 1)),
                        },
                        Reg32::Reg(Reg::Rax),
                    )));
                }

                // start creating the tuple value itself
                is.push(Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(Reg::Rbp))));
                // tag the tuple
                is.push(Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(1))));
                // make Rbp point to the next available value on the heap
                is.push(Instr::Add(BinArgs::ToReg(
                    Reg::Rbp,
                    Arg32::Unsigned(8 * (array_size as u32 + 1)),
                )));
            }
            SeqExp::ArraySet {
                array,
                index,
                new_value,
                ann: tag,
            } => {
                is.push(compile_immediate_help(array, &stack_lt, Reg::Rax));
                is.extend(generate_type_check(
                    Reg::Rax,
                    SnakeType::Array,
                    NOT_ARRAY,
                    format!("array_set_{}", tag).as_str(),
                ));
                // untag array
                is.push(Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(1))));
                // load index into R11
                is.push(compile_immediate_help(index, &stack_lt, Reg::R11));
                is.extend(generate_type_check(
                    Reg::R11,
                    SnakeType::Number,
                    INDEX_NOT_NUMBER,
                    format!("arr_set_idx_{}", tag).as_str(),
                ));
                // untag R11
                is.push(Instr::Sar(BinArgs::ToReg(Reg::R11, Arg32::Signed(1))));
                // load new value into R10
                is.push(compile_immediate_help(new_value, &stack_lt, Reg::R10));
                // perform mutation
                is.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rax,
                        offset: Offset::Computed {
                            reg: Reg::R11,
                            factor: 8,
                            constant: 8,
                        },
                    },
                    Reg32::Reg(Reg::R10),
                )));

                // tag array
                is.push(Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(1))));
            }
            SeqExp::CallClosure {
                fun,
                args,
                ann: tag,
            } => {
                // confirm we have a closure
                is.push(compile_immediate_help(fun, &stack_lt, Reg::Rax));
                is.extend(generate_type_check(
                    Reg::Rax,
                    SnakeType::Closure,
                    CALLED_NON_FUNCTION,
                    format!("call_closure_{}", tag).as_str(),
                ));
                // untag closure
                is.push(Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(3))));
                // check and confirm function arity
                is.extend(generate_arity_check(Reg::Rax, args.len() as u32, *tag));
                // calculate offset, used by non-tail calls
                let stack_offset = stack_align(sf_size, CallingConvention::Snake);
                // push environment onto stack for function call
                is.push(Instr::Mov(MovArgs::ToReg(
                    Reg::R11,
                    Arg64::Mem(MemRef {
                        reg: Reg::Rax,
                        offset: Offset::Constant(16),
                    }),
                )));
                is.push(Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rsp,
                        offset: Offset::Constant(
                            -8 - if is_tail { 0 } else { stack_offset + 8 } as i32,
                        ),
                    },
                    Reg32::Reg(Reg::R11),
                )));
                // push every other parameter onto the stack for function call
                for (i, param) in args.iter().enumerate() {
                    match param {
                        ImmExp::Var(name) => {
                            let memref = get_mem(*stack_lt.get(name.as_str()).unwrap());
                            is.push(Instr::Mov(MovArgs::ToReg(Reg::R11, Arg64::Mem(memref))));
                            is.push(Instr::Mov(MovArgs::ToMem(
                                MemRef {
                                    reg: Reg::Rsp,
                                    // for tail calls, first argument is put at rsp-8
                                    // non-tail calls need to reserve rsp-8 for return address, so
                                    // arguments start at rsp-16
                                    offset: Offset::Constant(
                                        -8 * (i + 2) as i32
                                            - if is_tail { 0 } else { stack_offset + 8 } as i32,
                                    ),
                                },
                                Reg32::Reg(Reg::R11),
                            )))
                        }
                        _ => unreachable!(),
                    }
                }
                if is_tail {
                    // store address to jump to into R11
                    is.push(Instr::Mov(MovArgs::ToReg(
                        Reg::R11,
                        Arg64::Mem(MemRef {
                            reg: Reg::Rax,
                            offset: Offset::Constant(8),
                        }),
                    )));
                    is.push(Instr::Jmp(JmpArg::Reg(Reg::R11)));
                } else {
                    // skip the current stack frame
                    is.push(Instr::Sub(BinArgs::ToReg(
                        Reg::Rsp,
                        Arg32::Unsigned(stack_offset),
                    )));
                    // store function label into R11
                    is.push(Instr::Mov(MovArgs::ToReg(
                        Reg::R11,
                        Arg64::Mem(MemRef {
                            reg: Reg::Rax,
                            offset: Offset::Constant(8),
                        }),
                    )));
                    // call function
                    is.push(Instr::Call(JmpArg::Reg(Reg::R11)));
                    // pull the stack pointer back
                    is.push(Instr::Add(BinArgs::ToReg(
                        Reg::Rsp,
                        Arg32::Unsigned(stack_offset),
                    )));
                }
            }
            SeqExp::MakeClosure {
                arity,
                label,
                env,
                ann: _,
            } => {
                is.extend([
                    // push the function arity to the heap
                    Instr::Mov(MovArgs::ToMem(
                        MemRef {
                            reg: Reg::Rbp,
                            offset: Offset::Constant(0),
                        },
                        Reg32::Unsigned(*arity as u32),
                    )),
                    // push the function address to the heap
                    Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Label(label.to_string()))),
                    Instr::Mov(MovArgs::ToMem(
                        MemRef {
                            reg: Reg::Rbp,
                            offset: Offset::Constant(8),
                        },
                        Reg32::Reg(Reg::Rax),
                    )),
                    // push function env to the heap
                    compile_immediate_help(env, &stack_lt, Reg::Rax),
                    Instr::Mov(MovArgs::ToMem(
                        MemRef {
                            reg: Reg::Rbp,
                            offset: Offset::Constant(16),
                        },
                        Reg32::Reg(Reg::Rax),
                    )),
                    // start creating the closure value itself
                    Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(Reg::Rbp))),
                    // tag the closure
                    Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(3))),
                    // make Rbp point to the next available value on the heap
                    Instr::Add(BinArgs::ToReg(Reg::Rbp, Arg32::Unsigned(24))),
                ])
            }
            SeqExp::MakeTypeInstance {
                typetag, // NOTE: make this a u32 instead
                fields,
                ann: _,
            } => is.extend([
                // push the type tag onto the heap
                Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rbp,
                        offset: Offset::Constant(0),
                    },
                    Reg32::Unsigned(*typetag as u32),
                )),
                // push the fields array onto the heap
                compile_immediate_help(fields, &stack_lt, Reg::Rax),
                Instr::Mov(MovArgs::ToMem(
                    MemRef {
                        reg: Reg::Rbp,
                        offset: Offset::Constant(8),
                    },
                    Reg32::Reg(Reg::Rax),
                )),
                // mark the heap pointer
                Instr::Mov(MovArgs::ToReg(Reg::Rax, Arg64::Reg(Reg::Rbp))),
                // tag it
                Instr::Add(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(0b101))),
                // pull heap pointer downwards
                Instr::Add(BinArgs::ToReg(Reg::Rbp, Arg32::Unsigned(16))),
            ]),
            SeqExp::MatchType {
                expr,
                typetag,
                ann: tag,
            } => {
                is.extend([
                    // load snakeval in Rax
                    compile_immediate_help(expr, &stack_lt, Reg::Rax),
                    // untag it
                    Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(0b101))),
                    // it is now a pointer to the type tag, extract the tag to R10
                    Instr::Mov(MovArgs::ToReg(
                        Reg::R10,
                        Arg64::Mem(MemRef {
                            reg: Reg::Rax,
                            offset: Offset::Constant(0),
                        }),
                    )),
                    // load the expected type tag in R11
                    Instr::Mov(MovArgs::ToReg(Reg::R11, Arg64::Unsigned(*typetag))),
                    // compare R10 and R11
                    Instr::Cmp(BinArgs::ToReg(Reg::R10, Arg32::Reg(Reg::R11))),
                    // if equal, type matches, return true
                    Instr::Je(JmpArg::Label(format!("matchtype_true_{}", tag))),
                    // otherwise return false
                    put_snakeval_in_rax(SNAKE_FLS),
                    Instr::Jmp(JmpArg::Label(format!("matchtype_end_{}", tag))),
                    Instr::Label(format!("matchtype_true_{}", tag)),
                    put_snakeval_in_rax(SNAKE_TRU),
                    Instr::Label(format!("matchtype_end_{}", tag)),
                ])
            }
            SeqExp::GetTypeFields(expr, tag) => {
                is.extend([
                    // load snakeval in Rax
                    compile_immediate_help(expr, &stack_lt, Reg::Rax),
                    // if the snakeval does not end in 0b101, return a dummy 0
                    Instr::Mov(MovArgs::ToReg(Reg::R10, Arg64::Unsigned(TAG_CHECKER))),
                    Instr::And(BinArgs::ToReg(Reg::R10, Arg32::Reg(Reg::Rax))),
                    Instr::Cmp(BinArgs::ToReg(Reg::R10, Arg32::Unsigned(0b101))),
                    Instr::Je(JmpArg::Label(format!("getfields_valid_{}", tag))),
                    put_snakeval_in_rax(RawSnakeVal(0)),
                    Instr::Jmp(JmpArg::Label(format!("getfields_end_{}", tag))),
                    Instr::Label(format!("getfields_valid_{}", tag)),
                    // otherwise fetch the field array
                    // untag it
                    Instr::Sub(BinArgs::ToReg(Reg::Rax, Arg32::Unsigned(0b101))),
                    // it is now a pointer to the type tag, extract the field array pointer to Rax
                    Instr::Mov(MovArgs::ToReg(
                        Reg::Rax,
                        Arg64::Mem(MemRef {
                            reg: Reg::Rax,
                            offset: Offset::Constant(8),
                        }),
                    )),
                    Instr::Label(format!("getfields_end_{}", tag)),
                ])
            }
        }
        is
    }
    let mut is: Vec<Instr> = Vec::new();
    // compile the main function
    // the main function is a snake function - it expects the stack to be aligned snakely
    // but that does not hold because Rust just called us
    // we solve the discrepancy by actually making it a function
    // this call re-aligns the stack since it pushes a value onto the stack
    is.push(Instr::Label(String::from("__snake__main")));

    is.extend(compile_to_instrs_help(
        &p.main,
        HashMap::new(),
        // main does not have parameters
        space_needed(&p.main, 0),
        true,
    ));

    is.push(Instr::Ret);
    // compile all other functions
    for funcdecl in p.funs.iter() {
        is.push(Instr::Label(funcdecl.name.clone()));
        let mut stack_lt: HashMap<&str, i32> = HashMap::new();
        // all parameters should be on the stack already
        for param in funcdecl.parameters.iter() {
            stack_lt.insert(param, alloc(&stack_lt));
        }
        // compile the function body, using an initial stack lookup table that contains all parameters
        is.extend(compile_to_instrs_help(
            &funcdecl.body,
            stack_lt,
            // we need to take into consideration the function parameters that also take stack space
            space_needed(&funcdecl.body, funcdecl.parameters.len() as u32 + 1),
            true,
        ));
        is.push(Instr::Ret);
    }
    is
}
