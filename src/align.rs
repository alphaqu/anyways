//! Padding utilities

use crate::formatter::get_length;
use std::fmt;
use std::fmt::Write;

pub struct PaddingEntry {
    pub text: String,
    pub width: usize,
    pub alignment: Alignment,
}

pub enum Alignment {
    Left,
    Center,
    Right,
}

pub fn align(
    entries: &Vec<PaddingEntry>,
    max_width: usize,
    pad_char: char,
    mut func: impl FnMut(String) -> fmt::Result,
) -> fmt::Result {
    let mut offset = 0;
    let mut line = String::new();
    for entry in entries {
        let text = &entry.text;
        let length = get_length(text);
        if length > entry.width {
            offset += length - entry.width;
            line.push_str(text);
        } else {
            let mut padding_size = entry.width - length;

            // try to correct the offset
            let offset_amount = offset.clamp(0, padding_size);
            offset -= offset_amount;
            padding_size -= offset_amount;

            let padding = pad_char.to_string().repeat(padding_size);
            match entry.alignment {
                Alignment::Left => {
                    write!(&mut line, "{text}{padding}")?;
                }
                Alignment::Center => {
                    let (left_padding, right_padding) = padding.split_at(padding.len() / 2);
                    write!(&mut line, "{left_padding}{text}{right_padding}")?;
                }
                Alignment::Right => {
                    write!(&mut line, "{padding}{text}")?;
                }
            }
        }
    }

    if get_length(&line) > max_width {
        let mut line_builder = String::new();
        for ch in line.chars() {
            line_builder.push(ch);
            if get_length(&line_builder) >= max_width {
                func(line_builder)?;
                line_builder = String::new();
            }
        }

        if !line_builder.is_empty() {
            func(line_builder)?;
        }
    } else {
        func(line)?;
    }

    Ok(())
}
