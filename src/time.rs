use std::{fmt, time::Duration};

use anyhow::Context;

#[derive(Clone, Debug)]
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

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
        if let Some(arg_1) = args.get(1) {
            match *arg_1 {
                "none" => Ok(Settings {
                    time_settings: None,
                }),
                "fischer" => {
                    let arg_2 = args
                        .get(2)
                        .context("time_settings: wrong number of arguments")?;
                    let arg_2 = arg_2
                        .parse::<u64>()
                        .context("time_settings: arg 2 is not an integer")?;

                    let arg_3 = args
                        .get(3)
                        .context("time_settings: wrong number of arguments")?;
                    let arg_3 = arg_3
                        .parse::<u64>()
                        .context("time_settings: arg 3 is not an integer")?;

                    Ok(Settings {
                        time_settings: Some(Time {
                            add_time: Duration::from_secs(arg_3),
                            time_left: Duration::from_secs(arg_2 * 60),
                        }),
                    })
                }
                _ => Err(anyhow::Error::msg(
                    "time_settings: the argument is not 'none' or 'fischer'",
                )),
            }
        } else {
            Err(anyhow::Error::msg(
                "time_settings: wrong number of arguments",
            ))
        }
    }
}
