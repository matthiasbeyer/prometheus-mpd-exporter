use std::fmt::Debug;
use std::fmt::Display;
use std::convert::TryInto;

use anyhow::Result;
use anyhow::Error;
use num_traits::Num;
use prometheus_exporter_base::prelude::*;

pub trait IntoNumMetric {
    type Output: Num + Display + Debug;

    fn into_num_metric(self) -> Self::Output;
}

impl IntoNumMetric for std::time::Duration {
    type Output = u64;
    fn into_num_metric(self) -> Self::Output {
        self.as_secs()
    }
}

impl IntoNumMetric for u8 {
    type Output = u8;
    fn into_num_metric(self) -> Self::Output {
        self
    }
}

impl IntoNumMetric for u32 {
    type Output = u32;
    fn into_num_metric(self) -> Self::Output {
        self
    }
}

impl IntoNumMetric for i32 {
    type Output = i32;
    fn into_num_metric(self) -> Self::Output {
        self
    }
}


pub struct Metric<'a, T: IntoNumMetric> {
    name: &'static str,
    value: T,
    description: &'static str,
    instance: &'a str,
}

impl<'a, T: IntoNumMetric + Debug> Metric<'a, T> {
    pub fn new(name: &'static str, value: T, description: &'static str, instance: &'a str) -> Self {
        log::trace!("New metric: {} = {:?}", name, value);
        Metric { name, value, description, instance }
    }

    pub fn into_metric<'b>(self) -> Result<PrometheusMetric<'b>> {
        let instance = PrometheusInstance::new()
            .with_label("instance", self.instance)
            .with_value(self.value.into_num_metric())
            .with_current_timestamp()
            .map_err(Error::from)?;

        let mut m = PrometheusMetric::new(self.name, MetricType::Counter, self.description);
        m.render_and_append_instance(&instance);
        Ok(m)
    }
}

