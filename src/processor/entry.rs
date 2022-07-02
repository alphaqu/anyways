use crate::audit::AuditSectionEntry;
use crate::processor::{Errors, UNKNOWN};
use backtrace::BacktraceSymbol;
use owo_colors::{OwoColorize};

pub struct ProcessingEntry {
    pub line: Option<u32>,
    pub character: Option<u32>,
    pub value: ProcessingValue,
    pub errors: Option<String>,

    pub collapsable: bool,
}

impl ProcessingEntry {
    pub fn get_location(&self) -> String {
        format!(
            "{}:{}",
            self.line.map(|v| v.to_string()).unwrap_or_else(UNKNOWN),
            self.character
                .map(|v| v.to_string())
                .unwrap_or_else(UNKNOWN)
        )
        .blue()
        .to_string()
    }

    pub fn build(self) -> AuditSectionEntry {
        let value: String = match &self.value {
            ProcessingValue::Entry { value, .. } => value.to_string(),
            ProcessingValue::Cast { from, value, .. } => {
                format!("{from} {} {value}", "->".white())
            }
            ProcessingValue::Unknown => "???".to_string(),
        };

        AuditSectionEntry {
            prefix_left: Some(self.get_location()),
            prefix: self.errors,
            separator: '|',
            prefix_right: self.value.get_module().map(|v| v.purple().to_string()),
            text: value,
            suffix: None,
        }
    }
}

#[derive(Debug)]
pub enum ProcessingValue {
    Entry {
        module: Option<String>,
        value: String,
    },
    Cast {
        from_module: Option<String>,
        from: String,

        module: Option<String>,
        value: String,
    },
    Unknown,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug)]
pub enum ProcessingValueMatcher {
    Value(String),
    Module(String),
    Item(String),
    Path(String),
}

impl ProcessingValue {
    pub fn get_value(&self) -> Option<&str> {
        match self {
            ProcessingValue::Cast { value, .. } | ProcessingValue::Entry { value, .. } => {
                Some(value)
            }
            ProcessingValue::Unknown => None,
        }
    }

    pub fn get_module(&self) -> Option<&str> {
        match self {
            ProcessingValue::Entry {
                module: Some(value),
                ..
            } => Some(value),
            ProcessingValue::Cast {
                module: Some(v1), ..
            } => Some(v1),
            _ => None,
        }
    }

    pub fn matches(&self, matcher: &ProcessingValueMatcher) -> bool {
        let value = match matcher {
            ProcessingValueMatcher::Value(target) => match self {
                ProcessingValue::Cast { value, .. } | ProcessingValue::Entry { value, .. } => {
                    value == target
                }
                ProcessingValue::Unknown => false,
            },
            ProcessingValueMatcher::Module(target) => match self {
                ProcessingValue::Entry { module, .. } => {
                    if let Some(module) = module {
                        module == target
                    } else {
                        false
                    }
                }
                ProcessingValue::Cast {
                    module: to_module, ..
                } => {
                    if let Some(module) = to_module {
                        module == target
                    } else {
                        false
                    }
                }
                ProcessingValue::Unknown => false,
            },
            ProcessingValueMatcher::Item(target) => match self {
                ProcessingValue::Cast { value, .. } | ProcessingValue::Entry { value, .. } => {
                    value.starts_with(target)
                }
                ProcessingValue::Unknown => false,
            },
            ProcessingValueMatcher::Path(target) => match self {
                ProcessingValue::Cast { value, .. } | ProcessingValue::Entry { value, .. } => {
                    value.split("::").any(|v| v == target)
                }
                ProcessingValue::Unknown => false,
            },
        };
        value
    }

