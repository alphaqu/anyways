use entry::HowlerEntry;
use file::HowlerFile;
use owo_colors::OwoColorize;
use std::{fmt};
use std::error::Error;
use std::fmt::Write;
use std::mem::swap;
use std::path::PathBuf;

use crate::Report;

mod entry;
mod file;

pub struct ProcessorSettings {
	// formatting
	// place | location        item
	/// Pads to the left of the pipe
	place_width: usize,
	/// Pads to the right of the module text.
	location_width: usize,

	// features
	file_strip_current_dir: bool,
	file_strip_library: bool,
	remove_hash: bool,
}

/// A very nice, simplistic and informative report processor.
pub struct ReportProcessor {
	pub report: Report,
	pub settings: ProcessorSettings,
	pub files: Vec<HowlerFile>,
}

impl ReportProcessor {
	pub fn new(mut report: Report) -> ReportProcessor {
		let mut last_file: Option<PathBuf> = None;

		let mut files = Vec::new();
		let mut entries = Vec::new();

		report.backtrace.resolve();
		for frame in report.backtrace.frames() {
			for symbol in frame.symbols() {
				let file = symbol.filename().map(PathBuf::from);
				if last_file != file {
					if !entries.is_empty() {
						let mut new_entries = Vec::new();
						swap(&mut new_entries, &mut entries);

						files.push(HowlerFile::new(last_file, new_entries));
					}

					last_file = file.clone();
				}

				entries.push(HowlerEntry::new(symbol));
			}
		}

		if !entries.is_empty() {
			files.push(HowlerFile::new(last_file, entries));
		}

		ReportProcessor {
			report,
			settings: ProcessorSettings {
				place_width: 6,
				location_width: 10,
				file_strip_current_dir: true,
				file_strip_library: true,
				remove_hash: true,
			},
			files,
		}
	}

	pub fn format(self, f: &mut String) -> fmt::Result {
		Self::write_title(&self.settings, f, "Errors:")?;
		let mut err = 0;
		for error in self.report.errors {
			let current: &dyn Error = &*(*error);
			Self::write_line(&self.settings, f, &format!("#{err}"), "", &current.to_string())?;
			err += 1;
			while let Some(child) = current.source() {
				Self::write_line(&self.settings, f, &format!("#{err}"), "", &child.to_string())?;
				err += 1;
			}
		}


		for section in self.report.sections {
			Self::write_title(&self.settings, f, &section.name)?;
			for entry in section.entries {
				Self::write_line(&self.settings, f, &entry.prefix, &entry.suffix, &entry.value)?;
			}
		}


		Self::write_title(&self.settings, f, "Backtrace:")?;
		for file in self.files {
			write!(f, "{}", " ".repeat(self.settings.place_width + 1))?;
			writeln!(
				f,
				"{}",
				file.file
					.unwrap_or_else(|| "unknown file".to_string())
					.white()
					.bold()
			)?;
			for entry in file.entries {
				// place
				let mut prefix = String::new();
				if let Some(value) = entry.line {
					prefix.write_str(&value.to_string())?;
				}
				if let Some(value) = entry.character {
					prefix.write_char(':')?;
					prefix.write_str(&value.to_string())?;
				}
				// module
				let suffix = entry.module.unwrap_or_else(|| "???".to_string());

				// item
				let mut value = entry.value.unwrap_or_else(|| "???".to_string());

				// keywords
				value = value.replace('↑', &"↑".white().to_string());
				value = value.replace("->", &"->".white().to_string());
				value = value.replace("Deref::deref::", &"deref::".red().to_string());
				value = value.replace("boxed::Box<F,A>", &"box".red().to_string());

				value = value.replace("FnOnce", &"FnOnce".green().to_string());
				value = value.replace("|fn|", &"|fn|".green().to_string());
				value = value.replace("[vtable]", &"[vtable]".green().to_string());

				// mute colons
				value = value.replace("::", &"::".white().to_string());

				Self::write_line(&self.settings, f, &prefix, &suffix, &value)?;
			}

			writeln!(f)?;
		}

		Ok(())
	}

