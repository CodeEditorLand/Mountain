//! `DashboardHelpers::GenerateSpanId`

use super::MetricType::Enum as MetricType;

/// Generate a new UUID v4 string for use as a span identifier.
pub fn Fn() -> String { uuid::Uuid::new_v4().to_string() }
