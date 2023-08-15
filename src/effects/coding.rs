use std::{fs, iter, path::Path};

use neovim_lib::{Neovim, NeovimApi, Session};

use crate::{Effect, LED_SIZE};

pub(crate) struct CodingEffect {
    nvim: Neovim,
}

#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

fn find_nvim_socket() -> Result<Option<String>, std::io::Error> {
    let dir = Path::new("/run/user/1000");

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.to_str().unwrap().contains("nvim") {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }

    Ok(None)
}

impl Effect for CodingEffect {
    fn new() -> Self
    where
        Self: Sized,
    {
        let path = find_nvim_socket().unwrap().expect("no nvim socket found");
        let session = Session::new_unix_socket(path);
        let mut nvim = Neovim::new(session.unwrap());
        nvim.session.start_event_loop();

        CodingEffect { nvim }
    }

    fn update(&mut self) -> Result<crate::LedData, Box<dyn std::error::Error>> {
        let mode = self.nvim.get_mode().unwrap();
        let mode = mode[0].1.as_str().unwrap();

        let mode = match mode {
            "n" => Mode::Normal,
            "i" => Mode::Insert,
            "v" => Mode::Visual,
            "V" => Mode::Visual,
            "c" => Mode::Command,
            _ => Mode::Command,
        };

        let color = match mode {
            Mode::Normal => (0, 0, 255),
            Mode::Insert => (0, 255, 0),
            Mode::Visual => (100, 0, 255),
            Mode::Command => (255, 0, 0),
        };

        Ok(iter::repeat(color).take(LED_SIZE).collect())
    }
}
