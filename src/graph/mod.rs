use core::fmt;
pub mod code;
use std::{fmt::Display, ops::Deref, sync::Arc};

use parking_lot::RwLock;

#[derive(Clone, Debug)]
struct Calls {
    inner: Vec<Arc<RwLock<Node>>>,
}

impl Deref for Calls {
    type Target = Vec<Arc<RwLock<Node>>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
struct Node {
    val: String,
    parents: Option<Calls>,
    children: Option<Calls>,
}

struct Tree {
    root: Arc<RwLock<Node>>,
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p_len = if let Some(parents) = &self.parents {
            parents.len()
        } else {
            0
        };
        let c_len = if let Some(children) = &self.children {
            children.len()
        } else {
            0
        };
        write!(
            f,
            "Node: {}\n\tParents:{}\n\tChildren:{}",
            self.val, p_len, c_len
        )
    }
}

impl Node {
    fn new(val: String) -> Self {
        Self {
            val,
            parents: None,
            children: None,
        }
    }

    fn add_parent(&mut self, parent: Arc<RwLock<Node>>) {
        if self.parents.is_none() {
            self.parents = Some(Calls {
                inner: vec![parent],
            });
        } else {
            let parents = self.parents.as_mut().unwrap();
            parents.inner.push(parent);
        }
    }

    fn add_child(&mut self, child: Arc<RwLock<Node>>) {
        if self.children.is_none() {
            self.children = Some(Calls { inner: vec![child] });
        } else {
            let children = self.children.as_mut().unwrap();
            children.inner.push(child);
        }
    }
}

impl Tree {
    fn new() -> Self {
        let root = Arc::new(RwLock::new(Node::new("root".to_string())));
        Self { root }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::prelude::*;
    use rand::{thread_rng, Rng};
    use std::collections::HashMap;
    use std::thread::JoinHandle;

    fn rand_str() -> String {
        let mut rng = thread_rng();
        let chars: String = (0..7).map(|_| rng.sample(Alphanumeric) as char).collect();
        chars
    }

    #[test]
    fn test_concurrent_hash_map() {
        let map = RwLock::new(HashMap::new());
        let mut defnitely_in_there = vec![];

        // add key-value pairs to the map
        (0..=1_000_000).into_iter().for_each(|n| {
            let quick_random_string = rand_str();
            if n % 3333 == 0 {
                defnitely_in_there.push(quick_random_string.clone());
            }
            map.write().insert(n, quick_random_string);
        });

        let haystack = RwLock::new(&defnitely_in_there);
        let handles = (0..=31)
            .into_iter()
            .map(|n| {
                let m = map.read().clone();
                let haystack = haystack.read().clone();
                let handle = std::thread::spawn(move || {
                    for (key, value) in m.iter() {
                        if haystack.iter().any(|s| s == value) {
                            println!("Worker thread{n} =  k:{}, v:{}", key, value);
                        }
                    }
                });
                handle
            })
            .collect::<Vec<JoinHandle<()>>>();

        for h in handles {
            h.join();
        }
    }
}
