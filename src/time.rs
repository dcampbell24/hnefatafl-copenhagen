use std::time::{Duration, Instant};

use regex::Regex;


#[derive(Clone, Debug)]
struct Time {
    add: Duration,
    duration: Duration,
    start: Instant,
}

#[derive(Clone, Debug)]
struct TimeSettings {
    time_settings: Option<Time>,
}

impl TryFrom<Vec<&str>> for TimeSettings {
    type Error = anyhow::Error;

    fn try_from(message: &str) -> Result<Self, Self::Error> {
        // none | HH:MM:SS (Hh | Mm | Ss)
        let re = Regex::new(r"(?<H>[0-9]+):(?<M>[0-9]+):(?<S>[0-9]+) (?<add_time>[0-9]+s)").unwrap();
        if let Some(caps) = re.captures(message) {
            let mut duration = caps["H"].parse::<u64>()? * 60 * 60;
            duration += caps["M"].parse::<u64>()? * 60;
            duration +=  caps["S"].parse::<u64>()?;
            let duration = Duration::from_secs(duration);

            let add = Duration::from_secs(caps["add_time"].parse::<u64>()?);
            let start = Instant::now();

            TimeSettings {
                time_settings: Some(Time {
                    add, duration, start,
                })
            }
        } else {
            TimeSettings { time_settings: None }
        }
    }
}
