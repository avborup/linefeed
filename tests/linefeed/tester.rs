use indoc::indoc;

use crate::helpers::eval_and_assert;

eval_and_assert!(factorial, include_str!("../factorial.lf"), "3628800", "");

eval_and_assert!(
    function_oneliners,
    include_str!("../functions.lf"),
    indoc! {r#"
        2
        2
        yes
    "#},
    ""
);

eval_and_assert!(addition, "print(1 + 2)", "3", "");
eval_and_assert!(subtraction, "print(1 - 2)", "-1", "");
eval_and_assert!(negation, "print(-1 + 3)", "2", "");
eval_and_assert!(multiplication, "print(3 * (1 + 3))", "12", "");
eval_and_assert!(division, "print((2 * 10) / 5)", "4", "");

eval_and_assert!(equals, "print(1 == 1);print(-1 == 1)", "true \n false", "");
eval_and_assert!(
    not_equals,
    "print(1 != 1);print(-1 != 1)",
    "false \n true",
    ""
);
eval_and_assert!(not, "print(!true);print(!false)", "false \n true", "");
