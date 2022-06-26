use std::error::Error;
use crate::Report;

type ErrBox = Box<dyn Error + 'static + Send + Sync>;



pub trait ReportExt<T> {
	fn wrap_err<E: Into<ErrBox>>(self, err: E) -> Result<T, Report>;
	fn wrap_err_with<R: Into<ErrBox>, E: FnOnce() -> R>(self, err: E) -> Result<T, Report>;
}

impl<T, I: Into<ErrBox>> ReportExt<T> for Result<T, I> {
	fn wrap_err<E: Into<ErrBox>>(self, err: E) -> Result<T, Report> {
		match self {
			Ok(ok) => Ok(ok),
			Err(source) => {
				let mut report = Report::new(source);
				report.push_err(err);
				Err(report)
			}
		}
	}

	fn wrap_err_with<R: Into<ErrBox>, E: FnOnce() -> R>(self, err: E) -> Result<T, Report> {
		match self {
			Ok(ok) => Ok(ok),
			Err(source) => {
				let mut report = Report::new(source);
				report.push_err(err());
				Err(report)
			}
		}
	}
}

impl<T> ReportExt<T> for Result<T, Report> {
	fn wrap_err<E: Into<ErrBox>>(self, err: E) -> Result<T, Report> {
		match self {
			Ok(ok) => Ok(ok),
			Err(mut report) => {
				report.push_err(err);
				Err(report)
			}
		}
	}

	fn wrap_err_with<R: Into<ErrBox>, E: FnOnce() -> R>(self, err: E) -> Result<T, Report> {
		match self {
			Ok(ok) => Ok(ok),
			Err(mut report) => {
				report.push_err(err());
				Err(report)
			}
		}
	}
}