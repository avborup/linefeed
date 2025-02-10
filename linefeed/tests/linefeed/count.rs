use crate::helpers::{
    eval_and_assert,
    output::{contains, empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    count_works_on_list,
    indoc! {r#"
        foo = [1, 2, 1, 3, 1, 4, 1, 5];
        print(foo.count(1));
        print(foo.count(2));
        print(foo.count(6));
    "#},
    equals("4 \n 1 \n 0"),
    empty()
);

eval_and_assert!(
    count_works_on_string,
    indoc! {r#"
        foo = "hello world";
        print(foo.count("o"));
        print(foo.count("l"));
        print(foo.count("z"));
    "#},
    equals("2 \n 3 \n 0"),
    empty()
);

eval_and_assert!(
    count_yields_error_on_invalid_type_in_string,
    indoc! {r#"
        foo = "hello world";
        print(foo.count(0));
    "#},
    empty(),
    contains("Type mismatch: Cannot count 'number' in 'str'")
);
