use std::collections::HashMap;
use crate::Expression::Var;
use std::ops::Deref;

#[derive(Debug, Clone)]
enum Expression {
    App(String, Vec<Expression>),
    Var(String),
    Const(i32),
}

type Substitution = HashMap<String, Expression>;

/// Does the variable v occur anywhere inside the term?
/// Variables in the term are looked up in the subst and the check
/// should be applied recursively
fn occurs_check(v: &Expression, term: &Expression, subst: &Substitution) -> bool {
    // Only check for variables
    dbg!(term);
    if let Var(var_name) = v {
        let result = match term.deref() {
            Var(term_name) => {
                // If the have the same name they are the same variables
                if term_name == var_name {
                    return true;
                }
                // If term is a var and name of the term is in the substitution
                // Check recursively
                if let Some(expr) = subst.get(term_name) {
                    occurs_check(v, expr, subst)
                } else {
                    false
                }
            },
            Expression::App(name, args) => {
                // Check for all parameters of the variable v occurs in the args
                args.iter().any(|expr| occurs_check(v, expr, subst))
            },
            Expression::Const(_) => false,
        };

        return result
    }
    false
}

fn main() {
    let expr1 = Expression::Var("X".to_string());
    let expr2 = Expression::Var("X".to_string());
    let subs = Substitution::new();

    println!("occurs_check: {}", occurs_check(&expr1, &expr2, &subs));

    let mut subs = Substitution::new();
    subs.insert("X".to_string(), Expression::Var("Y".to_string()));

    let expr3 = Expression::Var("Y".to_string());
    let expr4 = Expression::App("f".to_string(), [Expression::Var("X".to_string())].to_vec());

    println!("occurs_check: {}", occurs_check(&expr3, &expr4, &subs));
}