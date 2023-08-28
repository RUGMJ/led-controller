use std::{fs, iter, path::Path};

use neovim_lib::{Neovim, NeovimApi, Session};

use crate::{helpers::vec_to_led_data, Color, Effect, LED_SIZE};

pub struct CodingEffect {
    nvim: Neovim,
    last_mode: Option<Mode>,
}

#[derive(Debug, PartialEq, Clone)]
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

        CodingEffect {
            nvim,
            last_mode: None,
        }
    }

    fn update(&mut self) -> anyhow::Result<Option<crate::LedData>> {
        let mode = self.nvim.get_mode()?;
        let mode = mode[0].1.as_str().unwrap();

        let mode = match mode {
            "n" => Mode::Normal,
            "i" => Mode::Insert,
            "v" => Mode::Visual,
            "V" => Mode::Visual,
            "c" => Mode::Command,
            _ => Mode::Command,
        };

        let last_mode = &self.last_mode;

        if last_mode.as_ref().is_some_and(|l| l == &mode) {
            return Ok(None);
        }

        self.last_mode = Some(mode.clone());

        let color = match mode {
            Mode::Normal => Color::BLUE,
            Mode::Insert => Color::GREEN,
            Mode::Visual => Color::PURPLE,
            Mode::Command => Color::ORANGE,
        };

        Ok(Some(vec_to_led_data(
            iter::repeat(color).take(LED_SIZE).collect(),
        )))
    }
}
