#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use core::fmt;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one() {}
}
