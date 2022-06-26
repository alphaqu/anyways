use std::path::PathBuf;
use crate::default::entry::HowlerEntry;

pub struct HowlerFile {
     pub(crate) file: Option<String>,
     pub(crate) entries: Vec<HowlerEntry>,
}

impl HowlerFile {
    pub fn new(file: Option<PathBuf>, entries: Vec<HowlerEntry>) -> HowlerFile {
        HowlerFile {
            file: match file {
                None => None,
                Some(value) => value.to_str().map(|name| name.to_string()),
            },
            entries,
        }
    }
}
