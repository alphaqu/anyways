use crate::audit::{Audit, AuditSection, AuditSectionEntry};
use crate::formatter::entry::ReporterEntry;
use crate::formatter::file::ReporterFile;
use owo_colors::{AnsiColors, DynColors, OwoColorize};
use std::collections::HashSet;
use std::mem::swap;
use std::path::PathBuf;

const UNKNOWN: fn() -> String = || "???".to_string();

pub trait AuditProcessor: Sync {
    /// This processes the audit which later gets formatted by an AuditReporter
    fn process(&self, audit: &Audit) -> Audit;
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
                "anyways".to_string()
            ]),
            collapsable: HashSet::from(["|fn|()".to_string()]),
        }
    }
}

impl AuditProcessor for AnywaysAuditProcessor {
    fn process(&self, audit: &Audit) -> Audit {
        let mut audit = audit.clone();
        self.process(&mut audit);
        audit
    }
}

impl AnywaysAuditProcessor {
    pub fn process(&self, audit: &mut Audit) {
        let error = self.create_error_section(audit);
        audit.sections.push(error);
        let backtrace = self.create_backtrace_section(audit);
        audit.sections.push(backtrace);
    }

    fn create_error_section(&self, audit: &mut Audit) -> AuditSection {
        let mut entries = Vec::new();
        for (i, err) in audit.errors.iter().enumerate() {
            entries.push(AuditSectionEntry {
                prefix_left: Some(
                    format!(
                        "#{i} {}",
                        if i != audit.errors.len() - 1 {
                            "↓"
                        } else {
                            "→"
                        }
                        .white()
                    )
                    .blue()
                    .to_string(),
                ),
                prefix_right: None,
                text: format!("{err}"),
            });
        }
        AuditSection {
            name: "Errors".to_string(),
            color: DynColors::Ansi(AnsiColors::Red),
            entries,
        }
    }

    fn create_backtrace_section(&self, audit: &mut Audit) -> AuditSection {
        let files = self.read_backtrace(audit);
        let files = self.filter(files);
        let mut files = self.cleanup(files);
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
                });
            }

            entries.push(AuditSectionEntry {
                prefix_left: None,
                prefix_right: None,
                text: "".to_string(),
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

    fn read_backtrace(&self, audit: &mut Audit) -> Vec<ReporterFile> {
        audit.backtrace.resolve();
        let mut files = Vec::new();

        let mut old_path = None;
        let mut entries = Vec::new();
        for frame in audit.backtrace.frames() {
            for symbols in frame.symbols() {
                let path = symbols.filename().map(PathBuf::from);
                if old_path != path {
                    let mut values = Vec::new();
                    swap(&mut values, &mut entries);
                    files.push(ReporterFile::new(old_path, values));
                    old_path = path;
                }

                entries.push(ReporterEntry::new(
                    old_path.as_ref(),
                    symbols.lineno(),
                    symbols.colno(),
                    symbols.name().map(|v| v.to_string()),
                ));
            }
        }

        files
    }

    fn filter(&self, files: Vec<ReporterFile>) -> Vec<ReporterFile> {
        let mut out = Vec::new();
        for file in files {
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
        out
    }

    fn cleanup(&self, files: Vec<ReporterFile>) -> Vec<ReporterFile> {
        let mut out: Vec<ReporterFile> = Vec::new();
        for file in files {
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
        out
    }

    fn colorize(&self, files: &mut Vec<ReporterFile>) {
        for file in files {
            for entry in &mut file.entries {
                if let Some(value) = &mut entry.value {
                    *value = value.replace("|fn|", &"|fn|".green().to_string());
                    *value = value.replace(".call()", &".call()".green().to_string());

                    *value = value.replace("->", &"->".white().to_string());

                    *value = value.replace("box", &"box".red().to_string());
                }
            }
        }
    }
}
