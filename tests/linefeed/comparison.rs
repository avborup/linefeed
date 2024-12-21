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

eval_and_assert!(
    cmp_strings_eq,
    indoc! {r#"
        print("a" == "a");
        print("a" == "b");
        print("a" != "a");
        print("a" != "b");
    "#},
    equals("true \n false \n false \n true"),
    empty()
);

eval_and_assert!(
    cmp_strings_less,
    indoc! {r#"
        print("a" < "a");
        print("a" < "b");
        print("a" < "aa");
        print("a" < "ab");
    "#},
    equals("false \n true \n true \n true"),
    empty()
);

eval_and_assert!(
    cmp_list_less,
    indoc! {r#"
        print([1, 2, 3] < [1, 2, 3]);
        print([1, 2, 3] < [1, 2, 4]);
        print([1, 2, 3] < [1, 2, 2]);
        print([1, 2, 3] < [1, 3, 3]);
        print([1, 2, 3] < [2, 2, 3]);
    "#},
    equals("false \n true \n false \n true \n true"),
    empty()
);

eval_and_assert!(
    cmp_functions,
    indoc! {r#"
        print((|x| x) < (|x| x));
        print((|x| x) == (|x| x));
    "#},
    equals("false \n false"),
    empty()
);
