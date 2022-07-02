use crate::ext::get_caller;
use crate::{get_audit_formatter, get_audit_processor};
use backtrace::{Backtrace, Frame};
use owo_colors::{AnsiColors, DynColors, OwoColorize};
use std::error::Error;
use std::fmt::{Debug, Formatter, Write};
use std::ops::{Deref, DerefMut};

/// An Audit is the Error type of Anyways. It allows you to hold any type of error dynamically without worrying about it.
pub struct Audit {
    pub backtrace: Backtrace,
    pub errors: Vec<AuditError>,
    pub custom_sections: Vec<AuditSection>,
}

impl Audit {
    pub fn new_empty() -> Audit {
        Audit {
            backtrace: Backtrace::new_unresolved(),
            errors: vec![],
            custom_sections: vec![],
        }
    }

    pub fn new(err: impl Into<AuditError>) -> Audit {
        let mut audit = Audit::new_empty();
        audit.push_err(err);
        audit
    }

    pub fn downcast_mut<T: Error + 'static>(&mut self) -> Option<&mut T> {
        for err in &mut self.errors {
            if let Some(err) = err.downcast_mut::<T>() {
                return Some(err);
            }
        }

        None
    }


    /// Downcasts the audit to a concrete error type by checking the chain of errors and attempting to downcast them.
    /// If it cannot find any error that matches T it will return None, else it will return Some(value)
    pub fn downcast_ref<T: Error + 'static>(&self) -> Option<&T> {
        for err in &self.errors {
            if let Some(err) = err.downcast_ref::<T>() {
                return Some(err);
            }
        }

        None
    }

    /// Appends an error to the audit errors.
    ///
    /// # Arguments
    ///
    /// * `err`: The error that you want to append. If you input an AuditError you can also set its location for backtrace location information.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use anyways::ext::AuditExt;
    /// File::open("./hello").wrap(|audit| { audit.push_err("You blew stuff up"); });
    /// ```
    /// You can also append errors by just using [wrap_err](../ext.rs) which does the same thing.
    /// ```
    /// use std::fs::File;
    /// use anyways::ext::AuditExt;
    /// File::open("./hello").wrap_err("You blew stuff up");
    /// ```
    pub fn push_err(&mut self, err: impl Into<AuditError>) -> &mut Self {
        self.errors.insert(0, err.into());
        self
    }

    /// Appends a custom section to the Audit.
    ///
    /// # Arguments
    ///
    /// * `section`: The custom AuditSection you want to append.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use owo_colors::{AnsiColors, DynColors};
    /// use anyways::audit::AuditSection;
    /// use anyways::ext::AuditExt;
    ///
    /// File::open("./hello").wrap(|audit| {
    ///     audit.push_section(AuditSection {
    ///         name: "Section Name stuff".to_string(),
    ///         color: DynColors::Ansi(AnsiColors::BrightBlue),
    ///         entries: vec![],
    ///     });
    /// });
    /// ```
    ///
    /// You can also do the same thing by using [wrap_section](../ext.rs) which does the same thing.
    /// ```
    /// use std::fs::File;
    /// use owo_colors::{AnsiColors, DynColors};
    /// use anyways::audit::AuditSection;
    /// use anyways::ext::AuditExt;
    ///
    /// File::open("./hello").wrap_section_with(|| AuditSection {
    ///    name: "Section Name stuff".to_string(),
    ///    color: DynColors::Ansi(AnsiColors::BrightBlue),
    ///    entries: vec![],
    ///});
    /// ```
    pub fn push_section(&mut self, section: impl Into<AuditSection>) -> &mut Self {
        self.custom_sections.push(section.into());
        self
    }
}

impl<E: Into<AuditError>> From<E> for Audit {
    fn from(error: E) -> Self {
        let mut err: AuditError = error.into();
        err.location = get_caller(2);
        Audit::new(err)
    }
}

// this is actually the print error stuff
impl Debug for Audit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let audit = get_audit_processor().process(self);
        f.write_char('\n')?;
        get_audit_formatter().format(f, &audit)
    }
}

pub struct AuditError {
    pub error: Box<dyn Error + 'static>,
    pub location: Option<Frame>,
}

impl<E: Into<Box<dyn Error + 'static>>> From<E> for AuditError {
    fn from(err: E) -> Self {
        AuditError {
            error: err.into(),
            location: None,
        }
    }
}

impl Deref for AuditError {
    type Target = Box<dyn Error + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

impl DerefMut for AuditError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.error
    }
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
            entries,
        }
    }
}

/// An entry in an AuditSection.
///
/// The entry consists of 4 parts.
///
/// Here is an example of difference scenarios of its padding
/// ```md
/// ╭── Section ─────────────────────────────────────────────────────╮
/// │ e.prefix_left + e.prefix_right      e.text           e.suffix +│
/// │ e.prefix_left |                     e.text                     │
/// │               + e.prefix_right      e.text           e.suffix +│
/// │ e.text                                                         │
/// │            -> ^ <- separator                                   │
/// ╰────────────────────────────────────────────────────────────────╯
/// ```
#[derive(Clone)]
pub struct AuditSectionEntry {
    pub prefix: Option<String>,

    pub prefix_left: Option<String>,
    pub separator: String,
    pub prefix_right: Option<String>,
    pub text: String,
    pub suffix: Option<String>,
}

impl AuditSectionEntry {
    pub fn empty() -> AuditSectionEntry {
        AuditSectionEntry {
            prefix: None,
            prefix_left: None,
            separator: "|".to_string(),
            prefix_right: None,
            text: "".to_string(),
            suffix: None,
        }
    }
    pub fn text(text: String) -> AuditSectionEntry {
        AuditSectionEntry {
            prefix: None,
            prefix_left: None,
            separator: "|".to_string(),
            prefix_right: None,
            text,
            suffix: None,
        }
    }
}
