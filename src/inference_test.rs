use std::collections::HashMap;
use std::ops::Deref;

/// This is the expression that needs to be inferred, so the incoming expression as in the
/// AST
#[derive(Clone, Debug)]
enum Expression {
    EInt {
        value: i32,
    },
    EVar {
        name: String,
    },
    EFunc {
        param: String,
        body: Box<Expression>,
    },
    ECall {
        func: Box<Expression>,
        arg: Box<Expression>,
    },
    EIf {
        cond: Box<Expression>,
        true_b: Box<Expression>,
        false_b: Box<Expression>,
    },
}


/// This is the returned Type for the inference, so it is the outgoing type
#[derive(Clone, Debug)]
enum Type {
    // This is a named variable like bool
    TNamed {
        name: String,
    },
    // This is a stand in for when we do not know the type yet
    TVar {
        name: String,
    },
    // This is a function type that takes a type 'from' and returns a 'to'
    TFun {
        from: Box<Type>,
        to: Box<Type>,
    },
}

#[derive(Clone, Debug)]
struct Env(HashMap<String, Box<Type>>);

impl Env {
    /// Return an intially filled environment
    fn intial() -> Env {
        let mut env = Env{0: Default::default()};
        env.0.insert("true".to_string(), Box::new(Type::TNamed{name: "Bool".to_string()}));
        env.0.insert("false".to_string(), Box::new(Type::TNamed{name: "Bool".to_string()}));
        env
    }
}

#[derive(Clone, Debug)]
struct Context {
    pub next: i32,
    // next type variable to be generated
    pub env: Env, // mapping of variable scopes to types
}

impl Context {

    fn new(env: Env) -> Context {
        Context {
            next: 0,
            env
        }
    }
}

/// A map of type variables names to types assigned to them
struct Substitution(HashMap<String, Box<Type>>);

impl Substitution {
    fn new() -> Substitution {
        Substitution {
            0: Default::default()
        }
    }
}

/// replace the type variables in a type that are
/// present in the given substitution and return the
/// type with those variables with their substituted values
/// eg. Applying the substitution {"a": Bool, "b": Int}
/// to a type (a -> b) will give type (Bool -> Int)
fn appl_subs_to_type<'a>(subst: &Substitution, type_: &Box<Type>) -> Box<Type> {
    match type_.deref() {
        // In case of a name type like 'bool' just return it's type
        Type::TNamed {name: _} => {return type_.clone()}
        // In case of a type variable return it's type if it is in the substitution
        // otherwise, just return the given type
        Type::TVar {name} => {
            subst.0.get(name).unwrap_or(type_).clone()
        }
        // For the function type arguments recursively apply for the subtypes
        Type::TFun {from, to} => {
            Box::new(Type::TFun {from: appl_subs_to_type(subst, from), to: appl_subs_to_type(subst, to)})
        }
    }
}

/// Add a binding to a contexts environment
fn add_to_context(ctx: &Context, name: &String, type_: &Box<Type>) -> Context {
    let mut new_context = ctx.clone();
    new_context.env.0.insert(name.clone(), type_.clone());
    new_context

}

/// Create a new type variable
fn new_type_var(ctx: &mut Context) -> Box<Type> {
    let idx = ctx.next;
    ctx.next += 1;
    Box::new(Type::TVar {name: format!("T{}", idx).to_string()})
}

/// This function creates the substitution for a name and a type
fn var_bind(name: &String, t: &Box<Type>) -> Substitution {
    match t.deref() {
        // Return an empty substitution because it is the same type
        Type::TVar {name: type_name} => {
            if name == type_name {
                return Substitution::new()
            }
        }
        _ => {}
    };

    // Check if the type contains a reference to itself
    if contains(t, name) {
        panic!(format!("Type {:?} contains a reference to itself", t));
    }

    // Create a new substitution that substitutes the name for the type
    let mut sub = Substitution::new();
    sub.0.insert(name.clone(), t.clone());
    sub
}

/// Check if the type contains itself, recursively
fn contains(t: &Box<Type>, name: &String) -> bool {
    match t.deref() {
        Type::TNamed { .. } => false,
        Type::TVar { name: type_name } => name == type_name,
        Type::TFun { from, to } => contains(from, name) || contains(to, name),
    }

}

