use std::{cell::RefCell, rc::Rc};

use crate::turtle::Turtle;

pub enum Agent {
    Observer(/* TODO */),
    Turtle(Rc<RefCell<Turtle>>),
    Patch(/* TODO */),
    Link(/* TODO */),
}