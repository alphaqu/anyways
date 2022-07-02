use std::env;
use std::path::{Component, PathBuf};
use crate::processor::entry::ProcessingEntry;

pub struct ReporterFile {
	pub path: Option<PathBuf>,
	pub entries: Vec<ProcessingEntry>,
}

impl ReporterFile {
	pub fn new(mut path: Option<PathBuf>, entries: Vec<ProcessingEntry>, file_remove_library_prefix: bool, file_shorten_current_dir: bool) -> ReporterFile {
		if let Some(path) = &mut path {
			if file_shorten_current_dir {
				// If possible make absolute path relative
				if let Ok(current_dir) = env::current_dir() {
					if let Ok(out) = path.strip_prefix(current_dir) {
						*path = PathBuf::from(out);
					}
				}
			}

			if file_remove_library_prefix {
				// If the path is /rustc/*/library, remove the prefix as its a rust cargo library path.
				let mut components = path.components();
				if let Some(Component::RootDir) = components.next() {
					if let Some(v) = components.next() {
						if v.as_os_str() == "rustc" {
							components.next();
							if let Some(v) = components.next() {
								if v.as_os_str() == "library" {
									*path = components.collect();
								}
							}
						}
					}
				}
			}
		}

		ReporterFile { path, entries }
	}
}
