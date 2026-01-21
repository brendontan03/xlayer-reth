use clap::Args;

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct FullLinkMonitorArgs {
    /// Enable full link monitor functionality
    #[arg(
        long = "xlayer.full-link-monitor",
        help = "Enable full link monitor functionality (disabled by default)",
        default_value = "false"
    )]
    pub enable: bool,
    // TODO: add more full link monitor configuration here
}

impl FullLinkMonitorArgs {
    /// Validate full link monitor configuration
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
