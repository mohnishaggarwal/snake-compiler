#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct RawSnakeVal(u64);
static SNAKE_TRU: RawSnakeVal = RawSnakeVal(0xFF_FF_FF_FF_FF_FF_FF_FF);
static SNAKE_FLS: RawSnakeVal = RawSnakeVal(0x7F_FF_FF_FF_FF_FF_FF_FF);

enum SnakeVal {
  Number(i64),
  Boolean(bool),
  Array(Vec<SnakeVal>),
  ArrayLoop,
  Closure(u64),
  Custom(u64, Vec<SnakeVal>),
  Invalid(RawSnakeVal),
}

impl From<RawSnakeVal> for SnakeVal {
  fn from(rv: RawSnakeVal) -> SnakeVal {
    use std::collections::HashSet;
    fn from_help(rv: RawSnakeVal, seen_arrays: HashSet<*const u64>) -> SnakeVal {
      if rv.0 % 2 == 0 {
        return SnakeVal::Number(unsigned_to_signed(rv.0) >> 1);
      }
      match rv.0 % 8 {
        0b111 if rv == SNAKE_TRU => SnakeVal::Boolean(true),
        0b111 if rv == SNAKE_FLS => SnakeVal::Boolean(false),
        0b001 => unsafe {
          let ptr_to_arr: *const u64 = std::mem::transmute(rv.0 - 0b001);
          let mut seen_arrays_updated = seen_arrays.clone();
          if !seen_arrays_updated.insert(ptr_to_arr) {
            return SnakeVal::ArrayLoop;
          }
          let mut elts = Vec::new();
          let arr = load_snake_array(ptr_to_arr);
          for idx in 0..arr.size {
            elts.push(from_help(
              *arr.elts.add(idx as usize),
              seen_arrays_updated.clone(),
            ));
          }
          SnakeVal::Array(elts)
        },
        0b011 => unsafe {
          let ptr_to_closure: *const RawSnakeVal = std::mem::transmute(rv.0 - 0b011);
          SnakeVal::Closure((*ptr_to_closure).0)
        },
        0b101 => unsafe {
          let ptr_to_val: *const RawSnakeVal = std::mem::transmute(rv.0 - 0b101);
          let type_tag: u64 = (*ptr_to_val).0;
          let farr = *ptr_to_val.add(1);
          match SnakeVal::from(farr) {
            SnakeVal::Array(elts) => SnakeVal::Custom(type_tag, elts),
            _ => unreachable!(),
          }
        },
        _ => SnakeVal::Invalid(rv),
      }
    }
    from_help(rv, HashSet::new())
  }
}

fn print_vec_internal(v: &Vec<SnakeVal>) -> String {
  v.iter()
    .map(|e| format!("{}", e))
    .collect::<Vec<String>>()
    .join(", ")
}

use std::fmt;
impl fmt::Display for SnakeVal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let type_lt: Vec<&str> = include!("custom_types.txt");
    write!(
      f,
      "{}",
      match self {
        SnakeVal::Number(n) => format!("{}", n),
        SnakeVal::Boolean(b) => format!("{}", b),
        SnakeVal::Array(elts) => format!("[{}]", print_vec_internal(elts)),
        SnakeVal::ArrayLoop => String::from("<loop>"),
        SnakeVal::Closure(_) => String::from("<closure>"),
        SnakeVal::Custom(tt, elts) =>
          if elts.is_empty() {
            format!("{}", type_lt[*tt as usize])
          } else {
            format!("{}({})", type_lt[*tt as usize], print_vec_internal(elts))
          },
        SnakeVal::Invalid(rv) => format!("invalid SnakeVal: {:x}", rv.0),
      }
    )
  }
}

#[repr(C)]
struct SnakeArray {
  size: u64,
  elts: *const RawSnakeVal,
}

/* You can use this function to cast a pointer to an array on the heap
 * into something more convenient to access
 *
 */
fn load_snake_array(p: *const u64) -> SnakeArray {
  unsafe {
    let size = *p;
    SnakeArray {
      size,
      elts: std::mem::transmute(p.add(1)),
    }
  }
}

type ErrorCode = u64;
const ARITH_ERROR: ErrorCode = 0;
const COMPARISON_ERROR: ErrorCode = 1;
const IF_ERROR: ErrorCode = 2;
const LOGIC_ERROR: ErrorCode = 3;
const OVERFLOW_ERROR: ErrorCode = 4;
const NOT_ARRAY: ErrorCode = 5;
const INDEX_OUT_OF_BOUNDS: ErrorCode = 6;
const INDEX_NOT_NUMBER: ErrorCode = 7;
const CALLED_NON_FUNCTION: ErrorCode = 8;
const WRONG_ARITY: ErrorCode = 9;
const LENGTH_NON_ARRAY: ErrorCode = 10;

#[link(name = "compiled_code", kind = "static")]
extern "sysv64" {
  // The \x01 here is an undocumented feature of LLVM that ensures
  // it does not add an underscore in front of the name.
  #[link_name = "\x01start_here"]
  fn start_here() -> RawSnakeVal;
}

// reinterprets the bytes of an unsigned number to a signed number
fn unsigned_to_signed(x: u64) -> i64 {
  i64::from_le_bytes(x.to_le_bytes())
}

fn sprint_snake_val(rv: RawSnakeVal) -> String {
  format!("{}", SnakeVal::from(rv))
}

#[export_name = "\x01print_snake_val"]
extern "sysv64" fn print_snake_val(rv: RawSnakeVal) -> RawSnakeVal {
  return rv;
}

#[export_name = "\x01snake_error"]
extern "sysv64" fn snake_error(err_code: ErrorCode, v1: RawSnakeVal, v2: RawSnakeVal) {
  match err_code {
        ARITH_ERROR => eprintln!(
            "arithmetic expected a number, but got {}",
            sprint_snake_val(v1)
        ),
        COMPARISON_ERROR => eprintln!(
            "comparison expected a number, but got {}",
            sprint_snake_val(v1)
        ),
        IF_ERROR => eprintln!("if expected a boolean, but got {}", sprint_snake_val(v1)),
        LOGIC_ERROR => eprintln!("logic expected a boolean, but got {}", sprint_snake_val(v1)),
        OVERFLOW_ERROR => eprintln!("arithmetic resulted in overflow"),
        NOT_ARRAY => eprintln!("indexed into non-array {}", sprint_snake_val(v1)),
        INDEX_OUT_OF_BOUNDS => eprintln!("index out of bounds: got {}", v1.0 as i64),
        INDEX_NOT_NUMBER => eprintln!("index not a number: got {}", sprint_snake_val(v1)),
        CALLED_NON_FUNCTION => eprintln!("called a non-function {} where a closure is expected", sprint_snake_val(v1)),
        WRONG_ARITY => eprintln!(
            "wrong number of arguments: {} expected, {} given",
            v1.0,
            v2.0
        ),
        LENGTH_NON_ARRAY => eprintln!("length called with non-array: {}", sprint_snake_val(v1)),
        _ => eprintln!(
          "I apologize to you, dear user. I made a bug. The error code is {}. Here's a snake value: {}.",
          err_code,
          sprint_snake_val(v1),
        ),
    }
  std::process::exit(1);
}

fn main() {
  let output = unsafe { start_here() };
  println!("{}", sprint_snake_val(output));
}
