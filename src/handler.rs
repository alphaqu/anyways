use std::fmt;
use std::fmt::{Formatter, Pointer, Write};
use owo_colors::OwoColorize;
use crate::default::ReportProcessor;
use crate::Report;

static mut HANDLER: Option<Box<dyn ReportHandler>> = None;


pub fn fmt(f: &mut Formatter<'_>, report: &Report) -> fmt::Result {
	unsafe {
		if HANDLER.is_none() {
			HANDLER = Some(Box::new(DefaultHandler));
		}

		let handler = HANDLER.as_ref().unwrap();

		let mut string = String::new();
		handler.format_report(&mut string, report)?;
		f.write_str(&string)?;
	}

	Ok(())
}

pub trait ReportHandler {
	fn format_report(&self, f: &mut String, report: &Report) -> fmt::Result;
}

pub struct DefaultHandler;

impl ReportHandler for DefaultHandler {
	fn format_report(&self, f: &mut String, report: &Report) -> fmt::Result {
		f.write_char('\n')?;
		let mut howler = ReportProcessor::new(report.clone());
		howler.process();
		howler.format(f)?;

		//
		//
		// 		let mut current_dir = std::env::current_dir().ok().map(|dir| dir.to_str().unwrap().to_string());
		//
		// 		let mut last_file: Option<String> = None;
		// 		let mut last_entry = 0;
		// 		let mut len = report.backtrace.len();
		// 		for (i, entry) in report.backtrace.iter().enumerate() {
		// 			let i = (len - 1) - i;
		// 			for symbol in &entry.symbols {
		// 				if let Some(name) = &symbol.file {
		// 					let mut name = name.to_str().unwrap().to_string();
		// 					if let Some(current_dir) = &current_dir {
		// 						if name.starts_with(current_dir) {
		// 							name = name.replacen(current_dir, ".", 1);
		// 						}
		// 					}
		//
		// 					if name.starts_with("/rustc/") {
		// 						if let Some((_, lib)) = name.split_once("library") {
		// 							name = lib.to_string();
		// 						}
		// 					}
		//
		// 					let mut name = format!("\n {} {}", " ".repeat(5), name.white().bold());
		// 					if let Some(last_file) = last_file.as_ref() {
		// 						if name != last_file.as_str() {
		// 							writeln!(f, "{name}")?;
		// 						}
		// 					} else {
		// 						writeln!(f, "{name}")?;
		// 					}
		//
		// 					last_file = Some(name);
		// 				}
		//
		// 				let mut prefix = String::new();
		// 				if let Some(name) = &symbol.lineno {
		// 					write!(&mut prefix, "{}", name)?;
		// 				}
		//
		// 				if let Some(name) = &symbol.colno {
		// 					write!(&mut prefix, "{}", ":")?;
		// 					write!(&mut prefix, "{}", name)?;
		// 				}
		//
		// 				write!(f, "{}{}", format!("{: >6}", prefix).blue().bold(), " | ".white().bold())?;
		//
		//
		// 				if let Some(name) = &symbol.name {
		// 					let mut name = name.to_string();
		// 					//for x in STD_SHORTHANDS {
		// 					//	if name.starts_with(x) {
		// 					//		name = format!("{}::{}","std", &name[x.len()..]);
		// 					//		break;
		// 					//	}
		// 					//}
		// //
		// 					//for x in CORE_SHORTHANDS {
		// 					//	if name.starts_with(x) {
		// 					//		name = format!("{}::{}", "core", &name[x.len()..]);
		// 					//		break;
		// 					//	}
		// 					//}
		//
		//
		//
		// 					//let name = name.replace("::{{closure}}", &*" <- Fn()".green().to_string());
		// 					//let mut name = name.replace("{{vtable.shim}}", &*" VTable()".green().to_string());
		// 					//let mut name = match name.rsplit_once("::") {
		// 					//	Some((left, right)) => {
		// 					//		//let string = match left.rsplit_once("::") {
		// 					//		//	None => {
		// 					//		//		left.to_string()
		// 					//		//	}
		// 					//		//	Some((left, right)) => {
		// 					//		//		if !right.is_empty() && right.chars().next().unwrap().is_uppercase() {
		// 					//		//			format!("{left}::{}", righ))
		// 					//		//		} else {
		// 					//		//			format!("{left}::{right}")
		// 					//		//		}
		// 					//		//	}
		// 					//		//};
		// 					//		format!("{}{}{}", left, "::".to_string(), right.yellow())
		// 					//	}
		// 					//	None => {
		// 					//		name
		// 					//	}
		// 					//};
		//
		// 					if !name.starts_with("<") {
		// 						if let Some((left, right)) =name.split_once("::") {
		// 							name = format!("{} {}", format!("{: <10}", left).cyan(), right);
		// 						}
		// 					}
		//
		//
		//
		// 					let name = name.replace("::", &*"::".white().to_string());
		// 					write!(f, "{}", name)?;
		// 				}
		//
		// 				writeln!(f)?;
		// 			}
		// 		}
		//

		Ok(())
	}
}

pub const STD_SHORTHANDS: [&str; 22] = [
	"std::f32::",
	"std::f64::",
	"std::thread::",
	"std::ascii::",
	"std::backtrace::",
	"std::collections::",
	"std::env::",
	"std::error::",
	"std::sys::",
	"std::ffi::",
	"std::fs::",
	"std::io::",
	"std::net::",
	"std::num::",
	"std::os::",
	"std::panic::",
	"std::path::",
	"std::process::",
	"std::sync::",
	"std::time::",
	"std::panicking::",
	"std::sys_common::",
];

pub const CORE_SHORTHANDS: [&str; 1] = [
	"core::ops::"
];