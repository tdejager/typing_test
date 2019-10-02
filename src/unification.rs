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

fn occurs_check(v: &Expression, term: &Expression, subst: &Substitution) {
    if let Var(var_name) = v {
        match term.deref() {
            Var(term_name) => true,
            Expression::App(_, _) => {},
            Expression::Const(_) => {},
        }
    }
}
fn main() {
    println!("Help!")
}