use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

/// Represents a universally unique identifier (UUID).
pub type ID = Uuid;
/// Represents a date and time with a fixed offset from UTC.
pub type DateWithTimeZone = DateTime<FixedOffset>;
/// Represents a timestamp in milliseconds since the Unix epoch (1970-01-01T00:00:00Z).
pub type Timestamp = i64;