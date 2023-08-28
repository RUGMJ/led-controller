use crate::{helpers::vec_to_led_data, Color, Effect, LedData, LED_SIZE};
use std::{collections::VecDeque, f64::consts::PI};

#[derive(Clone)]
pub struct RainbowEffect {
    iterator: usize,
}

impl Effect for RainbowEffect {
    fn new() -> Self {
        Self { iterator: 0 }
    }

    fn update(&mut self) -> anyhow::Result<Option<LedData>> {
        let mut data: VecDeque<Color> = VecDeque::with_capacity(LED_SIZE);
        for i in 0..LED_SIZE {
            let phase_r = i as f64 * 2.0 * PI / LED_SIZE as f64;
            let phase_g = (i as f64 * 2.0 * PI / LED_SIZE as f64) + (2.0 * PI / 3.0);
            let phase_b = (i as f64 * 2.0 * PI / LED_SIZE as f64) + (4.0 * PI / 3.0);

            let r = ((phase_r).sin() * 127.0 + 128.0) as u8;
            let g = ((phase_g).sin() * 127.0 + 128.0) as u8;
            let b = ((phase_b).sin() * 127.0 + 128.0) as u8;

            data.push_back(Color::new(r, g, b));
        }

        data.rotate_right(self.iterator);
        let data = Vec::from(data);

        self.iterator += 1;
        if self.iterator >= data.len() {
            self.iterator = 0;
        }

        Ok(Some(vec_to_led_data(data)))
    }
}
