use backtrace::BacktraceSymbol;

pub struct HowlerEntry {
    // Prefix
    pub(crate) line: Option<u32>,
    pub(crate) character: Option<u32>,
    pub(crate) module: Option<String>,
    // Information
    pub(crate) value: Option<String>,
}

impl HowlerEntry {
    pub fn new(symbol: &BacktraceSymbol) -> HowlerEntry {
        let (module, value) =
           Self:: process_item(symbol.name().map(|symbol| format!("{symbol}")));
        HowlerEntry {
            line: symbol.lineno(),
            character: symbol.colno(),
            value,
            module
        }
    }

    fn process_item(name: Option<String>) -> (Option<String>, Option<String>) {
        match name {
            Some(name) => {
                if name.starts_with('<') {
                    let mut cast = String::new();
                    let mut right = String::new();

                    // extract casting
                    let mut v = 0;
                    let mut done = false;
                    let chars = name.chars();
                    for ch in chars.clone() {
                        if done {
                            right.push(ch);
                        } else {
                            if ch == '<' {
                                if v == 0 {
                                    // skip first <
                                    v += 1;
                                    continue;
                                }
                                v += 1;
                            } else if ch == '>' {
                                v -= 1;
                                if v == 0 {
                                    done = true;
                                    // skip last > and remaining characters
                                    continue;
                                }
                            }
                            cast.push(ch);
                        }
                    }

                    // split on as
                    if let Some((original, object_as)) = cast.split_once(" as ") {
                        if let Some((module, object)) = original.split_once("::") {
                            let object_as: &str = object_as.rsplit("::").next().unwrap();
                            (
                                Some(module.to_string()),
                                Some(format!("{object} -> {object_as}{right}")),
                            )
                        } else {
                            (None, Some(name.clone()))
                        }
                    } else {
                        panic!("where tf is as")
                    }
                } else if let Some((left, right)) = name.split_once("::") {
                    (
                        Some(left.to_string()),
                        Some(right.to_string()),
                    )
                } else {
                    (None, Some(name.clone()))
                }
            }
            None => (None, None),
        }
    }
}
