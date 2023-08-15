mod controller;
mod daemon;
mod effects;

use crate::controller::controller;
use crate::daemon::daemon;

use clap::Parser;
use effects::*;
use fs4::FileExt;

use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::OpenOptions;

const UNIVERSE: u16 = 1;

const LED_SIZE: usize = 170;

type LedData = Vec<(u8, u8, u8)>;

#[derive(Parser, Clone)]
#[command(
    author = "rugmj",
    version = "0.0.1",
    about = "Led Controller",
    long_about = "A program to control my led strip"
)]
struct Args {
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
    Clone, Deserialize, Serialize, PartialEq, Debug, zvariant::Type, enum_iterator::Sequence,
)]
pub enum ClientType {
    TestEffect,
    RainbowEffect,
    RandomEffect,
    CodingEffect,
}

impl From<&str> for ClientType {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "rainboweffect" => Self::RainbowEffect,
            "randomeffect" => Self::RandomEffect,
            "testeffect" => Self::TestEffect,
            "codingeffect" => Self::CodingEffect,
            misc => panic!("Unknown client type: {misc}"),
        }
    }
}

macro_rules! into_effect {
    ($self:expr, $( $effect:ident ),+) => {
        match $self {
            $(
            ClientType::$effect => Box::new(<$effect>::new()) as Box<dyn Effect + Send + Sync>,
        )+
        }
    };
}

impl ClientType {
    fn into_effect(self) -> Box<dyn Effect + Send + Sync> {
        into_effect![self, RainbowEffect, RandomEffect, TestEffect, CodingEffect]
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

pub trait Effect {
    fn update(&mut self) -> Result<LedData, Box<dyn Error>>;
    fn register(&mut self) {}
    fn unregister(&mut self) {}
    fn new() -> Self
    where
        Self: Sized;
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

    let mut daemonise = check_and_mark_running().is_ok();

    // If web status is set dont spawn a daemon
    if args.web_status.is_some() {
        daemonise = false;
    }

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
