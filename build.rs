use std::{fs, io, path::Path};

static HOME: &str = "hnefatafl-copenhagen";

fn main() -> Result<(), io::Error> {
    if std::env::var("DOCS_RS").is_ok() {
        return Ok(());
    }

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
