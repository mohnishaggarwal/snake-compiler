use crate::syntax::SeqExp;

pub fn space_needed<Ann>(_e: &SeqExp<Ann>, paramc: u32) -> u32 {
  use std::cmp;
  fn space_needed_help<Ann>(_e: &SeqExp<Ann>) -> u32 {
    match _e {
      SeqExp::Let {
        var: _,
        bound_exp: _,
        body,
        ann: _,
      } => 8 + space_needed_help(body),
      SeqExp::If {
        cond: _,
        thn,
        els,
        ann: _,
      } => cmp::max(space_needed_help(thn), space_needed_help(els)),
      _ => 0,
    }
  }
  8 * paramc + space_needed_help(_e)
}
