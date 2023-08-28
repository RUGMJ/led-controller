mod controller;
mod daemon;
mod effects;
pub mod helpers;

use crate::controller::controller;
use crate::daemon::daemon;

use clap::Parser;
use effects::*;
use fs4::FileExt;

use fuzzy_match::fuzzy_match;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use anyhow::Result;

use std::fs::OpenOptions;
use std::time::Duration;

const UNIVERSE: u16 = 1;

const LED_SIZE: usize = 170;

type LedData = [Color; LED_SIZE];

#[derive(Parser, Clone)]
#[command(
    author = "rugmj",
    version = "0.0.1",
    about = "Led Controller",
    long_about = "A program to control my led strip"
)]
pub struct Args {
    #[arg(
        short = 's',
        long = "set-effect",
        help = format!("Sets the effect to use {:?}", enum_iterator::all::<ClientType>().collect::<Vec<_>>())
    )]
    effect: Option<ClientType>,
    #[arg(
        short = 'w',
        long = "web",
        help = "Sets the status of https://led.rugmj.dev/"
    )]
    web_status: Option<WebStatus>,
    #[arg(
        short = 't',
        long = "test",
        help = "Tests the led strip, red -> green -> blue -> white -> repeat"
    )]
    test: bool,
    #[arg(short = 'k', long = "kill", help = "Kills the daemon running")]
    kill: bool,
}

#[derive(
    Copy,
    Clone,
    Deserialize,
    Serialize,
    PartialEq,
    Debug,
    Display,
    zvariant::Type,
    enum_iterator::Sequence,
)]
pub enum ClientType {
    TestEffect,
    RainbowEffect,
    RandomEffect,
    CodingEffect,
    MusicVisualiserEffect,
    TypingRippleEffect,
}

impl From<&str> for ClientType {
    fn from(value: &str) -> Self {
        let effects = enum_iterator::all::<ClientType>().collect::<Vec<_>>();
        let effects: Vec<_> = effects.iter().map(|x| (x.to_string(), *x)).collect();
        let effects: Vec<_> = effects.iter().map(|(s, x)| (s.as_str(), *x)).collect();

        let result = fuzzy_match(value, effects);
        match result {
            Some(x) => x,
            None => panic!("Unknown Effect {}", value),
        }
    }
}

macro_rules! into_effect {
    ($self:expr, $( $effect:ident ),+) => {
        match $self {
            $(
            ClientType::$effect => Box::new(<$effect>::new()) as Box<dyn Effect + Send >,
        )+
        }
    };
}

impl ClientType {
    fn into_effect(self) -> Box<dyn Effect + Send> {
        into_effect![
            self,
            RainbowEffect,
            RandomEffect,
            TestEffect,
            CodingEffect,
            MusicVisualiserEffect,
            TypingRippleEffect
        ]
    }
}

#[derive(Clone)]
enum WebStatus {
    On,
    Off,
}

impl From<&str> for WebStatus {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "on" => Self::On,
            "off" => Self::Off,
            "true" => Self::On,
            "false" => Self::Off,
            "1" => Self::On,
            "0" => Self::Off,
            _ => Self::On,
        }
    }
}

#[derive(Default)]
pub struct EffectConfig {
    delay: Duration,
}

pub trait Effect {
    fn update(&mut self) -> Result<Option<LedData>>;
    fn get_config(&self) -> EffectConfig {
        EffectConfig {
            delay: Duration::from_millis(10),
        }
    }
    fn new() -> Self
    where
        Self: Sized;
}

#[derive(Copy, Clone, Debug)]
pub struct Color(u8, u8, u8);

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    fn brightness(&mut self, brightness: f32) {
        fn apply_brightness(channel: u8, brightness: f32) -> u8 {
            (channel as f32 * brightness).clamp(0.0, 255.0) as u8
        }

        self.0 = apply_brightness(self.0, brightness);
        self.1 = apply_brightness(self.1, brightness);
        self.2 = apply_brightness(self.2, brightness);
    }
}

impl Color {
    const BLACK: Color = Color(0, 0, 0);
    const WHITE: Color = Color(255, 255, 255);
    const RED: Color = Color(255, 0, 0);
    const GREEN: Color = Color(0, 255, 0);
    const BLUE: Color = Color(0, 0, 255);
    const PURPLE: Color = Color(160, 32, 240);
    const ORANGE: Color = Color(255, 127, 0);
}

fn check_and_mark_running() -> Result<std::fs::File, std::io::Error> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("/home/rugmj/.led-controller/led-controller.lock")
        .unwrap();

    file.try_lock_exclusive()?;
    Ok(file)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let daemonise = check_and_mark_running().is_ok();

    if cfg!(debug_assertions) {
        tokio::spawn(daemon(args.clone())).await.unwrap();
    } else if daemonise {
        use nix::unistd::{fork, ForkResult};

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => daemon(args.clone()).await,
                Ok(ForkResult::Parent { child }) => {
                    println!("Deamon spawned with pid: {}", child)
                }

                Err(_) => println!("Failed to start deamon."),
            };
        }
    }

    controller(args).await.unwrap();
}

#[cfg(test)]
mod tests {}
