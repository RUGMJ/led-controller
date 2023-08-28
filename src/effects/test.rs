use std::{iter, time::Duration};

use crate::{helpers::vec_to_led_data, Color, Effect, LedData, LED_SIZE};
pub struct TestEffect {
    iterator: usize,
}
impl Effect for TestEffect {
    fn new() -> Self {
        Self { iterator: 0 }
    }
    fn update(&mut self) -> anyhow::Result<Option<LedData>> {
        self.iterator += 1;
        let colour = match self.iterator {
            1 => Color::RED,
            2 => Color::GREEN,
            3 => Color::BLUE,
            _ => {
                self.iterator = 0;
                Color::WHITE
            }
        };

        Ok(Some(vec_to_led_data(
            iter::repeat(colour).take(LED_SIZE).collect(),
        )))
    }

    fn get_config(&self) -> crate::EffectConfig {
        crate::EffectConfig {
            delay: Duration::from_secs(1),
        }
    }
}
