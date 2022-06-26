#![feature(backtrace)]
#![feature(backtrace_frames)]
#![feature(error_iter)]

pub mod handler;
mod default;
mod util;
//mod wrap;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use backtrace::{Backtrace};
use std::rc::Rc;

#[derive(Clone)]
pub struct Report {
	pub backtrace: Backtrace,
	pub sections: Vec<Section>,
	pub errors: Vec<Rc<Box<dyn Error + 'static + Send + Sync>>>
}

impl Report {
	pub fn new<E: Into<Box<dyn Error + 'static + Send + Sync>>>(err: E) -> Report {
		Report {
			backtrace: Backtrace::new_unresolved(),
			sections: vec![],
			errors: vec![Rc::new(err.into())]
		}
	}

	pub fn push_err<E: Into<Box<dyn Error + 'static + Send + Sync>>>(&mut self, err: E) {
		self.errors.push(Rc::new(err.into()));
	}
}

impl<I: Into<Box<dyn Error + 'static + Send + Sync>>> From<I> for Report {
	fn from(err: I) -> Self {
		Report::new(err)
	}
}

impl Debug for Report {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		handler::fmt(f, self)
	}
}

impl Display for Report {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		handler::fmt(f, self)
	}
}

#[derive(Clone)]
pub struct Section {
	pub name: String,
	pub entries: Vec<SectionEntry>
}

#[derive(Clone)]
pub struct SectionEntry {
	pub prefix: String,
	pub suffix: String,
	pub value: String,
}

#[cfg(test)]
mod tests {
	use std::fs;
	use std::ops::Deref;
	use crate::Report;
	use crate::util::ReportExt;

	#[test]
	fn it_works() -> Result<(), Report> {
		testing()
	}

	fn testing() -> Result<(), Report> {
		haha_testing()
	}


	fn haha_testing() -> Result<(), Report>{
		another_not_swear_word_mod::testing_in_another_one()
	}

	mod another_not_swear_word_mod {
		use owo_colors::OwoColorize;
		use crate::{Report, Section, SectionEntry};
		use crate::tests::Stuff;
		use crate::util::ReportExt;

		pub(crate) fn testing_in_another_one() -> Result<(), Report> {
			let mut section = Section {
				name: "Hasfjasdlkfjslda".green().to_string(),
				entries: vec![
					SectionEntry {
						prefix: "prefix".to_string(),
						suffix: "suffix".to_string(),
						value: "value that should prob tell you stuff".to_string()
					},
					SectionEntry {
						prefix: "ugh".to_string(),
						suffix: "agh".to_string(),
						value: "value that should prob tell you less important stuff".to_string()
					},
					SectionEntry {
						prefix: "ugh".to_string(),
						suffix: "agh".to_string(),
						value: format!("look we got stuff {}", "color".cyan().italic().bold().underline().strikethrough())
					}
				],
			};

			let mut result = haha_testing_super_funny_once_again().wrap_err_with(|| "I just had meatballs, they were pretty great.");
			if let Err(error) = &mut result  {
				error.sections.push(section);
			}
			result
		}


		pub(crate) fn haha_testing_super_funny_once_again() -> Result<(), Report>{

			let closure = || {
				let inner_closure = || {
					Err(Report::new("hi"))
				};

				let stuff = Stuff {
					inner: Box::new(inner_closure)
				};
				stuff()?;

				Ok(())
			};

			closure_invoker(closure).wrap_err("very funny testing exists here")
		}

		pub(crate) fn closure_invoker(value: impl FnOnce() -> Result<(), Report>) -> Result<(), Report> {
			value().wrap_err("well we just invoked a closure")?;

			Ok(())
		}
	}

	pub struct Stuff {
		inner: Box<dyn Fn() -> Result<(), Report>>
	}

	impl Deref for Stuff {
		type Target = dyn Fn() -> Result<(), Report>;

		fn deref(&self) -> &Self::Target {
			&|| {
				//let result: Result<(), &str> = Err("fdas");
				//result.unwrap();

				fs::write("./dir/that/does/not/exist", "fadfa").wrap_err("hello")
			}
		}
	}
}
