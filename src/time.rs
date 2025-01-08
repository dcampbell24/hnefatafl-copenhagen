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

        write!(f, "{minutes}m {seconds}s")
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub time_settings: Option<Time>,
}

impl TryFrom<Vec<&str>> for Settings {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> anyhow::Result<Self> {
        let err_msg = "expected: time_settings ('none' | 'fischer') MINUTES ADD_SECONDS";

        if Some("none").as_ref() == args.get(1) {
            return Ok(Settings {
                time_settings: None,
            });
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

            Ok(Settings {
                time_settings: Some(Time {
                    add_time: Duration::from_secs(arg_3),
                    time_left: Duration::from_secs(arg_2 * 60),
                }),
            })
        } else {
            Err(anyhow::Error::msg(err_msg))
        }
    }
}
