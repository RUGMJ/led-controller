use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

use crate::{Color, LedData, LED_SIZE};

/// Gets a "led strip friendly" random color
pub fn get_random_color() -> Color {
    let mut rng = thread_rng();
    get_random_color_with_rng(&mut rng)
}

/// Gets a "led strip friendly" random color
pub fn get_random_color_with_rng(rng: &mut ThreadRng) -> Color {
    let mut channels = [0; 3];

    channels[0] = rng.gen_range(0..255);
    channels[1] = rng.gen_range(0..(255 - channels[0]));
    channels[2] = 255 - channels[0] - channels[1];

    channels.shuffle(rng);
    Color::new(channels[0], channels[1], channels[2])
}

pub fn vec_to_led_data(data: Vec<Color>) -> LedData {
    assert_eq!(data.len(), LED_SIZE);

    data.try_into().unwrap()
}
