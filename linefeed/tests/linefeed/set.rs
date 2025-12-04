use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};

use indoc::indoc;

eval_and_assert!(
    set_remove_existing_element,
    indoc! {r#"
        s = set([1, 2, 3, 4, 5]);
        s.remove(3);
        print(s.len());
        print(3 in s);
    "#},
    equals(indoc! {r#"
        4
        false
    "#}),
    empty()
);

eval_and_assert!(
    set_remove_non_existent_element,
    indoc! {r#"
        s = set([1, 2, 3]);
        s.remove(10);
        print(s.len());
    "#},
    equals("3"),
    empty()
);

eval_and_assert!(
    set_remove_from_empty_set,
    indoc! {r#"
        s = set();
        s.remove(42);
        print(s.len());
    "#},
    equals("0"),
    empty()
);

eval_and_assert!(
    set_remove_string_element,
    indoc! {r#"
        s = set(["apple", "banana", "cherry"]);
        s.remove("banana");
        print(s.len());
        print("banana" in s);
        print("apple" in s);
    "#},
    equals(indoc! {r#"
        2
        false
        true
    "#}),
    empty()
);

eval_and_assert!(
    set_remove_all_elements,
    indoc! {r#"
        s = set([1, 2]);
        s.remove(1);
        s.remove(2);
        print(s.len());
    "#},
    equals("0"),
    empty()
);

eval_and_assert!(
    set_remove_and_add,
    indoc! {r#"
        s = set([1, 2, 3]);
        s.remove(2);
        s.append(4);
        print(s.len());
        print(2 in s);
        print(4 in s);
    "#},
    equals(indoc! {r#"
        3
        false
        true
    "#}),
    empty()
);

eval_and_assert!(
    set_remove_multiple_times,
    indoc! {r#"
        s = set([1, 2, 3]);
        s.remove(2);
        s.remove(2);
        s.remove(2);
        print(s.len());
        print(2 in s);
    "#},
    equals(indoc! {r#"
        2
        false
    "#}),
    empty()
);

eval_and_assert!(
    set_iterate_basic,
    indoc! {r#"
        s = set([1, 2, 3]);
        count = 0;
        for val in s {
            count = count + 1;
        };
        print(count);
    "#},
    equals("3"),
    empty()
);

eval_and_assert!(
    set_iterate_empty,
    indoc! {r#"
        s = set();
        count = 0;
        for val in s {
            count = count + 1;
        };
        print(count);
    "#},
    equals("0"),
    empty()
);

eval_and_assert!(
    set_iterate_with_break,
    indoc! {r#"
        s = set([1, 2, 3, 4, 5]);
        count = 0;
        for val in s {
            count = count + 1;
            if count == 3 {
                break;
            };
        };
        print(count);
    "#},
    equals("3"),
    empty()
);

eval_and_assert!(
    set_iterate_membership,
    indoc! {r#"
        s = set(["apple", "banana", "cherry"]);
        found_apple = false;
        found_banana = false;
        found_cherry = false;
        for item in s {
            if item == "apple" {
                found_apple = true;
            };
            if item == "banana" {
                found_banana = true;
            };
            if item == "cherry" {
                found_cherry = true;
            };
        };
        print(found_apple and found_banana and found_cherry);
    "#},
    equals("true"),
    empty()
);

eval_and_assert!(
    set_mutation_after_iteration,
    indoc! {r#"
        s = set([1, 2, 3]);
        for item in s {
            item;
        };
        s.add(4);
        print(s.len());
    "#},
    equals("4"),
    empty()
);

eval_and_assert!(
    set_difference,
    indoc! {r#"
        a = set([1, 2, 3, 4, 5]);
        b = set([3, 4, 5, 6, 7]);
        diff = a - b;
        print(diff.len());
        print(1 in diff);
        print(2 in diff);
        print(3 in diff);
    "#},
    equals(indoc! {r#"
        2
        true
        true
        false
    "#}),
    empty()
);

eval_and_assert!(
    set_difference_empty_result,
    indoc! {r#"
        a = set([1, 2, 3]);
        b = set([1, 2, 3, 4, 5]);
        diff = a - b;
        print(diff.len());
    "#},
    equals("0"),
    empty()
);

eval_and_assert!(
    set_difference_no_overlap,
    indoc! {r#"
        a = set([1, 2, 3]);
        b = set([4, 5, 6]);
        diff = a - b;
        print(diff.len());
        print(1 in diff);
        print(2 in diff);
        print(3 in diff);
    "#},
    equals(indoc! {r#"
        3
        true
        true
        true
    "#}),
    empty()
);
