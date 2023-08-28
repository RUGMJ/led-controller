use std::{
    iter,
    time::{Duration, Instant},
};

use evdev::{Device, InputEventKind};
use nix::sys::epoll::EpollFlags;

use crate::{
    helpers::{get_random_color, vec_to_led_data},
    Color, Effect, LED_SIZE,
};

const EVENT_KEY: &str = "/dev/input/event3";

const RIPPLE_TIME: Duration = Duration::from_secs(1);

#[derive(Debug)]
struct Ripple {
    start_time: Instant,
    colour: Color,
}

pub struct TypingRippleEffect {
    device: Device,
    ripples: Vec<Ripple>,
}

impl Effect for TypingRippleEffect {
    fn update(&mut self) -> anyhow::Result<Option<crate::LedData>> {
        if let Ok(events) = self.device.fetch_events() {
            let events: Vec<_> = events.collect();
            for event in events {
                if let InputEventKind::Key(_) = event.kind() {
                    if event.value() == 0 {
                        let colour = get_random_color();

                        self.ripples.push(Ripple {
                            start_time: Instant::now(),
                            colour,
                        });
                    }
                }
            }
        }

        self.ripples
            .retain(|ripple| ripple.start_time.elapsed() < RIPPLE_TIME);

        let positions: Vec<_> = self
            .ripples
            .iter()
            .map(|r| {
                ((r.start_time.elapsed().as_millis() as f32 / RIPPLE_TIME.as_millis() as f32)
                    * LED_SIZE as f32) as usize
            })
            .collect();

        let colours: Vec<_> = self.ripples.iter().map(|ripple| ripple.colour).collect();

        let mut data: Vec<Color> = iter::repeat(Color::BLACK).take(LED_SIZE).collect();
        for (pos, colour) in positions.iter().take(LED_SIZE).zip(colours.iter()) {
            data[*pos] = *colour;
        }

        data.reverse();

        Ok(Some(vec_to_led_data(data)))
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        let device = Device::open(EVENT_KEY).unwrap();

        set_non_blocking(&device).unwrap();

        Self {
            device,
            ripples: Vec::new(),
        }
    }
}

fn set_non_blocking(device: &Device) -> anyhow::Result<()> {
    use nix::{
        fcntl::{fcntl, FcntlArg, OFlag},
        sys::epoll,
    };
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

    let fd = device.as_raw_fd();

    fcntl(fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?;

    let epoll_fd = epoll::epoll_create1(epoll::EpollCreateFlags::EPOLL_CLOEXEC)?;
    let epoll_fd = unsafe { OwnedFd::from_raw_fd(epoll_fd) };

    let mut event = epoll::EpollEvent::new(EpollFlags::EPOLLIN, 0);

    epoll::epoll_ctl(
        epoll_fd.as_raw_fd(),
        epoll::EpollOp::EpollCtlAdd,
        fd,
        Some(&mut event),
    )?;
    Ok(())
}
