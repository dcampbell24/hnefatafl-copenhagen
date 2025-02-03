use std::{fmt, time::Duration};

use anyhow::Context;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Time {
    pub add_time: Duration,
    pub time_left: Duration,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut seconds = self.time_left.as_secs();
        let minutes = seconds / 60;
        seconds %= 60;

        write!(f, "{minutes}m {seconds}s / {}s", self.add_time.as_secs())
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct TimeSettings(pub Option<Time>);

impl fmt::Debug for TimeSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(time) = &self.0 {
            let time_left = time.time_left.as_secs();
            let add_seconds = time.add_time.as_secs();

            write!(f, "fischer {time_left} {add_seconds}")
        } else {
            write!(f, "un-timed _ _")
        }
    }
}

impl fmt::Display for TimeSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(time) = &self.0 {
            write!(f, "{time}")
        } else {
            write!(f, "-")
        }
    }
}

impl From<TimeSettings> for bool {
    fn from(time_settings: TimeSettings) -> Self {
        time_settings.0.is_some()
    }
}

impl TryFrom<Vec<&str>> for TimeSettings {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> anyhow::Result<Self> {
        let err_msg = "expected: time_settings ('un-timed' | 'fischer') MINUTES ADD_SECONDS";

        if Some("un-timed").as_ref() == args.get(1) {
            return Ok(Self(None));
        }

        if args.len() < 4 {
            return Err(anyhow::Error::msg(err_msg));
        }

        if "fischer" == args[1] {
            let arg_2 = args[2]
                .parse::<u64>()
                .context("time_settings: arg 2 is not an integer")?;

            let arg_3 = args[3]
                .parse::<u64>()
                .context("time_settings: arg 3 is not an integer")?;

            Ok(Self(Some(Time {
                add_time: Duration::from_secs(arg_3),
                time_left: Duration::from_secs(arg_2),
            })))
        } else {
            Err(anyhow::Error::msg(err_msg))
        }
    }
}
