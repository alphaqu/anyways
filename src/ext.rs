use std::error::Error;
use crate::audit::{Audit, AuditSection};

pub trait AuditExt<T>: Sized {
	fn wrap_err<E: Into<Box<dyn Error + 'static>>>(self, err: E) -> Result<T, Audit> {
		self.wrap(|audit| { audit.push_err(err); })
	}

	fn wrap_err_with<E: Into<Box<dyn Error + 'static>>>(self, err: impl FnOnce() -> E) -> Result<T, Audit> {
		self.wrap(|audit| { audit.push_err(err()); })
	}

	fn wrap_section(self, section: AuditSection) -> Result<T, Audit> {
		self.wrap(|audit| { audit.push_section(section); })
	}

	fn wrap_section_with(self, section: impl FnOnce() -> AuditSection) -> Result<T, Audit> {
		self.wrap(|audit| { audit.push_section(section()); })
	}

	fn wrap(self, func: impl FnOnce(&mut Audit)) -> Result<T, Audit>;
}

impl<T, E: Into<Box<dyn Error + 'static>>> AuditExt<T> for Result<T, E> {
	fn wrap(self, func: impl FnOnce(&mut Audit)) -> Result<T, Audit> {
		match self {
			Ok(value) => Ok(value),
			Err(audit) => {
				let mut audit = Audit::new(audit);
				func(&mut audit);
				Err(audit)
			}
		}
	}
}

impl<T> AuditExt<T> for Result<T, Audit> {
	fn wrap(self, func: impl FnOnce(&mut Audit)) -> Result<T, Audit> {
		match self {
			Ok(value) => Ok(value),
			Err(mut audit) => {
				func(&mut audit);
				Err(audit)
			}
		}
	}
}