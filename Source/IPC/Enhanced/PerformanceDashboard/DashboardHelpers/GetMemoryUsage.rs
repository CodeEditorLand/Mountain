//! `DashboardHelpers::GetMemoryUsage`

use super::MetricType::Enum as MetricType;

/// Current process memory usage in MB (stub: returns 100.0 until real
/// platform metrics are wired).
pub fn Fn() -> Result<f64, String> { Ok(100.0) }
