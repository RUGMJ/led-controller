use std::error::Error;

use crate::{Effect, LedData, LED_SIZE};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(Clone)]
pub struct RandomEffect;
impl Effect for RandomEffect {
    fn new() -> Self {
        Self {}
    }
    fn update(&mut self) -> Result<LedData, Box<dyn Error>> {
        const BLOCK_SIZE: usize = 10;
        let mut data: LedData = vec![];
        let mut rng = thread_rng();
        for _ in 0..(LED_SIZE / BLOCK_SIZE) {
            let mut channels = [0, 0, 0];

            channels[0] = rng.gen_range(0..255);
            channels[1] = rng.gen_range(0..(255 - channels[0]));
            channels[2] = 255 - channels[0] - channels[1];

            channels.shuffle(&mut rng);

            for _ in 0..BLOCK_SIZE {
                data.push((channels[0], channels[1], channels[2]));
            }
        }
        Ok(data)
    }
}
