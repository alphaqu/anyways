use crate::audit::{Audit, AuditSection, AuditSectionEntry};
use crate::formatter::entry::ReporterEntry;
use crate::formatter::file::ReporterFile;
use backtrace::{resolve_frame, BacktraceSymbol, Symbol};
use owo_colors::{AnsiColors, DynColors, OwoColorize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::mem::swap;
use std::path::PathBuf;

const UNKNOWN: fn() -> String = || "???".to_string();

pub trait AuditProcessor: Sync {
    /// This processes the audit which later gets formatted by an AuditReporter
    fn process(&self, audit: &Audit) -> Vec<AuditSection>;
}

pub struct AnywaysAuditProcessor {
    pub filtered: HashSet<String>,
    pub filtered_crates: HashSet<String>,
    pub collapsable: HashSet<String>,
}

impl Default for AnywaysAuditProcessor {
    fn default() -> Self {
        AnywaysAuditProcessor {
            filtered: HashSet::from([
                "panic::unwind_safe::AssertUnwindSafe<F> -> |fn|.call()".to_string(),
                "panicking::try::do_call".to_string(),
                "panicking::try".to_string(),
                "panic::catch_unwind".to_string(),
            ]),
            filtered_crates: HashSet::from([
              //  "anyways".to_string()
            ]),
            collapsable: HashSet::from(["|fn|()".to_string(), "try?".to_string()]),
        }
    }
}

impl AuditProcessor for AnywaysAuditProcessor {
    fn process(&self, audit: &Audit) -> Vec<AuditSection> {
        let mut sections = audit.custom_sections.clone();

        let (section, errors) = self.create_error_section(audit);
        sections.push(section);
        sections.push(self.create_backtrace_section(audit, &errors));
        sections
    }
}

impl AnywaysAuditProcessor {
    pub fn create_error_section(&self, audit: &Audit) -> (AuditSection, Errors) {
        let mut errors = HashMap::new();
        let mut entries = Vec::new();
        for (i, err) in audit.errors.iter().enumerate() {
            if let Some(value) = &err.location {
                resolve_frame(value, |symbol| {
                    let location: ErrorLocationKey = symbol.into();
                    if !errors.contains_key(&location) {
                        errors.insert(location.clone(), Vec::new());
                    }
                    errors.get_mut(&location).unwrap().push(i);
                });
            }
            entries.push(AuditSectionEntry {
                prefix_left: Some(
                    format!(
                        "E{i} {}",
                        if i != audit.errors.len() - 1 {
                            "↓"
                        } else {
                            "→"
                        }
                        .white()
                    )
                    .red()
                    .to_string(),
                ),
                prefix_right: None,
                text: format!("{}", err.error),
                suffix: None,
            });
        }

        (
            AuditSection {
                name: "Errors".to_string(),
                color: DynColors::Ansi(AnsiColors::Red),
                entries,
            },
            errors,
        )
    }

    pub fn create_backtrace_section(&self, audit: &Audit, errors: &Errors) -> AuditSection {
        let mut files = self.read_backtrace(audit, errors);
        self.remove_audit_creation(&mut files);
        self.filter(&mut files);
        self.cleanup(&mut files);
        self.colorize(&mut files);

        let mut entries = Vec::new();
        for file in files {
            entries.push(AuditSectionEntry {
                prefix_left: None,
                prefix_right: None,
                text: file
                    .path
                    .map(|p| p.to_str().unwrap().white().bold().to_string())
                    .unwrap_or_else(UNKNOWN),
                suffix: None,
            });

            for entry in file.entries {
                entries.push(AuditSectionEntry {
                    prefix_left: Some(
                        format!(
                            "{}:{}",
                            entry.line.map(|v| v.to_string()).unwrap_or_else(UNKNOWN),
                            entry
                                .character
                                .map(|v| v.to_string())
                                .unwrap_or_else(UNKNOWN)
                        )
                        .blue()
                        .to_string(),
                    ),
                    prefix_right: entry.module.map(|v| v.purple().to_string()),
                    text: entry.value.unwrap_or_else(UNKNOWN),
                    suffix: entry.suffix,
                });
            }

            entries.push(AuditSectionEntry {
                prefix_left: None,
                prefix_right: None,
                text: "".to_string(),
                suffix: None,
            });
        }
        // pop last empty line ^
        entries.pop();

        AuditSection {
            name: "Backtrace".to_string(),
            color: DynColors::Ansi(AnsiColors::Yellow),
            entries,
        }
    }

    fn read_backtrace(&self, audit: &Audit, errors: &Errors) -> Vec<ReporterFile> {
        let mut backtrace = audit.backtrace.clone();
        backtrace.resolve();
        let mut files = Vec::new();

        let mut old_path = None;
        let mut entries = Vec::new();
        for frame in backtrace.frames() {
            for symbols in frame.symbols() {
                let symbol: ErrorLocationKey = symbols.into();

                let mut suffix = None;
                if let Some(err) = errors.get(&symbol) {
                    let errors: Vec<String> = err
                        .iter()
                        .map(|id| {
                            format!("E{id}").red().to_string()
                        })
                        .collect();

                    suffix = Some(format!(
                        "{arrow} {errors}",
                        arrow = "".white(),
                        errors = errors.join(&" ".white().to_string()),
                    ));
                }

                if old_path != symbol.filename {
                    let mut values = Vec::new();
                    swap(&mut values, &mut entries);
                    files.push(ReporterFile::new(old_path, values));
                    old_path = symbol.filename;
                }

                entries.push(ReporterEntry::new(
                    old_path.as_ref(),
                    symbols.lineno(),
                    symbols.colno(),
                    symbols.name().map(|v| {
                        format!("{v}")
                    }),
                    suffix,
                ));
            }
        }

        files
    }

    fn filter(&self, files: &mut Vec<ReporterFile>) {
        let mut out = Vec::new();
        for file in files.drain(..) {
            let mut entries = Vec::new();
            for entry in file.entries {
                if let Some(value) = entry.module.as_deref() {
                    if self.filtered_crates.contains(value) {
                        continue;
                    }
                }

                if let Some(value) = entry.value.as_deref() {
                    if self.filtered.contains(value) {
                        continue;
                    }
                }
                entries.push(entry);
            }
            out.push(ReporterFile {
                path: file.path,
                entries,
            })
        }

        *files = out;
    }

    fn remove_audit_creation(&self, files: &mut [ReporterFile]) {
        self.try_remove(files, "audit::Audit::new_empty");
        self.try_remove(files, "audit::Audit::new");
        if self.try_remove(
            files,
            "result::Result<T,E> -> anyways::ext::AuditExt<T>::wrap",
        ) {
            self.try_remove(files, "ext::AuditExt::wrap_err");
            self.try_remove(files, "ext::AuditExt::wrap_err_with");
            self.try_remove(files, "ext::AuditExt::wrap_section");
            self.try_remove(files, "ext::AuditExt::wrap_section_with");
        } else if self.try_remove(files, "audit::Audit -> core::convert::From<E>::from") {
            self.try_remove(files, "result::Result<T,F> -> core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible,E>>::from_residual");
        }
    }

    fn try_remove(&self, files: &mut [ReporterFile], target: &str) -> bool {
        for file in files {
            if let Some(entry) = file.entries.get(0) {
                let mut remove = false;
                if let Some(value) = entry.value.as_deref() {
                    if value == target {
                        remove = true;
                    }
                }

                return if remove {
                    file.entries.remove(0);
                    true
                } else {
                    false
                };
            }
        }

        false
    }

    fn cleanup(&self, files: &mut Vec<ReporterFile>) {
        let mut out: Vec<ReporterFile> = Vec::new();
        for file in files.drain(..) {
            if file.entries.is_empty() {
                continue;
            }

            let mut collapsable = 0;
            for entry in &file.entries {
                if let Some(value) = entry.value.as_deref() {
                    if self.collapsable.contains(value) {
                        collapsable += 1;
                    }
                }
            }

            if let Some(last) = out.last_mut() {
                if collapsable == file.entries.len() || last.path == file.path {
                    for entry in file.entries {
                        last.entries.push(entry);
                    }
                    continue;
                }
            }

            out.push(file);
        }
        *files = out;
    }

    fn colorize(&self, files: &mut Vec<ReporterFile>) {
        for file in files {
            for entry in &mut file.entries {
                if let Some(value) = &mut entry.value {
                    *value = value.replace("|fn|", &"|fn|".green().to_string());
                    *value = value.replace(".call()", &".call()".green().to_string());
                    *value = value.replace("try?", &"try?".green().to_string());

                    *value = value.replace("->", &"->".white().to_string());

                    *value = value.replace("box", &"box".red().to_string());
                }
            }
        }
    }
}

pub type Errors = HashMap<ErrorLocationKey, Vec<usize>>;

#[derive(Clone)]
pub struct ErrorLocationKey {
    pub name: Option<Vec<u8>>,
    pub filename: Option<PathBuf>,
}

impl Eq for ErrorLocationKey {}
impl PartialEq for ErrorLocationKey {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name) && self.filename.eq(&other.filename)
    }
}

impl Hash for ErrorLocationKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.filename.hash(state);
    }
}

impl From<&Symbol> for ErrorLocationKey {
    fn from(symbol: &Symbol) -> Self {
        ErrorLocationKey {
            name: symbol.name().map(|m| m.as_bytes().to_vec()),
            filename: symbol.filename().map(|m| m.to_owned()),
        }
    }
}
impl From<&BacktraceSymbol> for ErrorLocationKey {
    fn from(symbol: &BacktraceSymbol) -> Self {
        ErrorLocationKey {
            name: symbol.name().map(|m| m.as_bytes().to_vec()),
            filename: symbol.filename().map(|m| m.to_owned()),
        }
    }
}