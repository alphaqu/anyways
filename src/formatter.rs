pub mod file;
pub mod entry;

use std::{fmt};
use std::fmt::{Formatter, Write};
use owo_colors::{DynColors, OwoColorize};

use crate::audit::{Audit, AuditSectionEntry};

pub trait AuditFormatter: Sync  {
    fn format(&self, f: &mut Formatter, audit: &Audit) -> fmt::Result;
}

pub struct AnywaysAuditFormatter {
    pub width: u32,
}

impl AuditFormatter for AnywaysAuditFormatter {
    fn format(&self, f: &mut Formatter, audit: &Audit) -> fmt::Result {
        self.format(f, audit)
    }
}

impl Default for AnywaysAuditFormatter {
    fn default() -> Self {
        AnywaysAuditFormatter {
            width: 120,
        }
    }
}

impl AnywaysAuditFormatter {
    pub fn format(&self, f: &mut Formatter<'_>, audit: &Audit) -> fmt::Result {
        for section in &audit.sections {
            self.write_section_header(f, &section.name, section.color)?;
            for entry in &section.entries {
                self.write_section_entry(f, entry, section.color)?;
            }
            self.write_section_footer(f, section.color)?;
        }

        Ok(())
    }

    fn write_section_entry(
        &self,
        f: &mut Formatter<'_>,
        entry: &AuditSectionEntry,
        color: DynColors,
    ) -> fmt::Result {
        let mut text = String::new();
        if let Some(left) = &entry.prefix_left {
            write!(&mut text, "{}{}", create_pad(" ", left, 8), left)?;
        }

        if entry.prefix_left.is_some() || entry.prefix_right.is_some()  {
            write!(&mut text, " {} ", "|".white().bold())?;
        }

        if let Some(right) = &entry.prefix_right {
            write!(&mut text, "{}{}", right, create_pad(" ", right, 10))?;
        }

        write!(&mut text, "{}", entry.text)?;

        // TODO make this less cringe
        let content_width = self.width as usize - 4;
        if get_length(&text) > content_width {
            let mut line = String::new();
            for ch in text.chars() {
                line.push(ch);
                if get_length(&line) == content_width {
                    self.write_section_line(f, &line, color)?;
                    line.clear();
                }
            }

            if !line.is_empty() {
                self.write_section_line(f, &line, color)?;
            }
        } else {
            self.write_section_line(f, &text, color)?;
        }

        Ok(())
    }

    fn write_section_header(&self, f: &mut Formatter<'_>, text: &str, color: DynColors) -> fmt::Result {
        writeln!(
            f,
            "{}{} {}{}",
            "╭── ".color(color),
            text.bold(),
            create_pad(&"─".color(color).to_string(), text, self.width as usize - 6),
            "╮".color(color)
        )
    }

    fn write_section_line(&self, f: &mut Formatter<'_>, text: &str, color: DynColors) -> fmt::Result {
        let s = "│".color(color);
        writeln!(
            f,
            "{s} {}{} {s}",
            text,
            create_pad(" ", text, self.width as usize - 4)
        )
    }

    fn write_section_footer(&self, f: &mut Formatter<'_>, color: DynColors) -> fmt::Result {
        writeln!(
            f,
            "{}{}{}",
            "╰".color(color),
            "─".color(color).to_string().repeat(self.width as usize - 2),
            "╯".color(color)
        )
    }
}

pub fn create_pad(value: &str, text: &str, length: usize) -> String {
    value
        .repeat(length.saturating_sub(get_length(text)))
}

pub fn get_length(text: &str) -> usize {
    let mut wait_m = false;
    let mut len = 0;
    for ch in text.chars() {
        if wait_m {
            if ch == 'm' {
                wait_m = false;
            }

            continue;
        } else if ch == '\x1b' {
            wait_m = true;
            continue;
        } else {
            len += 1;
        }
    }

    len
}
