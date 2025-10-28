use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    list_comprehensions_work,
    indoc::indoc! {r#"
        list = [1, 2, 3, 4, 5];
        print([x * 2 for x in list]);
        print([x for x in list if x % 2 == 0]);
        print([x for x in list if x > 1 and x < 5]);
        print([x * 2 in list for x in list]);
    "#},
    equals(indoc! {r#"
        [2, 4, 6, 8, 10]
        [2, 4]
        [2, 3, 4]
        [true, true, false, false, false]
    "#}),
    empty()
);

eval_and_assert!(
    list_comprehension_destructured_works,
    indoc::indoc! {r#"
        list = [(1, 2), (3, 4), (5, 6)];
        print([x + y for x, y in list]);
    "#},
    equals("[3, 7, 11]"),
    empty()
);

// This test is here to cover a (fixed) issue where the list comprehension would create temporary
// variables that would be stored relative to the the base pointer, but this would overwrite values
// currently on the stack (e.g. the 5 in the test below). If the 5 was *after* the list
// comprehension, there would be no issue.
eval_and_assert!(
    list_comprehension_doesnt_overwrite_stack_entry,
    indoc::indoc! {r#"
        a = 5 + sum([i for i in 1..=1]);
        print(a);
    "#},
    equals("6"),
    empty()
);
