//! Date and time handling.

#![no_std]

pub type Time = i64;

///Is the given year a leap year?
pub fn is_leap(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Timestamp from time components.
pub fn time2ts(hour: u32, minute: u32, second: u32) -> Time {
    (((hour * 60) + minute) * 60 + second) as Time
}

/// Timestamp from date components.
///
/// mday in range 1..=31
/// month in range 1..=12
pub fn date2ts(mday: u32, month: u32, year: u32) -> Time {
    let days_per_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let mut day_in_year = days_per_month[core::cmp::max(month as usize, 1) - 1] + mday as i32 - 1;
    if is_leap(year) && month < 3 {
        day_in_year -= 1
    };
    let year = year as Time;
    let days = day_in_year as Time + year * 365 + year / 4 - year / 100 + year / 400 - 719527;
    days * 24 * 60 * 60
}

/// Convert a DOS time-stamp to the UNIX format.
pub fn dos_time2ts(mtime: u16) -> Time {
    time2ts(
        (mtime as u32 >> 11) & 0x1f,
        (mtime as u32) >> 5 & 0x3f,
        (mtime as u32) << 1 & 0x3e,
    )
}
/// Convert a DOS date to the UNIX format.
pub fn dos_date2ts(mdate: u16) -> Time {
    date2ts(mdate as u32 & 0x1f, mdate as u32 >> 5 & 0xf, (mdate as u32 >> 9) + 1980)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DAYS_PER_MONTH_LEAP: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    #[test]
    fn test_zero() {
        assert_eq!(0, date2ts(1, 1, 1970));
    }

    #[test]
    fn test_two_ends() {
        assert_eq!(0, ((1 << 32) - date2ts(7, 2, 2106)) / 86400);
        assert_eq!(0, ((-1 << 32) - date2ts(24, 11, 1833)) / 86400);
    }

    #[test]
    fn test_range_forward() {
        let mut prev = -1;
        for year in 1970..2020 {
            for month in 1..=12 {
                for day in 1..=DAYS_PER_MONTH_LEAP[month - 1] {
                    if day == 29 && month == 2 && !is_leap(year) {
                        continue;
                    }
                    let delta = date2ts(day, month as u32, year) / 86400 - prev;
                    assert_eq!(delta, 1, "{}.{}.{}", day, month, year);
                    prev += delta;
                }
            }
        }
    }

    #[test]
    fn test_range_backward() {
        let mut prev = 10957;
        for year in (1850..2000).rev() {
            for month in (1..=12).rev() {
                for day in (1..=DAYS_PER_MONTH_LEAP[month - 1]).rev() {
                    if day == 29 && month == 2 && !is_leap(year) {
                        continue;
                    }
                    let delta = date2ts(day, month as u32, year) / 86400 - prev;
                    assert_eq!(delta, -1, "{}.{}.{}", day, month, year);
                    prev += delta;
                }
            }
        }
    }
}