fn unify(t1: &Box<Type>, t2: &Box<Type>) -> Substitution {
    match (t1.deref(), t2.deref()) {
        (Type::TNamed {name}, Type::TNamed {name: name2}) => {
            if name == name2 {
                Substitution::new()
            } else {
                panic!(format!("Unification failed, type names do not fit {} != {}", name, name2))
            }
        }
        (Type::TVar {name}, _) => {
            var_bind(name, t2)
        }
        (_, Type::TVar {name}) => {
            var_bind(name, t1)
        }
        (Type::TFun {from, to}, Type::TFun {from: from2, to: to2}) => {
            let s1 = unify(from, from2);
            let s2 = unify(&appl_subs_to_type(&s1, &to), &appl_subs_to_type(&s1, &to2));
            compose_substitution(&s1, &s2)
        }
        (_, _) => panic!(format!("Type mismatch expected: {:?}, but found: {:?}", t1, t2))
    }

}

/// Combines two subsitutios
fn compose_substitution(s1: &Substitution, s2: &Substitution) -> Substitution {
    let mut subs = Substitution::new();
    for (name, type_) in s2.0.iter() {
        subs.0.insert(name.clone(), appl_subs_to_type(s1, type_));
    };
    subs
}

/// apply given substitution to each type in the context's environment
/// Doesn't change the input context, but returns a new one
fn apply_subs_to_ctx(subs: &Substitution, ctx: &Context) -> Context {
    let mut new_ctx = Context::new(ctx.env.clone());
    new_ctx.next = ctx.next;

    for (name, type_) in ctx.env.0.iter() {
        new_ctx.env.0.insert(name.clone(), appl_subs_to_type(subs, type_));
    }

    new_ctx
}

/// For an expression and an environment infer it's type
fn infer(ctx: &mut Context, e: &Box<Expression>) -> (Box<Type>, Substitution) {
    match e.deref() {
        // An integer is just an integer
        Expression::EInt { value: _ } => (Box::new(Type::TNamed { name: "Int".to_string()}), Substitution::new()),
        // For a variable just look up it's type
        Expression::EVar { name } => {
            return (ctx.env
                .0
                .get(name)
                .expect(format!("Unbound {}", name).as_str())
                .clone(), Substitution::new())
        }
        Expression::EFunc {param, body} => {
            // Create a new type variable for the param
            let new_type = new_type_var(ctx);
            // Associate param with type variable, and extend the context,
            // this creates a new context because it is local
            let mut new_ctx = add_to_context(ctx, &param, &new_type);
            // Infer the types for the body
            let (body_type, subst) = infer(&mut new_ctx, body);
            // Substitute the inferred type
            let inferred_type = Box::new(Type::TFun {from: appl_subs_to_type(&subst, &new_type), to: body_type });
            // Return the result
            (inferred_type, subst)
        }
        Expression::ECall { func, arg } => {
            let (func_type, s1) = infer(ctx, func);
            let (arg_type, s2) = infer(&mut apply_subs_to_ctx(&s1, ctx), arg);

            let new_var = new_type_var(ctx);
            let s3 = compose_substitution(&s1, &s2);

            let func_pre_unify = Box::new(Type::TFun { from: arg_type.clone(), to: new_var });
            let s4 = unify(&func_pre_unify, &func_type);

            let func_unified = appl_subs_to_type(&s4, &func_type);
            let s5 = compose_substitution(&s4, &s3);

            if let Type::TFun { from, to } = func_unified.deref() {
                let s6 = unify(&appl_subs_to_type(&s5, from), &arg_type);
                let result_subs = compose_substitution(&s5, &s6);
                (appl_subs_to_type(&result_subs, to), result_subs)
            } else { panic!("Only expects TFun in call type") }
        }
        _ => unimplemented!(),
    }
}

fn main() {
    let env = Env::intial();
    let mut ctx = Context::new(env);
    let expression = Box::new(Expression::EFunc{param: "a".into(), body: Box::new(Expression::EVar{name: "true".into()})});

    let (type_, _subs) = infer(&mut ctx, &expression);
    println!("Found type: {:?}", type_.deref());
}
