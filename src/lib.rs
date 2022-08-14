#![feature(trait_alias)]
//! # Anyways
//! Anyways is a dynamic error reporting library.
//! Which allows you to not worry about creating error types and instead handling errors.
//!
//! This library is not recommended for other libraries to use and instead it is
//! advised to use something like `thiserror` to easily generate concrete types to make it easier for other people to use the library.
//! Instead this is intended for applications where a ton of libraries are used to create a product and where making a concrete error type is not feasible.
//!
//! ## Panic Processes
//! 1. Audit gets made
//! 2. The AuditProcessor removes useless information and makes the information more digestible
//! 3. The AuditFormatter formats the audit sections to the output.
use owo_colors::{Style};
use crate::audit::Audit;
use crate::formatter::{AnywaysAuditFormatter, AuditFormatter};
use crate::processor::{AnywaysAuditProcessorBuilder, AuditProcessor};

pub mod audit;
pub mod ext;
pub mod formatter;
pub mod processor;
mod align;

pub type Result<T, E = Audit> = std::result::Result<T, E>;

static mut AUDIT_FORMATTER: Option<Box<dyn AuditFormatter>> = None;
static mut AUDIT_PROCESSOR: Option<Box<dyn AuditProcessor>> = None;

pub fn set_audit_formatter(formatter: impl AuditFormatter + 'static) {
    unsafe {
        AUDIT_FORMATTER = Some(Box::new(formatter));
    }
}

pub fn set_audit_processor(processor: impl AuditProcessor + 'static) {
    unsafe {
        AUDIT_PROCESSOR = Some(Box::new(processor));
    }
}

pub fn get_audit_formatter() -> &'static dyn AuditFormatter {
    unsafe {
        if AUDIT_FORMATTER.is_none() {
            set_audit_formatter(AnywaysAuditFormatter::default());
        }
        AUDIT_FORMATTER.as_deref().unwrap()
    }
}

pub fn get_audit_processor() -> &'static dyn AuditProcessor {
    unsafe {
        if AUDIT_PROCESSOR.is_none() {
            set_audit_processor(AnywaysAuditProcessorBuilder::default().build());
        }
        AUDIT_PROCESSOR.as_deref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use owo_colors::{AnsiColors, DynColors};
    use crate::audit::{AuditSection, AuditSectionEntry};

    use crate::ext::AuditExt;
    use crate::Result;

    #[test]
    fn thigns() -> Result<()> {
        read_plugin_before().wrap_err("Failed to read plugin").wrap(|audit| {
            audit.custom_sections.push(AuditSection {
                name: "Dogs".to_string(),
                color: DynColors::Ansi(AnsiColors::BrightBlue),
                entries: vec![
                    AuditSectionEntry::text("Sheril".to_string())
                ]
            })
        })
    }

    fn read_plugin_before() -> Result<()> {
        match very_long_module_also_because_i_can_btw_i_need_this_to_see_if_wrapping_works_correctly::read_plugin_very_long_name_because_i_can_hello_there() {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        };
        Ok(())
    }

    mod very_long_module_also_because_i_can_btw_i_need_this_to_see_if_wrapping_works_correctly {
        use std::fs::File;
        use crate::ext::AuditExt;

        pub(crate) fn read_plugin_very_long_name_because_i_can_hello_there() -> crate::Result<()> {
            File::open("./your mom is very gay").wrap_err("Failed to find your mom being gay")?;

            Ok(())
        }
    }
}