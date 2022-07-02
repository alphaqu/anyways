use owo_colors::{DynColors, OwoColorize};
use std::fmt;
use std::fmt::{Formatter, Write};

use crate::audit::{AuditSection, AuditSectionEntry};
use crate::align::{align, Alignment, PaddingEntry};

pub trait AuditFormatter: Sync {
    fn format(&self, f: &mut Formatter, sections: &[AuditSection]) -> fmt::Result;
}

pub struct AnywaysAuditFormatter {
    /// The total width of the section
    pub width: u32,
    /// The amount of padding on both sides of the section entries.
    pub side_padding: u32,
    /// If the section should be in a simplified view
    pub simple_section: bool,

    pub prefix_padding: usize,
    pub prefix_left_padding: usize,
    pub prefix_right_padding: usize,
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
            prefix_padding: 3,
            prefix_left_padding: 8,
            prefix_right_padding: 12
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
        let mut entries = Vec::new();
        // Prefix
        entries.push(PaddingEntry {
            text: entry.prefix.as_deref().unwrap_or("").to_string(),
            width: 3,
            alignment: Alignment::Left,
        });


        // Prefix Left
        if let Some(value) = &entry.prefix_left {
            entries.push(PaddingEntry {
                text: value.clone(),
                width: 8,
                alignment: Alignment::Right,
            });
        }
        // Prefix Separator
        if entry.prefix_left.is_some() || entry.prefix_right.is_some() {
            entries.push(PaddingEntry {
                text: {
                    entry.separator.white().to_string()
                },
                width: 3,
                alignment: Alignment::Center,
            });
        }

        // Prefix Right
        if let Some(value) = &entry.prefix_right {
            entries.push(PaddingEntry {
                text: value.clone(),
                width: 10,
                alignment: Alignment::Left,
            });
        }


        // Text
        entries.push(PaddingEntry {
            text: entry.text.clone(),
            width: 0,
            alignment: Alignment::Left,
        });

        let pad = " ".repeat(self.side_padding as usize);
        let s = "│".color(color).to_string();
        let max_width = 116;
        align(&entries, max_width, ' ', |line| {
            let mut fill = " ".repeat(max_width.saturating_sub(get_length(&line)));
            writeln!(f, "{s}{pad}{line}{fill}{pad}{s}")
        })?;
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
