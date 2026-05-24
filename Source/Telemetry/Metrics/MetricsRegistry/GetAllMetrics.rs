//! `MetricsRegistry::GetAllMetrics`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use parking_lot::RwLock;
use crate::Telemetry::Metrics::{Metric, MetricValue};

pub fn Fn(This:&Struct) -> Vec<Metric::Struct> { This.Metrics.read().clone() }
