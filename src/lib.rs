#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod cmd;
mod graph;
mod search;
mod utils;

use anyhow::Result;
use glob::glob;
use std::collections::HashMap;

use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};
