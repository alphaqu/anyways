use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::mem::swap;
use std::path::PathBuf;

use backtrace::{resolve_frame, BacktraceSymbol, Symbol};
use owo_colors::{AnsiColors, DynColors, OwoColorize, Style};

use entry::{ProcessingEntry, ProcessingValueMatcher};
use file::ReporterFile;

use crate::audit::{Audit, AuditSection, AuditSectionEntry};

pub mod entry;
pub mod file;

pub(crate) const UNKNOWN: fn() -> String = || "???".to_string();

pub trait AuditProcessor: Sync {
    /// This processes the audit which later gets formatted by an AuditReporter
    fn process(&self, audit: &Audit) -> Vec<AuditSection>;
}

pub struct AnywaysAuditProcessorBuilder {
    pub shorten_result: bool,
    pub shorten_box: bool,
    pub shorten_closure: bool,
    pub shorten_try_from: bool,

    pub collapse_try_from: bool,
    pub collapse_closure: bool,

    pub file_remove_library_prefix: bool,
    pub file_shorten_current_dir: bool,
    pub replace_style: Style,
}

impl Default for AnywaysAuditProcessorBuilder {
    fn default() -> Self {
        AnywaysAuditProcessorBuilder {
            shorten_result: true,
            shorten_box: true,
            shorten_closure: true,
            shorten_try_from: true,
            collapse_try_from: true,
            collapse_closure: true,
            file_remove_library_prefix: true,
            file_shorten_current_dir: true,
            replace_style: Style::new().cyan()
        }
    }
}

impl AnywaysAuditProcessorBuilder {
    pub fn build(self) -> AnywaysAuditProcessor {
        let filter = HashSet::new();
        let mut replace = Vec::new();
        let mut collapse = HashSet::new();

        if self.shorten_result {
            replace.push((
                ProcessingValueMatcher::Item("result::Result<T,E>".to_string()),
                "result".to_string(),
            ));
            replace.push((
                ProcessingValueMatcher::Item("result::Result<T,F>".to_string()),
                "result".to_string(),
            ));
        }

        if self.shorten_box {
            replace.push((
                ProcessingValueMatcher::Item("boxed::Box<F,A>".to_string()),
                "box".to_string(),
            ));
        }

        if self.shorten_closure {
            replace.push((
                ProcessingValueMatcher::Value("ops::function::FnOnce<Args>::call_once".to_string()),
                "closure()".to_string(),
            ));
            replace.push((
                ProcessingValueMatcher::Value("ops::function::FnOnce<()>::call_once".to_string()),
                "closure()".to_string(),
            ));
            replace.push((
                ProcessingValueMatcher::Value("ops::function::FnOnce::call_once".to_string()),
                "closure()".to_string(),
            ));
            replace.push((
                ProcessingValueMatcher::Item("ops::function::FnOnce<Args>".to_string()),
                "closure".to_string(),
            ));
            replace.push((
                ProcessingValueMatcher::Item("ops::function::FnOnce<()>".to_string()),
                "closure".to_string(),
            ));
            replace.push((
                ProcessingValueMatcher::Item("ops::function::FnOnce".to_string()),
                "closure".to_string(),
            ));
            replace.push((
                ProcessingValueMatcher::Path("{{closure}}".to_string()),
                "closure".to_string(),
            ));
        }

        if self.shorten_try_from {
            replace.push((
                ProcessingValueMatcher::Value("convert::From<E>::from".to_string()),
                "from".to_string(),
            ));
            replace.push((ProcessingValueMatcher::Value("ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible,E>>::from_residual".to_string()), "try".to_string(), ));
        }

        if self.collapse_try_from {
            collapse.insert(ProcessingValueMatcher::Value(
                "ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible,E>>::from_residual".to_string(),
            ));
        }

        if self.collapse_closure {
            collapse.insert(ProcessingValueMatcher::Item(
                "ops::function::FnOnce::call_once".to_string(),
            ));
        }

        AnywaysAuditProcessor {
            filter,
            replace,
            collapse,
            replace_style: self.replace_style,
            file_remove_library_prefix: self.file_remove_library_prefix,
            file_shorten_current_dir: self.file_shorten_current_dir,
        }
    }
}

