use loss72_platemaker_core::model::GenerationContext;

use crate::{build_tasks::TaskResult, config::Configuration};

pub fn full_build(config: &Configuration, ctx: &GenerationContext) -> TaskResult<()> {
    crate::build_tasks::run_all_build_steps(config, ctx)
}
