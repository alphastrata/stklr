#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::{cell::Cell, collections::HashMap};

#[derive(Debug, Clone)]
struct Node<'a> {
    val: Cell<i32>,
    adjacent: Vec<&'a Node<'a>>,
}

impl Node<'_> {
    fn add_one(node: &Node) {
        let curval = node.val.get();
        node.val.set(curval + 1);
        for adj in &node.adjacent {
            Self::add_one(&adj)
        }
    }
}

fn main() {
    let a = Node {
        val: Cell::new(1),
        adjacent: vec![],
    };
    let b = Node {
        val: Cell::new(2),
        adjacent: vec![&a],
    };
    let c = Node {
        val: Cell::new(3),
        adjacent: vec![&a, &b],
    };

    dbg!(&a);
    dbg!(&b);
    dbg!(&c);
}
