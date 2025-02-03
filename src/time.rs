use std::fmt;

use anyhow::Context;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Time {
    pub add_seconds: u128,
    pub milliseconds_left: u128,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let minutes = self.milliseconds_left / 60_000;
        let milliseconds_left = self.milliseconds_left % 60_000;
        let seconds = milliseconds_left / 1_000;
        let seconds_10th = (milliseconds_left % 1_000) / 100;

        write!(
            f,
            "{minutes}m {seconds}.{seconds_10th}s / {}s",
            self.add_seconds
        )
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct TimeSettings(pub Option<Time>);

impl fmt::Debug for TimeSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(time) = &self.0 {
            write!(f, "fischer {} {}", time.milliseconds_left, time.add_seconds)
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
                .parse::<u128>()
                .context("time_settings: arg 2 is not an integer")?;

            let arg_3 = args[3]
                .parse::<u128>()
                .context("time_settings: arg 3 is not an integer")?;

            Ok(Self(Some(Time {
                add_seconds: arg_3,
                milliseconds_left: arg_2,
            })))
        } else {
            Err(anyhow::Error::msg(err_msg))
        }
    }
}
