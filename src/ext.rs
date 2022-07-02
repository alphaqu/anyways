use crate::audit::{Audit, AuditError, AuditSection};
use backtrace::Frame;

pub trait AuditExt<T>: Sized {
    fn wrap_err<E: Into<AuditError>>(self, err: E) -> crate::Result<T> {
	    self.wrap(|audit| {
            let mut e = err.into();
            e.location = get_caller(3);
            audit.push_err(e);
        })
    }

    fn wrap_err_with<E: Into<AuditError>>(self, err: impl FnOnce() -> E) -> crate::Result<T> {
        self.wrap(|audit| {
            let mut e = err().into();
            e.location = get_caller(3);
            audit.push_err(e);
        })
    }

    fn wrap_section(self, section: AuditSection) -> crate::Result<T> {
        self.wrap(|audit| {
            audit.push_section(section);
        })
    }

    fn wrap_section_with(self, section: impl FnOnce() -> AuditSection) -> crate::Result<T> {
        self.wrap(|audit| {
            audit.push_section(section());
        })
    }

    fn wrap(self, func: impl FnOnce(&mut Audit)) -> crate::Result<T>;
}

impl<T, E: Into<AuditError>> AuditExt<T> for Result<T, E> {
    fn wrap(self, func: impl FnOnce(&mut Audit)) -> crate::Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(audit) => {
                let mut error = audit.into();
                error.location = get_caller(2);
                let mut audit = Audit::new(error);
                func(&mut audit);
                Err(audit)
            }
        }
    }
}

impl<T> AuditExt<T> for Result<T, Audit> {
    fn wrap(self, func: impl FnOnce(&mut Audit)) -> crate::Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(mut audit) => {
                func(&mut audit);
                Err(audit)
            }
        }
    }
}

impl<T> AuditExt<T> for Option<T> {
    fn wrap(self, func: impl FnOnce(&mut Audit)) -> crate::Result<T> {
        match self {
            Some(value) => Ok(value),
            None => {
                let mut audit = Audit::new_empty();
                func(&mut audit);
                Err(audit)
            }
        }
    }
}

pub(crate) fn get_caller(extra_skips: i32) -> Option<Frame> {
    let mut caller = None;
    let mut remaining = 4 + extra_skips;
    backtrace::trace(|frame| {
        remaining -= 1;
        if remaining == 0 {
            caller = Some(frame.clone());
        }
        remaining > 0
    });

    caller
}