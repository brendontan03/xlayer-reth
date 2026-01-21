use clap::Args;

use xlayer_monitor::FullLinkMonitorArgs;

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct EngineApiArgs {
    /// Full link monitoring trace configurations
    #[command(flatten)]
    pub monitor: FullLinkMonitorArgs,
}
