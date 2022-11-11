use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct RTCCli {
    #[clap(subcommand)]
    pub command_type: CommandType,
}

#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// Get RTC time
    Get,

    /// Set RTC time
    Set(SetCommand),

    /// Get temperature
    Temp,
}

#[derive(Debug, Args)]
pub struct SetCommand {
    /// Seconds [0 - 59]
    pub seconds: u8,

    /// Minutes [0 - 59]
    pub minutes: u8,

    /// Hours [0 - 23]
    pub hours: u8,

    /// Date [1 - 31]
    pub date: u8,

    /// Month [1 - 12]
    pub month: u8,

    /// Year
    pub year: u32,

    /// Day of week [1 - 7]
    pub day: u8,
}
