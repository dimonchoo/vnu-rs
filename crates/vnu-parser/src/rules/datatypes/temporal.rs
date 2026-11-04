// Leap year check
fn is_leap_year(year: i32) -> bool {
    (year % 400 == 0) || (year % 4 == 0 && year % 100 != 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

// Date: YYYY-MM-DD
pub fn is_valid_date(val: &str) -> bool {
    if val.len() < 10 || val.len() > 32 {
        return false;
    }
    let mut parts = val.splitn(4, '-');
    let y_str = parts.next().unwrap_or("");
    let m_str = parts.next().unwrap_or("");
    let d_str = parts.next().unwrap_or("");
    if parts.next().is_some() || y_str.len() < 4 || m_str.len() != 2 || d_str.len() != 2 {
        return false;
    }
    let year: i32 = match y_str.parse() {
        Ok(y) if y >= 1 => y,
        _ => return false,
    };
    let month: u32 = match m_str.parse() {
        Ok(m) if (1..=12).contains(&m) => m,
        _ => return false,
    };
    let day: u32 = match d_str.parse() {
        Ok(d) if d >= 1 && d <= days_in_month(year, month) => d,
        _ => return false,
    };
    let _ = day;
    true
}

// Month: YYYY-MM
pub fn is_valid_month(val: &str) -> bool {
    if val.len() < 7 || val.len() > 32 {
        return false;
    }
    let mut parts = val.splitn(3, '-');
    let y_str = parts.next().unwrap_or("");
    let m_str = parts.next().unwrap_or("");
    if parts.next().is_some() || y_str.len() < 4 || m_str.len() != 2 {
        return false;
    }
    match (y_str.parse::<i32>(), m_str.parse::<u32>()) {
        (Ok(y), Ok(m)) => y >= 1 && (1..=12).contains(&m),
        _ => false,
    }
}

// Week: YYYY-Www (1..=53)
pub fn is_valid_week(val: &str) -> bool {
    if val.len() < 8 || val.len() > 32 {
        return false;
    }
    let (y_str, w_str) = match val.split_once("-W") {
        Some(pair) => pair,
        None => return false,
    };
    if y_str.len() < 4 || w_str.len() != 2 {
        return false;
    }
    match (y_str.parse::<i32>(), w_str.parse::<u32>()) {
        (Ok(y), Ok(w)) => y >= 1 && (1..=53).contains(&w),
        _ => false,
    }
}

// Time: HH:MM, HH:MM:SS, HH:MM:SS.sss
pub fn is_valid_time(val: &str) -> bool {
    if val.len() < 5 || val.len() > 32 {
        return false;
    }
    let mut parts = val.splitn(4, ':');
    let h_str = parts.next().unwrap_or("");
    let m_str = parts.next().unwrap_or("");
    let sec_part = parts.next();
    if parts.next().is_some() || h_str.len() != 2 || m_str.len() != 2 {
        return false;
    }
    let h: u32 = match h_str.parse() {
        Ok(v) if v <= 23 => v,
        _ => return false,
    };
    let m: u32 = match m_str.parse() {
        Ok(v) if v <= 59 => v,
        _ => return false,
    };
    let _ = (h, m);
    if let Some(sec) = sec_part {
        let sec_digits = sec.split('.').next().unwrap_or("");
        if sec_digits.len() != 2 {
            return false;
        }
        match sec_digits.parse::<u32>() {
            Ok(s) if s <= 59 => {}
            _ => return false,
        }
    }
    true
}

// Datetime local: Date + 'T' or ' ' + Time
pub fn is_valid_datetime_local(val: &str) -> bool {
    if val.len() < 16 || val.len() > 64 {
        return false;
    }
    let (date_part, time_part) = if let Some(pair) = val.split_once('T') {
        pair
    } else if let Some(pair) = val.split_once(' ') {
        pair
    } else {
        return false;
    };
    is_valid_date(date_part) && is_valid_time(time_part)
}

fn is_valid_timezone_offset(tz: &str) -> bool {
    let parts: Vec<&str> = tz.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return parts[0].len() == 2 && parts[1].len() == 2 && h <= 23 && m <= 59;
        }
    }
    false
}

