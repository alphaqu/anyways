use std::path::PathBuf;

pub struct ReporterEntry {
	pub line: Option<u32>,
	pub character: Option<u32>,
	pub module: Option<String>,
	pub value: Option<String>,
	pub suffix: Option<String>,
}

impl ReporterEntry {
	pub fn new(
		path: Option<&PathBuf>,
		line: Option<u32>,
		character: Option<u32>,
		mut value: Option<String>,
		suffix: Option<String>,
	) -> ReporterEntry {
		let mut module = None;
		if let Some(value) = &mut value {
			Self::strip_hash(value);
			module = Self::acquire_module(value);
			Self::apply_shorthands(value);
		}

		ReporterEntry {
			line,
			character,
			module,
			value,
			suffix
		}
	}

	fn strip_hash(value: &mut String) {
		if let Some((out, _)) = value.rsplit_once("::") {
			*value = out.to_string();
		}
	}

	fn acquire_module(value: &mut String) -> Option<String> {
		let mut module = None;
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

			if let Some((mut item, as_item)) = statement.split_once(" as ") {
				if let Some((module_out, out)) = item.split_once("::") {
					module = Some(module_out.to_string());
					// dont include module
					item = out;
				}
				*value = format!("{item} -> {as_item}{rest}");
			}
		} else {
			// If not as then the first :: will be the module
			if let Some((module_out, out)) = value.split_once("::") {
				module = Some(module_out.to_string());
				*value = out.to_string();
			}
		}
		module
	}

	fn apply_shorthands(value: &mut String) {
		*value = value.replace("ops::function::FnOnce::call_once", "|fn|.call()");
		*value = value.replace("core::ops::function::FnOnce<()>::call_once", "|fn|.call()");
		*value = value.replace("core::ops::function::FnOnce<Args>::call_once", "|fn|.call()");

		*value = value.replace("boxed::Box<F,A>", "box");
		*value = value.replace("{{closure}}", "|fn|");
		*value = value.replace("{{vtable.shim}}", "[vtable.shim]");
	}
}