//! `DashboardHelpers::GetCpuUsage`

use super::MetricType::Enum as MetricType;

/// Current process CPU usage percentage (stub: returns 25.0 until real
/// platform metrics are wired).
pub fn Fn() -> Result<f64, String> { Ok(25.0) }
