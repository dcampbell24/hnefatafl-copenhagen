use std::{fs, io, path::Path};

static HOME: &str = "hnefatafl-copenhagen";

fn main() -> Result<(), io::Error> {
    if let Some(mut path) = dirs::data_dir() {
        path = path.join(HOME);
        fs::create_dir_all(&path)?;

        ok_permission_denied(fs::copy(
            Path::new("sound").join("capture.ogg"),
            path.join("capture.ogg"),
        ))?;

        ok_permission_denied(fs::copy(
            Path::new("sound").join("game_over.ogg"),
            path.join("game_over.ogg"),
        ))?;

        ok_permission_denied(fs::copy(
            Path::new("sound").join("move.ogg"),
            path.join("move.ogg"),
        ))?;
    }

    Ok(())
}

fn ok_permission_denied(error: io::Result<u64>) -> io::Result<()> {
    match error {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error),
    }
}
