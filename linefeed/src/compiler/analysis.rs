use std::collections::HashSet;

use crate::{
    compiler::ir_value::IrValue,
    grammar::ast::{AssignmentTarget, Expr, Span, Spanned},
};

pub fn find_all_assignments(expr: &Spanned<Expr>) -> Vec<Spanned<String>> {
    fn find_all_assignments_inner<'src>(expr: &Spanned<Expr<'src>>) -> Vec<Spanned<&'src str>> {
        fn resolve_assignment_target<'src>(
            target: &AssignmentTarget<'src>,
            span: Span,
        ) -> Vec<Spanned<&'src str>> {
            match target {
                AssignmentTarget::Local(local) => vec![Spanned(local, span)],
                AssignmentTarget::Destructure(locals) => locals
                    .iter()
                    .flat_map(|local| resolve_assignment_target(local, span))
                    .collect(),
                AssignmentTarget::Index(target, index) => {
                    let mut res = find_all_assignments_inner(target);
                    res.extend(find_all_assignments_inner(index));
                    res
                }
            }
        }

        match &expr.0 {
            Expr::Assign(target, val) => {
                let mut res = find_all_assignments_inner(val);
                res.extend(resolve_assignment_target(target, expr.span()));
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
                let mut res = resolve_assignment_target(loop_var, expr.span());
                res.extend(find_all_assignments_inner(iterable));
                res.extend(find_all_assignments_inner(body));
                res
            }

            Expr::ListComprehension(body, loop_var, iterable) => {
                let mut res = resolve_assignment_target(loop_var, expr.span());
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

            Expr::Match(expr, arms) => {
                let mut res = find_all_assignments_inner(expr);
                for (cond, body) in arms {
                    // TODO: assign arm variable here
                    res.extend(find_all_assignments_inner(cond));
                    res.extend(find_all_assignments_inner(body));
                }
                res
            }

            Expr::Map(items) => {
                let mut res = Vec::new();
                for (key, value) in items {
                    res.extend(find_all_assignments_inner(key));
                    res.extend(find_all_assignments_inner(value));
                }
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

pub fn eval_simple_constant(expr: &Spanned<Expr>) -> Result<Option<IrValue>, String> {
    let res = match &expr.0 {
        Expr::Value(ast_val) => Some(IrValue::try_from(ast_val)?),

        Expr::List(items) | Expr::Tuple(items) => {
            let mut values = Vec::new();
            for item in items {
                match eval_simple_constant(item)? {
                    Some(val) => values.push(val),
                    _ => return Ok(None),
                }
            }

            Some(match &expr.0 {
                Expr::List(_) => IrValue::List(values),
                Expr::Tuple(_) => IrValue::Tuple(values),
                _ => unreachable!(),
            })
        }

        Expr::Block(sub_expr) => eval_simple_constant(sub_expr)?,

        Expr::Sequence(exprs) => {
            if exprs.len() == 1 {
                eval_simple_constant(&exprs[0])?
            } else {
                None
            }
        }

        _ => None,
    };

    Ok(res)
}
