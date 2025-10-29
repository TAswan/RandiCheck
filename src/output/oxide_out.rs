use core::panic;
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::{Arc, Mutex, RwLock}};

use conjure_cp_core::{ast::{Atom, DeclarationPtr, Domain, Expression, Literal, Metadata, Moo, Name, SymbolTable}, context::Context, error::Error, rule_engine::{resolve_rule_sets, rewrite_naive}, solver::{Solver, SolverFamily, adaptors::{self, Minion}, states::ExecutionSuccess}, *};
use ustr::Ustr;
use crate::adt::{Adt, Func, Operand};



pub fn generate_oxide_output(adt: &Adt, funcs: &Vec<Func>, verbose: bool) {
    if verbose {
        println!("--- Generating Oxide Output ---");
    println!("--- ADT ---");
    println!("{:#?}", adt);
    println!("--- Functions ---");
    for f in funcs {
        println!("{:#?}", f);
    }
}

    let model = create_model(adt, funcs, Default::default(), verbose);

    if verbose {
        println!("--- Generated Model ---");
        println!("{model}", );
    }

    const DEFAULT_RULE_SETS: &[&str] = &[];
    let rule_sets = resolve_rule_sets(SolverFamily::Minion, DEFAULT_RULE_SETS).unwrap();


    let model = rewrite_naive(&model, &rule_sets, true, false).unwrap();
    println!("Rewritten model: \n {model} \n",);


    let solver = Solver::new(adaptors::Minion::new());
    let solver = solver.load_model(model).unwrap();

    let counter_ptr = Arc::new(Mutex::new(0));
    let counter_ptr_2 = counter_ptr.clone();

    let all_solutions_ptr = Arc::new(Mutex::<Vec<HashMap<Name, Literal>>>::new(vec![]));
    let all_solutions_ptr_2 = all_solutions_ptr.clone();

    let result = solver.solve(Box::new(move |sols| {
        let mut counter = (*counter_ptr_2).lock().unwrap();
        *counter += 1;

        let mut all_solutions = (*all_solutions_ptr_2).lock().unwrap();
        (*all_solutions).push(sols);
        true
    }));

    // Did the solver run successfully?
    let solver: Solver<Minion, ExecutionSuccess> = match result {
        Ok(s) => s,
        Err(e) => {
            panic!("Error! {e:?}");
        }
    };

    // Read our counter.
    let counter = (*counter_ptr).lock().unwrap();
    println!("Num solutions: {counter}\n");

    // Read solutions, print 3
    let all_sols = (*all_solutions_ptr).lock().unwrap();
    for (i, sols) in all_sols.iter().enumerate() {
        if i > 2 {
            println!("... and {} more", *counter - i);
            break;
        }
        println!("Solution {}:", i + 1);
        for (k, v) in sols.iter() {
            println!("  {k} = {v}");
        }
        println!()
    }
    println!();

    println!("--- Oxide Specification ---");
}

fn create_model(adt: &Adt, funcs: &[Func],  context: Arc<RwLock<Context<'static>>>, verbose: bool) -> Model{
    let mut model = Model::new(context);
    let scope = model.as_submodel().symbols_ptr_unchecked().clone();
    let submodel = model.as_submodel_mut();

    // Add ADT to model
    parse_adt_to_model(adt, &mut submodel.symbols_mut());

    let constraints: Vec<Expression> = funcs
        .iter()
        .map(|x| {
            convert_function(adt, x, model.as_submodel_mut().symbols_ptr_unchecked(), verbose)
        })
        .collect();
    model.as_submodel_mut().add_constraints(constraints);


    model
}

fn parse_adt_to_model(adt: &Adt, symbols: &mut SymbolTable)  {
    let cons = adt.constructors.clone();


    for con in cons {
        let con_name = con.prefix;
        for (i, t) in con.types.iter().enumerate() {
            let var_name = format!("{}_{}", con_name, i);
            
            let name = Name::User(Ustr::from(&var_name));

            let domain = match t {
                crate::adt::Type::Int => Domain::Int([range!(1..5)].to_vec()), // Example range
                crate::adt::Type::Bool => Domain::Bool,
            };

             let _ = symbols
            .insert(DeclarationPtr::new_var(name.clone(), domain))
            .ok_or(Error::Parse(format!(
                "Could not add {name} to symbol table as it already exists"
            )));
        }
    }

    // add tag variable
    let tag_name = Name::User(Ustr::from("tag"));
    let cons_len = adt.constructors.len() as i32;
    let tag_domain = Domain::Int([range!(1..cons_len)].to_vec());
    let _ = symbols
        .insert(DeclarationPtr::new_var(tag_name.clone(), tag_domain))
        .ok_or(Error::Parse(format!(
            "Could not add {tag_name} to symbol table as it already exists"
        )));

}

