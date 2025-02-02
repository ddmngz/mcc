use crate::lex::Identifier;
use std::collections::HashSet;
use std::rc::Rc;

use crate::parse::Program;

use crate::parse::Label;

use crate::parse::Expression;

use crate::parse::Declaration;

use crate::parse::Statement;

use crate::parse::BlockItem;

use crate::parse::Function;

pub fn check(program: &mut Program, vars: &HashSet<Rc<Identifier>>) -> Result<(), Error> {
    check_function(&mut program.0, vars)
}

fn check_function(
    Function { name: _, body }: &mut Function,
    vars: &HashSet<Rc<Identifier>>,
) -> Result<(), Error> {
    check_body(body, vars)?;
    Ok(())
}

fn check_body(block: &[BlockItem], vars: &HashSet<Rc<Identifier>>) -> Result<(), Error> {
    let mut labels = HashSet::new();
    for item in block.iter() {
        if let BlockItem::S(statement) = item {
            check_labels(statement, vars, &mut labels)?;
        }
    }

    for item in block {
        if let BlockItem::S(statement) = item {
            check_gotos(statement, &labels)?;
        }
    }

    Ok(())
}

fn check_labels(
    statement: &Statement,
    vars: &HashSet<Rc<Identifier>>,
    labels: &mut HashSet<Rc<Identifier>>,
) -> Result<(), Error> {
    match statement {
        Statement::Label(Label::C23(label)) => {
            if vars.contains(label) {
                Err(Error::ClashedLabel)
            } else if labels.insert(label.clone()) {
                Ok(())
            } else {
                Err(Error::RedefinedLabel)
            }
        }
        Statement::Label(Label::C17 { label, body }) => {
            if vars.contains(label) {
                Err(Error::ClashedLabel)
            } else if labels.insert(label.clone()) {
                check_labels(body, vars, labels)
            } else {
                Err(Error::RedefinedLabel)
            }
        }
        Statement::If {
            condition: _,
            then,
            r#else,
        } => {
            check_labels(then, vars, labels)?;
            if let Some(r#else) = r#else {
                check_labels(r#else, vars, labels)?;
            };
            Ok(())
        }
        Statement::Ret(_) | Statement::Exp(_) | Statement::Null | Statement::Goto(_) => Ok(()),
    }
}

fn check_gotos(statement: &Statement, labels: &HashSet<Rc<Identifier>>) -> Result<(), Error> {
    match statement {
        Statement::Goto(goto) => {
            if labels.contains(goto) {
                Ok(())
            } else {
                Err(Error::UndefinedLabel)
            }
        }
        Statement::If {
            condition: _,
            then,
            r#else,
        } => {
            check_gotos(then, labels)?;
            if let Some(r#else) = r#else {
                check_gotos(r#else, labels)?;
            };
            Ok(())
        }
        Statement::Label(Label::C17 { body, .. }) => check_gotos(body, labels),
        Statement::Ret(_)
        | Statement::Exp(_)
        | Statement::Null
        | Statement::Label(Label::C23(_)) => Ok(()),
    }
}

#[derive(Debug)]
pub enum Error {
    RedefinedLabel,
    ClashedLabel,
    UndefinedLabel,
}
