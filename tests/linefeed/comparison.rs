use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    op_equals,
    indoc! {r#"
        print(1 == 1);
        print(-1 == 1);
    "#},
    equals("true \n false"),
    empty()
);

eval_and_assert!(
    op_not_equals,
    indoc! {r#"
        print(1 != 1);
        print(-1 != 1);
    "#},
    equals("false \n true"),
    empty()
);

eval_and_assert!(
    op_less,
    indoc! {r#"
        print(1 < 1);
        print(2 < 1);
        print(-1 < 1);
    "#},
    equals("false \n false \n true"),
    empty()
);

eval_and_assert!(
    op_less_or_equal,
    indoc! {r#"
        print(1 <= 1);
        print(2 <= 1);
        print(-1 <= 1);
    "#},
    equals("true \n false \n true"),
    empty()
);

eval_and_assert!(
    op_greater,
    indoc! {r#"
        print(1 > 1);
        print(2 > 1);
        print(-1 > 1);
    "#},
    equals("false \n true \n false"),
    empty()
);

eval_and_assert!(
    op_greater_or_equal,
    indoc! {r#"
        print(1 >= 1);
        print(2 >= 1);
        print(-1 >= 1);
    "#},
    equals("true \n true \n false"),
    empty()
);
