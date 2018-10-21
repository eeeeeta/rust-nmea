//! Replacements for chrono types.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NaiveDate {
    pub year: i32,
    pub month: u32,
    pub day: u32
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NaiveTime {
    pub hour: u32,
    pub min: u32,
    pub sec: f64
}
