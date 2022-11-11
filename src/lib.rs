use std::io;
use std::fmt::Display;

use rppal::i2c::{self, I2c};

/* The following constants are all derived from the datasheet of the DS3231 */
const I2C_ADDRESS: u16 = 0b1101000;

const NUM_CLOCK_AND_CALENDAR_REGS: usize = 7;
const REG_SECONDS: usize = 0x00;
const REG_MINUTES: usize = 0x01;
const REG_HOURS: usize = 0x02;
const REG_DAY: usize = 0x03;
const REG_DATE: usize = 0x04;
const REG_MONTH_CENTURY: usize = 0x05;
const REG_YEAR: usize = 0x06;

const HOURS_MASK: u8 = 0x3F;
const MONTH_MASK: u8 = 0x1F;
const CLOCK_TOGGLE_BIT: u8 = 6;
const CENTURY_BIT: u8 = 7;

const NUM_TEMP_REGS: usize = 2;
const REG_TEMPS: usize = 0x11;

const TEMP_LSB_BIT: usize = 6;

#[derive(Debug)]
enum Day {
    Mon = 1,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun
}

impl Day {
    fn from_u8(day: u8) -> Day {
        match day {
            1 => Day::Mon,
            2 => Day::Tue,
            3 => Day::Wed,
            4 => Day::Thu,
            5 => Day::Fri,
            6 => Day::Sat,
            7 => Day::Sun,
            _ => panic!("Invalid day number: {}. Should be between 1 and 7", day),
        }
    }
}

#[derive(Debug)]
enum Month {
    Jan = 1,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    fn from_u8(month: u8) -> Month {
        match month {
            1 => Month::Jan,
            2 => Month::Feb,
            3 => Month::Mar,
            4 => Month::Apr,
            5 => Month::May,
            6 => Month::Jun,
            7 => Month::Jul,
            8 => Month::Aug,
            9 => Month::Sep,
            10 => Month::Oct,
            11 => Month::Nov,
            12 => Month::Dec,
            _ => panic!("Invalid month number: {}. Should be between 1 and 12", month),
        }
    }
}

#[derive(Debug, Default)]
pub struct RTCDate {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub day: u8,
    pub date: u8,
    pub month: u8,
    pub year: u32,
}

impl Display for RTCDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timezone = "CET";
        write!(f, "{:?} {} {:?} {}:{}:{} {} {}", Day::from_u8(self.day), self.date, Month::from_u8(self.month), self.hours, self.minutes, self.seconds, timezone, self.year)
    }
}

#[derive(Debug)]
pub struct RTC {
    i2c: I2c,
    start_year: u32,
}

impl RTC {
    pub fn new(start_year: u32) -> i2c::Result<RTC> {
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(I2C_ADDRESS)?;

        Ok(RTC {
            i2c: i2c,
            start_year: start_year,
        })
    }

    pub fn fetch_date(&self) -> io::Result<RTCDate> {
        let mut read_buffer = [0u8; NUM_CLOCK_AND_CALENDAR_REGS];

        match self.i2c.block_read(REG_SECONDS as u8, &mut read_buffer) {
            Ok(()) => {
                let mut rtc_date = RTCDate::default();

                rtc_date.seconds = bcd_to_dec(read_buffer[REG_SECONDS]);
                rtc_date.minutes = bcd_to_dec(read_buffer[REG_MINUTES]);
                rtc_date.hours = bcd_to_dec(read_buffer[REG_HOURS] & HOURS_MASK);
                rtc_date.day = bcd_to_dec(read_buffer[REG_DAY]);
                rtc_date.date = bcd_to_dec(read_buffer[REG_DATE]);
                rtc_date.month = bcd_to_dec(read_buffer[REG_MONTH_CENTURY] & MONTH_MASK);
                rtc_date.year =
                    calculate_normal_years(read_buffer[REG_YEAR], read_buffer[REG_MONTH_CENTURY])
                        as u32
                        + self.start_year;

                Ok(rtc_date)
            }
            Err(i2c_error) => Err(io::Error::new(io::ErrorKind::Other, i2c_error)),
        }
    }

