use rand::thread_rng;

use crate::helpers::{get_random_color_with_rng, vec_to_led_data};
use crate::{Color, Effect, LedData, LED_SIZE};
use std::time::Duration;

#[derive(Clone)]
pub struct RandomEffect;
impl Effect for RandomEffect {
    fn new() -> Self {
        Self {}
    }
    fn update(&mut self) -> anyhow::Result<Option<LedData>> {
        const BLOCK_SIZE: usize = 10;
        let mut data: Vec<Color> = Vec::with_capacity(LED_SIZE);
        let mut rng = thread_rng();
        for _ in 0..(LED_SIZE / BLOCK_SIZE) {
            for _ in 0..BLOCK_SIZE {
                data.push(get_random_color_with_rng(&mut rng));
            }
        }
        Ok(Some(vec_to_led_data(data)))
    }

    fn get_config(&self) -> crate::EffectConfig {
        crate::EffectConfig {
            delay: Duration::from_secs(1),
        }
    }
}
