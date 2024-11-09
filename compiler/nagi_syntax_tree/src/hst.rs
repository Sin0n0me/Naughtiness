use crate::token::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HSTNode {
    pub node_kind: HSTNodeKind,
    pub children: Vec<HSTNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HSTNodeKind {
    Factor { token: Token },
}
