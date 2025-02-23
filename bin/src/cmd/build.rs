use crate::{build_tasks::TaskResult, config::Configuration};

pub fn full_build(config: &Configuration) -> TaskResult<()> {
    crate::build_tasks::full_build(config)
}