	fn write_title(settings: &ProcessorSettings, f: &mut String, name: &str) -> fmt::Result {
		writeln!(f)?;
		writeln!(f, "{}{} {}",
		       "=".repeat(settings.place_width).red().bold(),
		       "=>".red().bold(),
		       name.bright_white().bold()
		)?;


		//let mut len = 80 - name.len();
		//writeln!(f, "{} {} {}",
		//         "━".repeat((len as f32 / 2.0).ceil() as usize).red(),
		//         name.bright_white().bold(),
		//         "━".repeat((len as f32 / 2.0).floor() as usize).red()
		//)?;

		Ok(())
	}
	fn write_line(settings: &ProcessorSettings, f: &mut String, prefix: &str, suffix: &str, value: &str) -> fmt::Result {
		if !prefix.is_empty() {
			write!(
				f,
				"{}",
				format!("{prefix: >width$}", width = settings.place_width)
					.blue()
					.bold()
			)?;
		}

		write!(f, "{}", " | ".white().bold())?;

		if !suffix.is_empty() {
			write!(
				f,
				"{}",
				format!("{suffix: <width$}", width = settings.location_width)
					.purple()
					.bold()
			)?;
		}

		writeln!(f, "{value}")
	}

	pub fn process(&mut self) {
		self.file_strip_current_dir();
		self.file_strip_library();
		self.remove_hash();
		self.pop_top();
		self.simplify_closure();
		self.simplify_vtable();
		self.compact_closures();
		self.cleanup();
	}

	/// removes Report::new
	fn pop_top(&mut self) {
		for value in &mut self.files {
			let mut pop = 0;
			for value in &value.entries {
				if let Some(value) = &value.value {
					if value == "Report::new" || value == "result::Result<T,I> -> ReportExt<T>::wrap_err" {
						pop += 1;
					} else {
						break;
					}
				}
			}

			for _ in 0..pop {
				value.entries.remove(0);
			}
		}
	}

	/// # file_strip_current_dir
	/// If the file name starts with the directory you currently are in.
	/// It will replace that path prefix with ./
	fn file_strip_current_dir(&mut self) {
		if let Ok(current_dir) = std::env::current_dir() {
			if let Some(current_dir) = current_dir.to_str() {
				for file in &mut self.files {
					if let Some(file) = &mut file.file {
						if file.starts_with(current_dir) {
							*file = file.replacen(current_dir, ".", 1);
						}
					}
				}
			}
		}
	}

	/// # file_strip_library
	/// If the file name starts with /rustc/, we can determine that this is a path to a library.
	/// To save space we can remove /rustc/hashingisreallypogbutlong/library/ to save space.
	fn file_strip_library(&mut self) {
		for file in &mut self.files {
			if let Some(file) = &mut file.file {
				if file.starts_with("/rustc/") {
					if let Some((_, lib)) = file.split_once("library") {
						*file = lib.to_string();
					}
				}
			}
		}
	}


	/// # remove_hash
	/// Removes the item hash.
	fn remove_hash(&mut self) {
		for file in &mut self.files {
			for entry in &mut file.entries {
				if let Some(name) = &mut entry.value {
					*name = name.rsplitn(2, "::").skip(1).collect();
				}
			}
		}
	}

	fn simplify_closure(&mut self) {
		for file in &mut self.files {
			for entry in &mut file.entries {
				if let Some(name) = &mut entry.value {
					*name = name.replace("{{closure}}", "|fn|");
				}
			}
		}
	}

	fn simplify_vtable(&mut self) {
		for file in &mut self.files {
			for entry in &mut file.entries {
				if let Some(name) = &mut entry.value {
					*name = name.replace("{{vtable.shim}}", "[vtable]");
				}
			}
		}
	}

	fn compact_closures(&mut self) {
		let mut next_entries = Vec::new();
		for file in &mut self.files {
			if !next_entries.is_empty() {
				for entry in next_entries.drain(..) {
					file.entries.insert(0, entry);
				}
			}

			let mut closures = 0;
			for entry in &mut file.entries {
				if let Some(name) = &mut entry.value {
					if name == "ops::function::FnOnce::call_once" {
						*name = "↑ FnOnce".to_string();
						closures += 1;
					}
				}
			}

			if closures == file.entries.len() {
				swap(&mut next_entries, &mut file.entries);
			}
		}
	}

	fn mute_double_colon(&mut self) {
		for file in &mut self.files {
			for entry in &mut file.entries {
				if let Some(value) = &mut entry.value {
					*value = value.replace("::", &"::".white().to_string());
				}
			}
		}
	}

	fn cleanup(&mut self) {
		let mut files: Vec<HowlerFile> = Vec::new();
		for file in self.files.drain(..) {
			if !file.entries.is_empty() {
				if let Some(last_file) = files.last_mut() {
					if last_file.file == file.file {
						for x in file.entries {
							last_file.entries.push(x);
						}
					} else {
						files.push(file);
					}
				} else {
					files.push(file);
				}
			}
		}

		self.files = files;
	}
}
