#[derive(Debug, PartialEq, Eq)]
pub enum CompileErr<Span> {
  UnboundVariable {
    unbound: String,
    location: Span,
  },
  // The Span here is the Span of the let-expression that has the two duplicated bindings
  DuplicateBinding {
    duplicated_name: String,
    location: Span,
  },

  Overflow {
    num: i64,
    location: Span,
  },

  DuplicateFunName {
    duplicated_name: String,
    location: Span, // the location of the 2nd function
  },

  DuplicateArgName {
    duplicated_name: String,
    location: Span,
  },

  UndefinedType {
    undefined_type: String,
    location: Span,
  },

  WrongTypeArity {
    type_used: String,
    expected_arity: usize,
    given_arity: usize,
    location: Span,
  },

  DuplicateTypeDefs {
    duplicate_type: String,
    location: Span,
  },

  DuplicateMatchArms {
    type_used: String,
    location: Span,
  },

  DuplicateMatchArmArguments {
    type_used: String,
    location: Span,
  },

  WrongTypeCall {
    type_used: String,
    location: Span,
  },

  ShadowPrimType {
    primitive_type: String,
    location: Span,
  },
}
