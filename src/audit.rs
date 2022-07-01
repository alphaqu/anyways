use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use std::rc::Rc;
use backtrace::Backtrace;
use owo_colors::{AnsiColors, DynColors};
use crate::{get_audit_formatter, get_audit_processor};

/// An audit is a special error structure.
/// It holds information about the failure and all of the needed information to display it with useful information
#[derive(Clone)]
pub struct Audit {
	pub backtrace: Backtrace,
	pub errors: Vec<Rc<Box<dyn Error + 'static>>>,
	pub sections: Vec<AuditSection>
}

impl Audit {
	pub fn new_empty() -> Audit {
		Audit {
			backtrace: Backtrace::new_unresolved(),
			errors: vec![],
			sections: vec![]
		}
	}

	pub fn new(err: impl Into<Box<dyn Error + 'static>>) -> Audit {
		let mut audit = Audit::new_empty();
		audit.push_err(err);
		audit
	}

	pub fn push_err(&mut self, err: impl Into<Box<dyn Error + 'static>>) -> &mut Self {
		self.errors.insert(0, Rc::new(err.into()));
		self
	}

	pub fn push_section(&mut self, section: impl Into<AuditSection>) -> &mut Self {
		self.sections.push(section.into());
		self
	}
}

impl<E: Into<Box<dyn Error + 'static>>> From<E> for Audit {
	fn from(error: E) -> Self {
		Audit::new(error)
	}
}


// this is actually the print error stuff
impl Debug for Audit  {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let audit = get_audit_processor().process(self);
		f.write_char('\n')?;
		get_audit_formatter().format(f, &audit)
	}
}

#[derive(Clone)]
pub struct AuditInner {
	pub backtrace: Backtrace,
	pub errors: Vec<Rc<Box<dyn Error + 'static>>>,
	pub sections: Vec<AuditSection>
}

/// A custom section in the audit
#[derive(Clone)]
pub struct AuditSection {
	pub name: String,
	pub color: DynColors,
	pub entries: Vec<AuditSectionEntry>,
}

impl AuditSection {
	pub fn new(name: impl ToString, entries: Vec<AuditSectionEntry>) -> AuditSection {
		AuditSection {
			name: name.to_string(),
			color: DynColors::Ansi(AnsiColors::Magenta),
			entries
		}
	}
}

/// An entry in an AuditSection.
///
/// The entry consists of 3 parts.
/// ```md
/// prefix_left | prefix_right      text
/// ```
#[derive(Clone)]
pub struct AuditSectionEntry {
	pub prefix_left: Option<String>,
	pub prefix_right: Option<String>,
	pub text: String,
}

impl AuditSectionEntry {
	pub fn text(text: String) -> AuditSectionEntry {
		AuditSectionEntry {
			prefix_left: None,
			prefix_right: None,
			text
		}
	}
}