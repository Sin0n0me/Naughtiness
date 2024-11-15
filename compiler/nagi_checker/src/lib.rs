use nagi_errors::*;
use nagi_syntax_tree::ast::ASTNode;
use nagi_syntax_tree::cst::CSTNode;
use nagi_syntax_tree::token::*;
use semantic_analyzer::SemanticAnalyzer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

mod semantic_analyzer;
mod type_checker;

pub fn check(cst: &CSTNode) -> Result<ASTNode, Error> {
    let mut analyzer = SemanticAnalyzer::new();

    analyzer.semantic_analyze(cst)
}

#[derive(Debug, Clone)]
pub struct SymbolTreeNode {
    parent: Option<RefCell<Weak<SymbolTreeNode>>>,
    children: RefCell<Vec<Rc<SymbolTreeNode>>>,
    symbol_table: HashMap<SymbolKey, SymbolRecord>,
}

impl SymbolTreeNode {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: RefCell::new(vec![]),
            symbol_table: HashMap::new(),
        }
    }

    pub fn add_child(&mut self) -> Self {
        let child = Rc::new(Self {
            parent: Some(RefCell::new(Rc::downgrade(&Rc::new(self.clone())))),
            children: RefCell::new(vec![]),
            symbol_table: HashMap::new(),
        });
        self.children.borrow_mut().push(Rc::clone(&child));

        child.as_ref().clone()
    }

    pub fn insert_function(&mut self, symbol_name: &str, return_type: Option<SymbolType>) -> bool {
        self.symbol_table
            .insert(
                SymbolKey {
                    symbol_pattern: SymbolPattern::Function,
                    symbol_name: symbol_name.to_string(),
                },
                SymbolRecord::Function(FunctionSymbolRecord { return_type }),
            )
            .is_none()
    }

    pub fn insert_variable(
        &mut self,
        symbol_name: &str,
        rarity: Rarity,
        symbol_type: SymbolType,
        size: u32,
    ) {
        self.symbol_table.insert(
            SymbolKey {
                symbol_pattern: SymbolPattern::Variable,
                symbol_name: symbol_name.to_string(),
            },
            SymbolRecord::Variable(VariableSymbolRecord {
                rarity,
                symbol_type,
                size,
            }),
        );
    }

    // ルートノードまで特定のシンボルが存在するか探す
    pub fn is_symbol_in_ancestors(&self, pattern: &SymbolPattern, symbol_name: &str) -> bool {
        if self.has_symbol(pattern, symbol_name) {
            return true;
        }

        if self.parent.is_none() {
            return false;
        }

        let mut currnt_node = self.parent.as_ref().unwrap().borrow().upgrade();
        while let Some(node) = currnt_node {
            if node.has_symbol(pattern, symbol_name) {
                return true;
            }
            currnt_node = node.parent.as_ref().unwrap().borrow().upgrade();
        }

        false
    }

    // 自身のノードにシンボルが存在するか
    fn has_symbol(&self, pattern: &SymbolPattern, symbol_name: &str) -> bool {
        self.symbol_table.contains_key(&SymbolKey {
            symbol_pattern: pattern.clone(),
            symbol_name: symbol_name.to_string(),
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SymbolKey {
    symbol_pattern: SymbolPattern,
    symbol_name: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SymbolPattern {
    Variable,
    Function,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SymbolRecord {
    Variable(VariableSymbolRecord),
    Function(FunctionSymbolRecord),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariableSymbolRecord {
    rarity: Rarity,
    symbol_type: SymbolType,
    size: u32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FunctionSymbolRecord {
    return_type: Option<SymbolType>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SymbolType {
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    Uint64,
    UInt128,
    Float32,
    Float64,

    // 後で実装
    Vec2,
    Vec3,
    Vec4,
}
