use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

fn parse_cron_field(field: &str, min: u32, max: u32) -> Option<Vec<u32>> {
    if field == "*" {
        return Some((min..=max).collect());
    }
    let val: u32 = field.parse().ok()?;
    if val < min || val > max {
        return None;
    }
    Some(vec![val])
}

pub fn calculate_next_due(expression: &str, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let fields: Vec<&str> = expression.split_whitespace().collect();
    if fields.len() != 5 {
        return None;
    }

    let minutes = parse_cron_field(fields[0], 0, 59)?;
    let hours = parse_cron_field(fields[1], 0, 23)?;
    let dom = parse_cron_field(fields[2], 1, 31)?;
    let months = parse_cron_field(fields[3], 1, 12)?;
    let dow = parse_cron_field(fields[4], 0, 7)?; // 0=Sun, 7 also means Sun

    let mut dt = from + Duration::minutes(1);
    let max_iterations: u32 = 525600 * 2; // 2 years of minutes

    for _ in 0..max_iterations {
        let minute = dt.minute();
        let hour = dt.hour();
        let day = dt.day();
        let month = dt.month();
        let weekday = dt.weekday().num_days_from_sunday();

        if minutes.contains(&minute)
            && hours.contains(&hour)
            && dom.contains(&day)
            && months.contains(&month)
            && (dow.contains(&weekday) || (weekday == 0 && dow.contains(&7)))
        {
            return Some(dt);
        }
        dt += Duration::minutes(1);
    }

    None
}
