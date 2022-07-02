use owo_colors::{DynColors, OwoColorize};
use std::fmt;
use std::fmt::{Formatter, Write};

use crate::audit::{AuditSection, AuditSectionEntry};

pub trait AuditFormatter: Sync {
    fn format(&self, f: &mut Formatter, sections: &[AuditSection]) -> fmt::Result;
}

pub struct AnywaysAuditFormatter {
    pub width: u32,
    pub side_padding: u32,
    pub simple_section: bool,
}

impl AuditFormatter for AnywaysAuditFormatter {
    fn format(&self, f: &mut Formatter, sections: &[AuditSection]) -> fmt::Result {
        self.format(f, sections)
    }
}

impl Default for AnywaysAuditFormatter {
    fn default() -> Self {
        AnywaysAuditFormatter {
            width: 120,
            side_padding: 1,
            simple_section: false,
        }
    }
}

impl AnywaysAuditFormatter {
    pub fn format(&self, f: &mut Formatter<'_>, sections: &[AuditSection]) -> fmt::Result {
        for section in sections {
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

        if entry.prefix_left.is_some() || entry.prefix_right.is_some() {
            if entry.suffix.is_some() | entry.prefix.is_some() {
                write!(&mut text, " {} ", "+".color(color).bold())?;
            } else {
                write!(&mut text, " {} ", entry.separator.to_string().white())?;
            }
        }

        if let Some(right) = &entry.prefix_right {
            write!(&mut text, "{}{}", right, create_pad(" ", right, 10))?;
        }

        write!(&mut text, "{}", entry.text)?;

        // TODO make this less cringe
        let content_width = self.width as usize - (self.side_padding as usize * 2) - 2;
        if get_length(&text) > content_width {
            let mut line = String::new();
            let mut suffix = entry.suffix.as_deref();
            let mut prefix = entry.prefix.as_deref();
            for ch in text.chars() {
                line.push(ch);
                let length = suffix.map(get_length).unwrap_or(0) + get_length(&line);
                if length == content_width {
                    self.write_section_line(f, &line, suffix.take(), prefix.take(), color)?;
                    line.clear();
                }
            }

            if !line.is_empty() {
                self.write_section_line(f, &line, suffix.take(), prefix.take(), color)?;
            }
        } else {
            self.write_section_line(f, &text, entry.suffix.as_deref(), entry.prefix.as_deref(), color)?;
        }

        Ok(())
    }

    fn write_section_header(
        &self,
        f: &mut Formatter<'_>,
        text: &str,
        color: DynColors,
    ) -> fmt::Result {
        if self.simple_section {
            writeln!(f, "{}{}", "==> ".color(color), text.bold(),)
        } else {
            writeln!(
                f,
                "{}{} {}{}",
                "╭── ".color(color),
                text.bold(),
                create_pad(&"─".color(color).to_string(), text, self.width as usize - 6),
                "╮".color(color)
            )
        }
    }

    fn write_section_line(
        &self,
        f: &mut Formatter<'_>,
        text: &str,
        suffix: Option<&str>,
        prefix: Option<&str>,
        color: DynColors,
    ) -> fmt::Result {
        let suffix = suffix.unwrap_or_default();
        let prefix = prefix.unwrap_or_default();
        let ls = if self.simple_section { "" } else { "│" }
            .color(color)
            .to_string();

        if !self.simple_section {
            let pad = " ".repeat(self.side_padding as usize);
            let prefix_pad = create_pad(" ", prefix, 3);
            writeln!(
                f,
                "{ls}{pad}{prefix}{prefix_pad}{text}{}{suffix}{pad}{ls}",
                create_pad(
                    " ",
                    text,
                    self.width as usize
                        - get_length(&ls)
                        - get_length(&ls)
                        - get_length(&prefix_pad)
                        - get_length(suffix)
                        - get_length(prefix)
                        - self.side_padding as usize * 2,
                )
            )
        } else {
            writeln!(f, "{}{suffix}{text}", create_pad(" ", suffix, 4))
        }
    }

    fn write_section_footer(&self, f: &mut Formatter<'_>, color: DynColors) -> fmt::Result {
        if !self.simple_section {
            writeln!(
                f,
                "{}{}{}",
                "╰".color(color),
                "─".color(color).to_string().repeat(self.width as usize - 2),
                "╯".color(color)
            )
        } else {
            writeln!(f)
        }
    }
}

// Repeats the value until it can pad the text
pub fn create_pad(value: &str, text: &str, length: usize) -> String {
    value.repeat(length.saturating_sub(get_length(text)))
}

// Get length of a string skipping all ansi color codes.
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
