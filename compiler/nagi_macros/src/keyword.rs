macro_rules! define_separator {
    ( $(($name:ident, $symbol:pat)),* ) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
            pub enum Keyword {
                $($name),*
            }

            impl Keyword {
                pub fn from_str(keyword: &str) -> Option<Self> {
                    match sep {
                        $($symbol => Some(Separator::$name),)*
                        _ => None
                    }
                }
                
                pub fn is_keyword(keyword: &str) -> Option<Self> {
                    match sep {
                        $($symbol => Some(Separator::$name),)*
                        _ => None
                    }
                }
                
            }

    };

}
