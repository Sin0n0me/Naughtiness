use crate::token::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    pub node_kind: ASTNodeKind,
    pub children: Vec<ASTNode>,
}

impl ASTNode {
    pub fn new(node_kind: ASTNodeKind) -> Self {
        Self {
            node_kind,
            children: Vec::new(),
        }
    }

    pub fn write_cst(&self, file_name: &str) {
        let Ok(mut file) = File::create(file_name) else {
            return;
        };
        let Ok(data) = serde_json::to_string(self) else {
            return;
        };

        file.write_all(data.as_bytes()).unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ASTNodeKind {
    Factor { token: Token },
}
