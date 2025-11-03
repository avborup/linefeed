use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    sort_integers,
    indoc! {r#"
        list = [3, 1, 4, 1, 5, 9, 2, 6];
        print(list.sort());
    "#},
    equals("[1, 1, 2, 3, 4, 5, 6, 9]"),
    empty()
);

eval_and_assert!(
    sort_strings,
    indoc! {r#"
        words = ["zebra", "apple", "mango", "banana"];
        print(words.sort());
    "#},
    equals(r#"["apple", "banana", "mango", "zebra"]"#),
    empty()
);

eval_and_assert!(
    sort_in_place_and_returns_list,
    indoc! {r#"
        original = [5, 2, 8, 1];
        reference = original;
        result = original.sort();
        print(original);
        print(reference);
        print(result);
        print(result == original);
    "#},
    equals(indoc! {r#"
        [1, 2, 5, 8]
        [1, 2, 5, 8]
        [1, 2, 5, 8]
        true
    "#}),
    empty()
);

eval_and_assert!(
    sort_by_string_length,
    indoc! {r#"
        words = ["a", "abc", "ab", "abcd"];
        sorted = words.sort(fn (item) item.len());
        print(sorted);
    "#},
    equals(r#"["a", "ab", "abc", "abcd"]"#),
    empty()
);

eval_and_assert!(
    sort_by_custom_key,
    indoc! {r#"
        pairs = [[2, "b"], [1, "c"], [3, "a"]];
        sorted = pairs.sort(fn (pair) pair[1]);
        print(sorted);
    "#},
    equals(r#"[[3, "a"], [2, "b"], [1, "c"]]"#),
    empty()
);

eval_and_assert!(
    sort_by_negative_value,
    indoc! {r#"
        nums = [1, 2, 3, 4, 5];
        sorted = nums.sort(fn (x) -x);
        print(sorted);
    "#},
    equals("[5, 4, 3, 2, 1]"),
    empty()
);