    pub fn replace(&mut self, matcher: &ProcessingValueMatcher, to: &str) {
        match matcher {
            ProcessingValueMatcher::Value(from) => match self {
                ProcessingValue::Cast { value, .. } | ProcessingValue::Entry { value, .. } => {
                    *value = value.replace(from, to);
                }
                ProcessingValue::Unknown => {}
            },
            ProcessingValueMatcher::Module(from) => match self {
                ProcessingValue::Entry { module, .. } => {
                    if let Some(module) = module {
                        *module = module.replace(from, to);
                    }
                }
                ProcessingValue::Cast {
                    from_module,
                    module,
                    ..
                } => {
                    if let Some(module) = from_module {
                        *module = module.replace(from, to);
                    }

                    if let Some(module) = module {
                        *module = module.replace(from, to);
                    }
                }
                ProcessingValue::Unknown => {}
            },
            ProcessingValueMatcher::Item(pat) => match self {
                ProcessingValue::Cast { from, value, .. } => {
                    if value.starts_with(pat) {
                        *value = value.replacen(pat, to, 1);
                    }

                    if from.starts_with(pat) {
                        *from = from.replacen(pat, to, 1);
                    }
                }
                ProcessingValue::Entry { value, .. } => {
                    if value.starts_with(pat) {
                        *value = value.replacen(pat, to, 1);
                    }
                }
                ProcessingValue::Unknown => {}
            },
            ProcessingValueMatcher::Path(from) => match self {
                ProcessingValue::Cast { value, .. } | ProcessingValue::Entry { value, .. } => {
                    let strings: Vec<String> =
                        value.split("::").map(|v| v.replace(from, to)).collect();

                    *value = strings.join("::");
                }
                ProcessingValue::Unknown => {}
            },
        }
    }
}

impl ProcessingEntry {
    pub fn new(symbol: &BacktraceSymbol, errors: &Errors) -> ProcessingEntry {
        let value = symbol
            .name()
            .map(|v| Self::acquire_value(Self::strip_hash(&v.to_string())))
            .unwrap_or(ProcessingValue::Unknown);

        let errors = errors.get(&symbol.into()).map(|err| {
            let errors: Vec<String> = err
                .iter()
                .map(|id| format!("E{id}").red().to_string())
                .collect();

            errors.join(" ")
        });

        ProcessingEntry {
            line: symbol.lineno(),
            character: symbol.colno(),
            value,
            errors,
            collapsable: false,
        }
    }

    fn strip_hash(value: &str) -> &str {
        if let Some((out, _)) = value.rsplit_once("::") {
            return out;
        }

        value
    }

    fn acquire_value(value: &str) -> ProcessingValue {
        // this is for the as expression
        if value.starts_with('<') {
            // cringe parsing
            let mut statement = String::new();
            let mut rest = String::new();

            let mut depth = 0;
            let mut outside = false;
            for ch in value.chars() {
                if !outside {
                    if ch == '<' {
                        depth += 1;
                        // skip first <
                        if depth == 1 {
                            continue;
                        }
                    } else if ch == '>' {
                        depth -= 1;
                        if depth == 0 {
                            outside = true;
                            // skip last >
                            continue;
                        }
                    }

                    statement.push(ch);
                } else {
                    rest.push(ch);
                }
            }

            if let Some((mut from_item, mut to_item)) = statement.split_once(" as ") {
                let from_module = if let Some((module_out, out)) = from_item.split_once("::") {
                    // dont include module
                    from_item = out;
                    Some(module_out.to_string())
                } else {
                    None
                };

                let to_module = if let Some((module_out, out)) = to_item.split_once("::") {
                    // dont include module
                    to_item = out;
                    Some(module_out.to_string())
                } else {
                    None
                };

                return ProcessingValue::Cast {
                    from_module,
                    from: from_item.to_string(),
                    module: to_module,
                    value: format!("{to_item}{rest}"),
                };
            }
        } else {
            // If not as then the first :: will be the module
            let mut module = None;
            let mut value = value.to_string();
            if let Some((module_out, out)) = value.split_once("::") {
                module = Some(module_out.to_string());
                value = out.to_string();
            }

            return ProcessingValue::Entry { module, value };
        }

        ProcessingValue::Unknown
    }
}