// Time element datetime attribute: accepts date, time, datetime, duration, etc.
pub fn is_valid_time_datetime(val: &str) -> bool {
    let s = val.trim();
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    if is_valid_date(s)
        || is_valid_time(s)
        || is_valid_month(s)
        || is_valid_week(s)
        || is_valid_datetime_local(s)
    {
        return true;
    }
    if s.ends_with('Z') || s.ends_with('z') {
        let base = &s[..s.len() - 1];
        if is_valid_datetime_local(base) {
            return true;
        }
    }
    if let Some((base, tz)) = s.rsplit_once('+') {
        if is_valid_timezone_offset(tz) && is_valid_datetime_local(base) {
            return true;
        }
    }
    if let Some((base, tz)) = s.rsplit_once('-') {
        if base.len() >= 10 && is_valid_timezone_offset(tz) && is_valid_datetime_local(base) {
            return true;
        }
    }
    if s.starts_with('P') || s.starts_with('p') {
        return true;
    }
    false
}

// Datetime attribute on <del> and <ins>: Date (YYYY-MM-DD) or Global Datetime (datetime-tz)
pub fn is_valid_del_ins_datetime(s: &str) -> bool {
    if s.len() < 10 || s.len() > 64 || s.trim() != s {
        return false;
    }
    let b = s.as_bytes();
    if b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    if !b[0..4].iter().all(u8::is_ascii_digit)
        || !b[5..7].iter().all(u8::is_ascii_digit)
        || !b[8..10].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let year: i32 = match s[0..4].parse() {
        Ok(y) => y,
        Err(_) => return false,
    };
    let month: u32 = match s[5..7].parse() {
        Ok(m) => m,
        Err(_) => return false,
    };
    let day: u32 = match s[8..10].parse() {
        Ok(d) => d,
        Err(_) => return false,
    };
    if !(1000..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day < 1
        || day > days_in_month(year, month)
    {
        return false;
    }
    if s.len() == 10 {
        return true;
    }
    if b[10] != b'T' && b[10] != b' ' {
        return false;
    }
    let rest = &s[11..];
    if rest.len() < 6 {
        return false;
    }
    let rb = rest.as_bytes();
    if !rb[0..2].iter().all(u8::is_ascii_digit)
        || rb[2] != b':'
        || !rb[3..5].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    let h: u32 = match rest[0..2].parse() {
        Ok(v) if v <= 23 => v,
        _ => return false,
    };
    let m: u32 = match rest[3..5].parse() {
        Ok(v) if v <= 59 => v,
        _ => return false,
    };
    let _ = (h, m);
    let mut tail = &rest[5..];
    if tail.starts_with(':') {
        if tail.len() < 3 || !tail.as_bytes()[1..3].iter().all(u8::is_ascii_digit) {
            return false;
        }
        let sec: u32 = match tail[1..3].parse() {
            Ok(v) if v <= 59 => v,
            _ => return false,
        };
        let _ = sec;
        tail = &tail[3..];
        if tail.starts_with('.') {
            tail = &tail[1..];
            let frac_len = tail.bytes().take_while(u8::is_ascii_digit).count();
            if !(1..=3).contains(&frac_len) {
                return false;
            }
            tail = &tail[frac_len..];
        }
    }
    if tail == "Z" {
        return true;
    }
    if tail.starts_with('+') || tail.starts_with('-') {
        let neg = tail.starts_with('-');
        let tz = &tail[1..];
        let (tz_h, tz_m) = if tz.len() == 5 && tz.as_bytes()[2] == b':' {
            if !tz.as_bytes()[0..2].iter().all(u8::is_ascii_digit)
                || !tz.as_bytes()[3..5].iter().all(u8::is_ascii_digit)
            {
                return false;
            }
            (
                tz[0..2].parse::<u32>().unwrap_or(99),
                tz[3..5].parse::<u32>().unwrap_or(99),
            )
        } else if tz.len() == 4 && tz.bytes().all(|c| c.is_ascii_digit()) {
            (
                tz[0..2].parse::<u32>().unwrap_or(99),
                tz[2..4].parse::<u32>().unwrap_or(99),
            )
        } else {
            return false;
        };

        if neg && tz_h == 0 && tz_m == 0 {
            return false;
        }
        if tz_m > 59 {
            return false;
        }
        if neg && tz_h > 12 {
            return false;
        }
        if !neg && tz_h > 14 {
            return false;
        }
        if neg && tz_h == 12 && tz_m > 0 {
            return false;
        }
        if !neg && tz_h == 14 && tz_m > 0 {
            return false;
        }
        if tz_m != 0 && tz_m != 30 && tz_m != 45 {
            return false;
        }
        return true;
    }
    false
}
