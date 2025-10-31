use crate::helpers::{
    eval_and_assert,
    output::{empty, equals},
};
use indoc::indoc;

// Basic memoization - side effects only happen once
eval_and_assert!(
    memoized_basic_caching,
    indoc! {r#"
        call_count = 0;

        memoized fn expensive(n) {
            call_count = call_count + 1;
            n * 2
        };

        print(expensive(5));
        print(expensive(5));
        print(expensive(5));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        10
        10
        10
        Total calls: 1
    "#}),
    empty()
);

// Different arguments are cached separately
eval_and_assert!(
    memoized_different_args,
    indoc! {r#"
        memoized fn square(n) {
            print("Computing square of", n);
            n * n
        };

        print(square(2));
        print(square(3));
        print(square(2));
        print(square(3));
        print(square(4));
    "#},
    equals(indoc! {r#"
        Computing square of 2
        4
        Computing square of 3
        9
        4
        9
        Computing square of 4
        16
    "#}),
    empty()
);

// Multiple parameters - cache key includes all params
eval_and_assert!(
    memoized_multiple_params,
    indoc! {r#"
        call_count = 0;

        memoized fn add(a, b) {
            call_count = call_count + 1;
            a + b
        };

        print(add(1, 2));
        print(add(2, 1));
        print(add(1, 2));
        print(add(2, 1));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        3
        3
        3
        3
        Total calls: 2
    "#}),
    empty()
);

// Recursive memoization - dramatic performance improvement
eval_and_assert!(
    memoized_recursive_fibonacci,
    indoc! {r#"
        call_count = 0;

        memoized fn fib(n) {
            call_count = call_count + 1;
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        };

        print("fib(10) =", fib(10));
        print("Total calls:", call_count);
        print("fib(10) again =", fib(10));
        print("Total calls after second invocation:", call_count);
    "#},
    equals(indoc! {r#"
        fib(10) = 55
        Total calls: 11
        fib(10) again = 55
        Total calls after second invocation: 11
    "#}),
    empty()
);

// Zero parameters - cache the single result
eval_and_assert!(
    memoized_zero_params,
    indoc! {r#"
        call_count = 0;

        memoized fn get_constant() {
            call_count = call_count + 1;
            print("Computing constant");
            42
        };

        print(get_constant());
        print(get_constant());
        print(get_constant());
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        Computing constant
        42
        42
        42
        Total calls: 1
    "#}),
    empty()
);

// Lists as arguments - value equality
eval_and_assert!(
    memoized_with_lists,
    indoc! {r#"
        call_count = 0;

        memoized fn process_list(items) {
            call_count = call_count + 1;
            sum(items)
        };

        print(process_list([1, 2, 3]));
        print(process_list([1, 2, 3]));
        print(process_list([1, 2, 4]));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        6
        6
        7
        Total calls: 2
    "#}),
    empty()
);

// Tuples as arguments
eval_and_assert!(
    memoized_with_tuples,
    indoc! {r#"
        call_count = 0;

        memoized fn process_tuple(pair) {
            call_count = call_count + 1;
            (a, b) = pair;
            a + b
        };

        print(process_tuple((1, 2)));
        print(process_tuple((1, 2)));
        print(process_tuple((2, 1)));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        3
        3
        3
        Total calls: 2
    "#}),
    empty()
);

// Maps as arguments
eval_and_assert!(
    memoized_with_maps,
    indoc! {r#"
        call_count = 0;

        memoized fn process_map(m) {
            call_count = call_count + 1;
            m["x"] + m["y"]
        };

        print(process_map(map = {"x": 1, "y": 2}));
        print(process_map(map = {"x": 1, "y": 2}));
        print(process_map(map = {"x": 2, "y": 2}));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        3
        3
        4
        Total calls: 2
    "#}),
    empty()
);

// Sets as arguments
eval_and_assert!(
    memoized_with_sets,
    indoc! {r#"
        call_count = 0;

        memoized fn process_set(s) {
            call_count = call_count + 1;
            s.len()
        };

        print(process_set(set([1, 2, 3])));
        print(process_set(set([1, 2, 3])));
        print(process_set(set([1, 2, 3, 4])));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        3
        3
        4
        Total calls: 2
    "#}),
    empty()
);

// Memoized function inside if block (any scope)
eval_and_assert!(
    memoized_in_if_block,
    indoc! {r#"
        call_count = 0;

        if true {
            memoized fn foo(x) {
                call_count = call_count + 1;
                x * 2
            };

            print(foo(5));
            print(foo(5));
            print(foo(5));
        };

        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        10
        10
        10
        Total calls: 1
    "#}),
    empty()
);

// Memoized function inside while block
eval_and_assert!(
    memoized_in_while_block,
    indoc! {r#"
        call_count = 0;
        i = 0;

        while i < 1 {
            memoized fn double(x) {
                call_count = call_count + 1;
                x * 2
            };

            print(double(3));
            print(double(3));
            i = i + 1;
        };

        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        6
        6
        Total calls: 1
    "#}),
    empty()
);

// Memoized function inside another function
eval_and_assert!(
    memoized_nested_in_function,
    indoc! {r#"
        call_count = 0;

        fn outer() {
            memoized fn inner(x) {
                call_count = call_count + 1;
                x + 10
            };

            print(inner(5));
            print(inner(5));
            print(inner(7));
            print(inner(5));
        };

        outer();
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        15
        15
        17
        15
        Total calls: 2
    "#}),
    empty()
);

// Memoized with mixed types
eval_and_assert!(
    memoized_mixed_types,
    indoc! {r#"
        call_count = 0;

        memoized fn process(a, b, c) {
            call_count = call_count + 1;
            print("Computing:", a, b, c);
            a + b.len() + c
        };

        print(process(1, [1, 2], 3));
        print(process(1, [1, 2], 3));
        print(process(1, [1, 2, 3], 3));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        Computing: 1 [1, 2] 3
        6
        6
        Computing: 1 [1, 2, 3] 3
        7
        Total calls: 2
    "#}),
    empty()
);

// Memoized with null values
eval_and_assert!(
    memoized_with_null,
    indoc! {r#"
        call_count = 0;

        memoized fn handle_null(x) {
            call_count = call_count + 1;
            if x == null {
                "got null"
            } else {
                x
            }
        };

        print(handle_null(null));
        print(handle_null(null));
        print(handle_null(5));
        print(handle_null(null));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        got null
        got null
        5
        got null
        Total calls: 2
    "#}),
    empty()
);

// Memoized with empty collections
eval_and_assert!(
    memoized_empty_collections,
    indoc! {r#"
        call_count = 0;

        memoized fn process_empty(lst) {
            call_count = call_count + 1;
            lst.len()
        };

        print(process_empty([]));
        print(process_empty([]));
        print(process_empty([1]));
        print(process_empty([]));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        0
        0
        1
        0
        Total calls: 2
    "#}),
    empty()
);

// Multiple memoized functions interacting
eval_and_assert!(
    multiple_memoized_functions,
    indoc! {r#"
        count_a = 0;
        count_b = 0;

        memoized fn func_a(x) {
            count_a = count_a + 1;
            x * 2
        };

        memoized fn func_b(x) {
            count_b = count_b + 1;
            func_a(x) + 1
        };

        print(func_b(5));
        print(func_b(5));
        print(func_a(5));
        print(func_b(3));
        print(func_b(5));

        print("func_a calls:", count_a);
        print("func_b calls:", count_b);
    "#},
    equals(indoc! {r#"
        11
        11
        10
        7
        11
        func_a calls: 2
        func_b calls: 2
    "#}),
    empty()
);

// Memoized with boolean arguments
eval_and_assert!(
    memoized_with_booleans,
    indoc! {r#"
        call_count = 0;

        memoized fn process_bool(flag) {
            call_count = call_count + 1;
            if flag {
                "yes"
            } else {
                "no"
            }
        };

        print(process_bool(true));
        print(process_bool(false));
        print(process_bool(true));
        print(process_bool(false));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        yes
        no
        yes
        no
        Total calls: 2
    "#}),
    empty()
);

// Memoized with string arguments
eval_and_assert!(
    memoized_with_strings,
    indoc! {r#"
        call_count = 0;

        memoized fn greet(name) {
            call_count = call_count + 1;
            "Hello, " + name
        };

        print(greet("Alice"));
        print(greet("Bob"));
        print(greet("Alice"));
        print(greet("Bob"));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        Hello, Alice
        Hello, Bob
        Hello, Alice
        Hello, Bob
        Total calls: 2
    "#}),
    empty()
);

// Memoized with large fibonacci to demonstrate performance
eval_and_assert!(
    memoized_large_fibonacci,
    indoc! {r#"
        call_count = 0;

        memoized fn fib(n) {
            call_count = call_count + 1;
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        };

        result = fib(20);
        print("fib(20) =", result);
        print("Total calls:", call_count);

        result2 = fib(15);
        print("fib(15) =", result2);
        print("Total calls after fib(15):", call_count);
    "#},
    equals(indoc! {r#"
        fib(20) = 6765
        Total calls: 21
        fib(15) = 610
        Total calls after fib(15): 21
    "#}),
    empty()
);

// Memoized function with captured variables (closure behavior)
eval_and_assert!(
    memoized_with_closure_behavior,
    indoc! {r#"
        multiplier = 10;
        call_count = 0;

        memoized fn compute(x) {
            call_count = call_count + 1;
            x * multiplier
        };

        print(compute(5));
        print(compute(5));

        multiplier = 20;

        print(compute(5));
        print("Total calls:", call_count);
    "#},
    equals(indoc! {r#"
        50
        50
        50
        Total calls: 1
    "#}),
    empty()
);
