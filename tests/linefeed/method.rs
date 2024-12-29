use crate::helpers::{
    eval_and_assert,
    output::{contains, empty},
};

use indoc::indoc;

eval_and_assert!(
    method_on_wrong_type_errors,
    indoc! {r#"
        print(42.upper());
    "#},
    empty(),
    contains("Type mismatch: Cannot call method 'upper' on type 'number'")
);
