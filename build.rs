use std::{fs, io::Error, path::Path};

static HOME: &str = "hnefatafl-copenhagen";

fn main() -> Result<(), Error> {
    if let Some(mut path) = dirs::data_dir() {
        path = path.join(HOME);
        fs::create_dir_all(&path)?;

        fs::copy(
            Path::new("sound").join("capture.ogg"),
            path.join("capture.ogg"),
        )?;
        fs::copy(Path::new("sound").join("move.ogg"), path.join("move.ogg"))?;
    }

    Ok(())
}