fn convert_function(adt: &Adt, func : &Func, symbols: &Rc<RefCell<SymbolTable>>, verbose: bool)  -> Expression {
    let prefixes = adt
        .constructors
        .iter()
        .map(|c| c.prefix.clone())
        .collect::<Vec<String>>();

    let func_con = adt
        .constructors
        .iter()
        .find(|c| c.prefix == func.con.prefix)
        .expect("Function input constructor not found in ADT");

    let not_prefixes = prefixes
        .iter()
        .filter(|p| *p != &func.con.prefix)
        .collect::<Vec<&String>>();
    //TODO, ADD THE NOTS

    // Build expression
match &func.opp {
        crate::adt::Operation::ConstSelf => {
            let name = func.con.prefix.clone();
            let prefix = Name::User(Ustr::from(&format!("{}_0", name))); // hard coding as a test TODO FIX
            
            let declaration: DeclarationPtr = symbols
                .borrow()
                .lookup(&prefix)
                .expect("Could not find declaration for function input");

            let name_expr = Expression::Atomic(
                Metadata::new(),
                Atom::Reference(declaration),
            );

            //right now const self just assumes boolean true, later this should be extended to be a general constant
            let const_self = Expression::Atomic(
                Metadata::new(),
                Atom::Literal(Literal::Bool(true)),
            );

            let eq_expr = Expression::Eq(Metadata::new(), Moo::new(name_expr), Moo::new(const_self));

            // now we have to add this inside an and with the tag constraints

            let tag_name = Name::User(Ustr::from("tag"));
            let tag_declaration: DeclarationPtr = symbols
                .borrow()
                .lookup(&tag_name)
                .expect("Could not find declaration for tag variable");

            let tag_expr = Expression::Eq(
                Metadata::new(),
                Moo::new(Expression::Atomic(
                    Metadata::new(),
                    Atom::Reference(tag_declaration),
                )),
                Moo::new(Expression::Atomic(
                    Metadata::new(),
                    Atom::Literal(Literal::Int(
                        ((prefixes
                            .iter()
                            .position(|p| p == &func.con.prefix)
                            .unwrap()
                            ) as i64).try_into().unwrap()
                    )))));
            let constraints = into_matrix_expr!([eq_expr, tag_expr].to_vec());
            

           Expression::And(Metadata::new(), Moo::new(constraints))

        },
        crate::adt::Operation::Lt(l, r) =>{
            if verbose {
                println!("Converting Lt function: {:?}", func);
            }
            if let Operand::Var(_) = l {
                let prefix = Name::User(Ustr::from(&format!("{}_0", func_con.prefix))); // hard coding as a test TODO FIX

                println!("Prefix for Lt left operand: {:?}", prefix);
                
                let declaration: DeclarationPtr = symbols
                    .borrow()
                    .lookup(&prefix)
                    .expect("Could not find declaration for function input");

                let name_expr = Expression::Atomic(
                    Metadata::new(),
                    Atom::Reference(declaration),
                );

                let const_lit = match r {
                    Operand::Lit(i) => Literal::Int(*i),
                    _ => panic!("Right operand must be a literal for Lt operation"),
                };

                let const_expr = Expression::Atomic(
                    Metadata::new(),
                    Atom::Literal(const_lit),
                );

                let lt_expr = Expression::Lt(Metadata::new(), Moo::new(name_expr), Moo::new(const_expr));

                // now we have to add this inside an and with the tag constraints

                let tag_name = Name::User(Ustr::from("tag"));
                let tag_declaration: DeclarationPtr = symbols
                    .borrow()
                    .lookup(&tag_name)
                    .expect("Could not find declaration for tag variable");

                let tag_expr = Expression::Eq(
                    Metadata::new(),
                    Moo::new(Expression::Atomic(
                        Metadata::new(),
                        Atom::Reference(tag_declaration),
                    )),
                    Moo::new(Expression::Atomic(
                        Metadata::new(),
                        Atom::Literal(Literal::Int(
                            ((prefixes
                                .iter()
                                .position(|p| p == &func.con.prefix)
                                .unwrap()
                                ) as i64).try_into().unwrap()
                        )))));
                let constraints = into_matrix_expr!([lt_expr, tag_expr].to_vec());
                

               Expression::And(Metadata::new(), Moo::new(constraints))
            } else {
                panic!("Left operand must be a variable for Lt operation");
            }
        },
        crate::adt::Operation::Gt(_, _) => unimplemented!(),
        crate::adt::Operation::Eq(_, _) => unimplemented!(),
        crate::adt::Operation::Neq(_, _) => unimplemented!(),
        crate::adt::Operation::Leq(_, _) => unimplemented!(),
        crate::adt::Operation::Geq(_, _) => unimplemented!(),
        crate::adt::Operation::Add(_, _) => unimplemented!(),
    }
    

}