pub struct AnywaysAuditProcessor {
    /// If a filter gets matched the entry will get removed
    pub filter: HashSet<ProcessingValueMatcher>,

    /// If a shorthand get matched the entry will get replaced with the right side and colored on its default.
    /// You can locally overwrite this by just setting a color.
    pub replace: Vec<(ProcessingValueMatcher, String)>,
    pub replace_style: Style,

    /// If a collapse gets matched the entry will be allowed to move outside of its file and inline its usage.
    pub collapse: HashSet<ProcessingValueMatcher>,

    /// Removes /rustc/hashisreallycoolhere/library prefix which is present in every library
    pub file_remove_library_prefix: bool,
    /// If the source file is bound to the current directory it will get shortened to ./
    pub file_shorten_current_dir: bool,
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
            // Try to resolve the location and append that to the errors lookup.
            // This is later used to indicate where an error has occured in the backtrace.
            if let Some(value) = &err.location {
                resolve_frame(value, |symbol| {
                    let location: ErrorLocationKey = symbol.into();
                    if !errors.contains_key(&location) {
                        errors.insert(location.clone(), Vec::new());
                    }
                    errors.get_mut(&location).unwrap().push(i);
                });
            }

            // Push the section entry.
            entries.push(AuditSectionEntry {
                prefix: None,
                prefix_left: Some(format!("E{i}").red().to_string()),
                separator: if i != audit.errors.len() - 1 {
                    '↓'
                } else {
                    '→'
                },
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
        // Apply filter on entries.
        let mut files: Vec<ReporterFile> = Vec::new();
        for mut file in self.read_backtrace(audit, errors) {
            let mut entries = Vec::new();
            'entry: for mut entry in file.entries {
                // If any filter matches skip this entry
                for matcher in &self.filter {
                    if entry.value.matches(matcher) {
                        continue 'entry;
                    }
                }

                // Check if this entry is collapsable
                for matcher in &self.collapse {
                    if entry.value.matches(matcher) {
                        entry.collapsable = true;
                        break;
                    }
                }

                // Replace everything you can.
                for (matcher, to) in &self.replace {
                    entry
                        .value
                        .replace(matcher, &(to.style(self.replace_style).to_string()));
                }

                entries.push(entry);
            }

            // If the entries is empty do not append the file
            if entries.is_empty() {
                continue;
            }

            // If all of the entries are collapsable you can append the entries to the previous file.
            if let Some(last) = files.last_mut() {
                if last.path == file.path || entries.iter().all(|v| v.collapsable) {
                    for entry in entries {
                        last.entries.push(entry);
                    }
                    continue;
                }
            }

            file.entries = entries;
            files.push(file);
        }

        let mut entries = Vec::new();
        for file in files {
            // File name
            entries.push(AuditSectionEntry::text(
                file.path
                    .map(|p| p.to_str().unwrap().white().bold().to_string())
                    .unwrap_or_else(UNKNOWN),
            ));

            // File entries
            for entry in file.entries {
                entries.push(entry.build());
            }

            // Just a spacer after the files
            entries.push(AuditSectionEntry::empty());
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
        // Make sure that the backtrace is resolved.
        // We need to sadly clone the backtrace as we do not have a mutable reference in here to resolve the backtrace,
        // which leaves opportunity for other Audits to also resolve their backtrace.
        // However Audits are mostly only reported once so ¯\_(ツ)_/¯
        backtrace.resolve();

        let mut files = Vec::new();
        let mut entries = Vec::new();
        let mut old_path = None;

        for frame in backtrace.frames() {
            for symbol in frame.symbols() {
                let location: ErrorLocationKey = symbol.into();

                if old_path != location.filename {
                    let mut values = Vec::new();
                    swap(&mut values, &mut entries);
                    files.push(ReporterFile::new(old_path, values, self.file_remove_library_prefix, self.file_shorten_current_dir));
                    old_path = location.filename.clone();
                }

                entries.push(ProcessingEntry::new(symbol, errors));
            }
        }

        files
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
