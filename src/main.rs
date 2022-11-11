use rtc_rs::{RTC, RTCDate};

use std::error::Error;

mod cli;
use cli::RTCCli;
use clap::Parser;

const DEFAULT_START_YEAR: u32 = 2000;

fn main() -> Result<(), Box<dyn Error>> {
    let mut rtc = RTC::new(DEFAULT_START_YEAR)?;

    let args = RTCCli::parse();
    match args.command_type {
        cli::CommandType::Get => {
            println!("{}", rtc.fetch_date()?);
        },
        cli::CommandType::Set(command) => {
            let rtc_date = RTCDate{seconds: command.seconds, minutes: command.minutes, hours: command.hours, day: command.day, date: command.date, month: command.month, year: command.year};
            rtc.set_date(&rtc_date)?;
        },
        cli::CommandType::Temp => {
            println!("Temperature: {} C", rtc.fetch_temperature()?);
        }
    }

    Ok(())
}
