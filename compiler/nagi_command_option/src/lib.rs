use std::path::{Path, PathBuf};

const NAGI_EXTENSION: &str = "nag";
const NAGI_AST_EXTENSION: &str = "ast";

#[derive(Debug)]
pub struct CompileCommandOption {
    pub is_debug: bool,
    pub is_compiler_debug: bool,
    pub is_output_ast: bool,
    pub target_list: Vec<String>,
}

impl CompileCommandOption {
    pub fn new(args: &Vec<String>) -> Result<Self, String> {
        let mut is_debug = false;
        let mut is_compiler_debug = false;
        let mut is_output_ast = false;
        let mut target_list = vec![];

        if args.first().is_none() {
            return Ok(Self {
                is_debug,
                is_compiler_debug,
                is_output_ast,
                target_list: get_file(&PathBuf::from("./"), true).unwrap(),
            });
        };

        let mut iter = args.iter();
        while let Some(option) = iter.next() {
            match option.as_str() {
                "--path" => {
                    let path_list = extract_path(&iter.as_slice().to_vec());
                    for _ in 0..path_list.len() {
                        iter.next(); // advance_byみたいなことやりたい
                    }

                    for path in path_list {
                        target_list.append(&mut get_file(&path, false)?);
                    }
                }
                "--path-recursive" => {
                    let path_list = extract_path(&iter.as_slice().to_vec());
                    for _ in 0..path_list.len() {
                        iter.next();
                    }
                    for path in path_list {
                        target_list.append(&mut get_file(&path, true)?);
                    }
                }
                "--debug" => {
                    is_debug = true;
                }
                "--debug-compiler" => {
                    is_compiler_debug = true;
                }
                "--ast" => {
                    is_output_ast = true;
                }

                _ => {
                    let text = format!("unknown option '{}'", option);
                    println!("{}", text);
                    return Err(text);
                }
            };
        }

        Ok(Self {
            is_debug,
            is_compiler_debug,
            is_output_ast,
            target_list,
        })
    }
}

fn extract_path(args: &Vec<String>) -> Vec<PathBuf> {
    let mut paths = vec![];
    for arg in args.iter() {
        let Ok(path) = Path::new(arg).canonicalize() else {
            break;
        };
        if !path.exists() {
            break;
        }

        paths.push(PathBuf::from(arg.as_str()));
    }
    paths
}

fn get_file(target: &PathBuf, recursive: bool) -> Result<Vec<String>, String> {
    println!("{:?}", target.as_os_str());
    let mut target_list = Vec::<String>::new();
    if let Some(extension) = target.extension() {
        let Ok(target_str) = target.clone().into_os_string().into_string() else {
            return Err(format!(""));
        };

        if let Some(ext) = extension.to_str() {
            if ext == NAGI_EXTENSION {
                target_list.push(target_str);
            }
        }
    } else {
        let Ok(files) = target.read_dir() else {
            return Err(format!(""));
        };

        for file in files {
            let Ok(dir_entry) = file else {
                return Err(format!(""));
            };
            let path = dir_entry.path();

            if recursive {
                // ディレクトリが存在すれば再帰的に取得
                target_list.append(&mut get_file(&path, recursive)?);
            } else if !path.is_dir() {
                target_list.append(&mut get_file(&path, recursive)?);
            }
        }
    }

    Ok(target_list)
}
