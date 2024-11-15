#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub error_kind: ErrorKind,
    pub error_text: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Lexcal(LexicalError),
    Syntax(SyntaxError),
    Semantic(SemanticError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexicalError {
    IllegalCharacter,      // 不正な文字
    InvalidIdentifierName, // 無効な識別子
    InvalidEscapeSequence, // 無効なエスケープシーケンス
    InvalidNumberFormat,   // 無効な数値フォーマット
    InvalidCommentFormat,  // 無効なコメントフォーマット
}

#[derive(Debug, PartialEq, Eq)]
pub enum SyntaxError {
    Recursed,             // 再帰した
    NotMatch,             // マッチしなかった
    ExpectedToken,        // 期待したトークンがない
    MissingSemicolon,     // セミコロン忘れ
    ParenthesesNotClosed, // 括弧が閉じられていない
}

#[derive(Debug, PartialEq, Eq)]
pub enum SemanticError {
    TODO,                  //  TODO
    UndefinedVariable,     // 未定義の変数
    UndefinedFunction,     // 未定義の関数
    RedeclarationVariable, // 変数の再宣言
    RedefinitionFunction,  // 関数の再定義
    TypeMissmatch,         // 型の不一致
    DivisionByZero,        // 0除算をしようとした
    TooFewArguments,       // 引数が少ない
    TooManyArguments,      // 引数が多い
}
