use std::{error::Error, thread, time::Duration};

use crate::{Effect, LedData, LED_SIZE};
pub struct TestEffect {
    iterator: usize,
}
impl Effect for TestEffect {
    fn new() -> Self {
        Self { iterator: 0 }
    }
    fn update(&mut self) -> Result<LedData, Box<dyn Error>> {
        self.iterator += 1;
        thread::sleep(Duration::from_secs(1));
        match self.iterator {
            1 => Ok([(255, 0, 0); LED_SIZE].to_vec()),
            2 => Ok([(0, 255, 0); LED_SIZE].to_vec()),
            3 => Ok([(0, 0, 255); LED_SIZE].to_vec()),
            _ => {
                self.iterator = 0;
                Ok([(255, 255, 255); LED_SIZE].to_vec())
            }
        }
    }
}
