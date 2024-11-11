#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ParseMemoKey {
    pub position: usize,
    pub rule: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParseMemoValue<T> {
    pub node: T,
    pub next_position: usize,
}

pub enum MemoResult<T> {
    None,
    Recursive,
    Some(T),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    E,
    Recursive,
    NotExpected, // 期待したトークンではなかった
}
