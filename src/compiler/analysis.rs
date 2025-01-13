use std::collections::HashSet;

use crate::grammar::ast::{Expr, Spanned};

pub fn find_all_assignments(expr: &Spanned<Expr>) -> Vec<Spanned<String>> {
    fn find_all_assignments_inner<'src>(expr: &Spanned<Expr<'src>>) -> Vec<Spanned<&'src str>> {
        match &expr.0 {
            Expr::Let(local, val) => {
                let mut res = find_all_assignments_inner(val);
                res.push(Spanned(local, expr.span()));
                res
            }

            Expr::Destructure(locals, val) => {
                let mut res = find_all_assignments_inner(val);
                res.extend(locals.iter().map(|local| Spanned(*local, expr.span())));
                res
            }

            Expr::Break | Expr::Continue | Expr::Value(_) | Expr::ParseError | Expr::Local(_) => {
                vec![]
            }

            Expr::List(items) | Expr::Tuple(items) => {
                items.iter().flat_map(find_all_assignments_inner).collect()
            }

            Expr::Index(value, index) => {
                let mut res = find_all_assignments_inner(value);
                res.extend(find_all_assignments_inner(index));
                res
            }

            Expr::If(cond, a, b) => {
                let mut res = find_all_assignments_inner(cond);
                res.extend(find_all_assignments_inner(a));
                res.extend(find_all_assignments_inner(b));
                res
            }

            Expr::While(cond, body) => {
                let mut res = find_all_assignments_inner(cond);
                res.extend(find_all_assignments_inner(body));
                res
            }

            Expr::For(loop_var, iterable, body) => {
                let mut res = vec![Spanned(*loop_var, expr.span())];
                res.extend(find_all_assignments_inner(iterable));
                res.extend(find_all_assignments_inner(body));
                res
            }

            Expr::ListComprehension(body, loop_var, iterable) => {
                let mut res = vec![Spanned(*loop_var, expr.span())];
                res.extend(find_all_assignments_inner(iterable));
                res.extend(find_all_assignments_inner(body));
                res
            }

            Expr::Call(func, args) => {
                let mut res = find_all_assignments_inner(func);
                res.extend(args.iter().flat_map(find_all_assignments_inner));
                res
            }

            Expr::MethodCall(target, _, args) => {
                let mut res = find_all_assignments_inner(target);
                res.extend(args.iter().flat_map(find_all_assignments_inner));
                res
            }

            Expr::Unary(_, sub_expr) => find_all_assignments_inner(sub_expr),

            Expr::Binary(lhs, _, rhs) => {
                let mut res = find_all_assignments_inner(lhs);
                res.extend(find_all_assignments_inner(rhs));
                res
            }

            Expr::Sequence(exprs) => exprs.iter().flat_map(find_all_assignments_inner).collect(),

            Expr::Block(sub_expr) => find_all_assignments_inner(sub_expr),

            Expr::Return(val) => find_all_assignments_inner(val),
        }
    }

    let mut seen = HashSet::new();
    find_all_assignments_inner(expr)
        .into_iter()
        .filter(|Spanned(name, _)| seen.insert(*name))
        .map(|Spanned(name, span)| Spanned(name.to_string(), span))
        .collect()
}
