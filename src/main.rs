#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::{cell::RefCell, collections::HashMap};

#[derive(Debug, Clone)]
struct Node<'a> {
    val: RefCell<String>,
    adjacent: Vec<&'a Node<'a>>,
}

impl Node<'_> {
    fn add_one(node: &Node) {
        let mut curval = node.val.borrow_mut();
        curval.push('!');
        for adj in &node.adjacent {
            Self::add_one(&adj)
        }
    }
}

fn main() {
    let a = Node {
        val: RefCell::new("A".into()),
        adjacent: vec![],
    };
    let b = Node {
        val: RefCell::new("B".into()),
        adjacent: vec![&a],
    };
    let c = Node {
        val: RefCell::new("C".into()),
        adjacent: vec![&a, &b],
    };

    dbg!(&a);
    dbg!(&b);
    dbg!(&c);
}
