use serde::{Deserialize, Serialize};

macro_rules! define_keywords {
    ( $(($name:ident, $symbol:pat)),* ) => {
        #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
            pub enum Keyword {
                $($name),*
            }

            impl Keyword {
                pub fn from_str(sep: &str) -> Option<Self> {
                    match sep {
                        $($symbol => Some(Keyword::$name),)*
                        _ => None
                    }
                }

            }

    };

}

define_keywords!(
    (Const, "const"),
    (Continue, "continue"),
    (Crate, "crate"),
    (Else, "else"),
    (Enum, "enum"),
    (Extern, "extern"),
    (False, "false"),
    (Fn, "fn"),
    (For, "for"),
    (If, "if"),
    (Impl, "impl"),
    (In, "in"),
    (Let, "let"),
    (Loop, "loop"),
    (Match, "match"),
    (Mod, "mod"),
    (Move, "move"),
    (Mut, "mut"),
    (Pub, "pub"),
    (Ref, "ref"),
    (Return, "return"),
    (SelfValue, "self"),
    (SelfType, "Self"),
    (Static, "static"),
    (Struct, "struct"),
    (Super, "super"),
    (Trait, "trait"),
    (True, "true"),
    (Type, "type"),
    (Unsafe, "unsafe"),
    (Use, "use"),
    (Where, "where"),
    (While, "while"),
    (Async, "async"),
    (Await, "await"),
    (Dyn, "dyn"),
    (Abstract, "abstract"),
    (Become, "become"),
    (Box, "box"),
    (Do, "do"),
    (Final, "final"),
    (Macro, "macro"),
    (Override, "override"),
    (Priv, "priv"),
    (Typeof, "typeof"),
    (Unsized, "unsized"),
    (Virtual, "virtual"),
    (Yield, "yield"),
    (Try, "try"),
    (MacroRules, "macro_rules"),
    (Union, "union"),
    (StaticLifetime, "'static"),
    (Ur, "ur"),
    (Sr, "sr"),
    (Nr, "nr")
);
