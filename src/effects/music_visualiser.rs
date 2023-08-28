use std::time::Duration;

use audioviz::{
    io::{Device, Input, InputController},
    spectrum::{
        config::{Interpolation, ProcessorConfig, StreamConfig, VolumeNormalisation},
        stream::Stream,
    },
};

use crate::{
    helpers::{get_random_color, vec_to_led_data},
    Color, Effect, LED_SIZE,
};

pub struct MusicVisualiserEffect {
    _audio_input: Input,
    input_controller: InputController,
    stream: Stream,
    color: Color,
}

unsafe impl Send for MusicVisualiserEffect {}

impl Effect for MusicVisualiserEffect {
    fn update(&mut self) -> anyhow::Result<Option<crate::LedData>> {
        if let Some(data) = self.input_controller.pull_data() {
            self.stream.push_data(data);
            self.stream.update();
        }

        let frequencies = self.stream.get_frequencies();
        let frequencies = frequencies.first().unwrap().iter();

        let frequencies = frequencies.map(|f| (f.volume.clamp(0.0, 1.0)));

        if frequencies.clone().all(|f| f == 0.0) {
            self.color = get_random_color();
        };

        let frequencies: Vec<f32> = frequencies.collect();

        Ok(Some(vec_to_led_data(
            frequencies
                .iter()
                .map(|f| -> Color {
                    let mut color = self.color;
                    color.brightness(*f);
                    color
                })
                .collect(),
        )))
    }

    fn get_config(&self) -> crate::EffectConfig {
        crate::EffectConfig {
            delay: Duration::ZERO,
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        let mut audio_input = Input::new();
        let (_, _, input_controller) = audio_input.init(&Device::DefaultInput, None).unwrap();

        let config = StreamConfig {
            channel_count: 1,
            processor: ProcessorConfig {
                resolution: Some(LED_SIZE),
                volume_normalisation: VolumeNormalisation::Mixture,
                interpolation: Interpolation::Cubic,
                ..ProcessorConfig::default()
            },
            ..StreamConfig::default()
        };
        let stream = Stream::new(config);

        MusicVisualiserEffect {
            _audio_input: audio_input,
            input_controller,
            stream,
            color: get_random_color(),
        }
    }
}
