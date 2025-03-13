#[cfg(not(docsrs))]
use std::{fs, io, path::Path};

#[cfg(not(docsrs))]
static HOME: &str = "hnefatafl-copenhagen";

#[cfg(not(docsrs))]
fn main() -> Result<(), io::Error> {
    if let Some(mut path) = dirs::data_dir() {
        path = path.join(HOME);
        fs::create_dir_all(&path)?;

        fs::copy(
            Path::new("sound").join("capture.ogg"),
            path.join("capture.ogg"),
        )?;

        fs::copy(
            Path::new("sound").join("game_over.ogg"),
            path.join("game_over.ogg"),
        )?;

        fs::copy(Path::new("sound").join("move.ogg"), path.join("move.ogg"))?;
    }

    Ok(())
}