    pub fn set_date(&mut self, rtc_date: &RTCDate) -> io::Result<()> {
        let mut write_buffer = [0u8; NUM_CLOCK_AND_CALENDAR_REGS];

        /* Check date validity */
        if rtc_date.seconds > 59 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seconds must be between 0 and 59",
            ));
        } else if rtc_date.minutes > 59 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "minutes must be between 0 and 59",
            ));
        } else if rtc_date.hours > 23 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "hours must be between 0 and 23",
            ));
        } else if rtc_date.day < 1 || rtc_date.day > 7 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "day must be between 1 and 7",
            ));
        } else if rtc_date.date < 1 || rtc_date.date > 31 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "date must be between 1 and 31",
            ));
        } else if rtc_date.month < 1 || rtc_date.month > 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "month must be between 1 and 12",
            ));
        } else if rtc_date.year < self.start_year || rtc_date.year - self.start_year > 199 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "year must be between {} and {}",
                    self.start_year,
                    self.start_year + 199
                ),
            ));
        }

        /* Fill buffer with RTC date data */
        write_buffer[REG_SECONDS] = dec_to_bcd(rtc_date.seconds);
        write_buffer[REG_MINUTES] = dec_to_bcd(rtc_date.minutes);
        write_buffer[REG_HOURS] = calculate_reg_hours(rtc_date.hours);
        write_buffer[REG_DAY] = dec_to_bcd(rtc_date.day);
        write_buffer[REG_DATE] = dec_to_bcd(rtc_date.date);
        write_buffer[REG_MONTH_CENTURY] =
            calculate_reg_month_century(rtc_date.month, rtc_date.year, self.start_year);
        write_buffer[REG_YEAR] = calculate_reg_year(rtc_date.year, self.start_year);

        /* Write date to rtc */
        match self.i2c.block_write(REG_SECONDS as u8, &write_buffer) {
            Ok(()) => Ok(()),
            Err(i2c_error) => Err(io::Error::new(io::ErrorKind::Other, i2c_error)),
        }
    }

    pub fn fetch_temperature(&self) -> io::Result<f32> {
        let mut read_buffer = [0u8; NUM_TEMP_REGS];

        match self.i2c.block_read(REG_TEMPS as u8, &mut read_buffer) {
            Ok(()) => {
                let integer = read_buffer[0] as i8;
                let decimal = (read_buffer[1] >> TEMP_LSB_BIT) as i8;
                let temperature = match decimal {
                    0 => (integer as f32) + 0.0,
                    1 => (integer as f32) + 0.25,
                    2 => (integer as f32) + 0.5,
                    3 => (integer as f32) + 0.75,
                    _ => integer as f32,
                };

                Ok(temperature)
            },
            Err(i2c_error) => Err(io::Error::new(io::ErrorKind::Other, i2c_error)),
        }
    }
}

/* Conversion functions between normal values and register values */
fn calculate_normal_years(reg_years: u8, reg_month_century: u8) -> u8 {
    if reg_month_century & (1 << CENTURY_BIT) != 0 {
        bcd_to_dec(reg_years) + 100
    } else {
        bcd_to_dec(reg_years)
    }
}

fn calculate_reg_hours(normal_hours: u8) -> u8 {
    (1 << CLOCK_TOGGLE_BIT) | dec_to_bcd(normal_hours)
}

fn calculate_reg_month_century(normal_month: u8, normal_year: u32, start_year: u32) -> u8 {
    let century = (normal_year - start_year > 100) as u8;
    (century << CENTURY_BIT) | dec_to_bcd(normal_month)
}

fn calculate_reg_year(normal_year: u32, start_year: u32) -> u8 {
    dec_to_bcd(((normal_year - start_year) % 100) as u8)
}

/* Time and calendar registers use the Binary Coded Decimal format on the
stored values. These are helper functions to convert to normal decimal
values. */
fn bcd_to_dec(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}

fn dec_to_bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}
