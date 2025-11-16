use bevy::prelude::*;
use bevy::prelude::*;
use symphonia::default::*;
use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::codecs::DecoderOptions;
use rustfft::{FftPlanner, num_complex::Complex};

pub struct AudioData {
    pub samples: Vec<f32>,       // decoded PCM mono signal
    pub position: usize,         // playback cursor
    pub fft: Vec<f32>,           // last FFT magnitudes
    pub fft_size: usize,
}

#[derive(Component)]
pub struct VisualizerBar;

#[derive(Component)]
pub struct BandRange(pub f32, pub f32);

#[derive(Component, Default)]
pub struct BarValue(pub f32);



pub const BANDS: [(f32, f32); 10] = [
    (20.0, 31.0),
    (31.0, 63.0),
    (63.0, 125.0),
    (125.0, 250.0),
    (250.0, 500.0),
    (500.0, 1000.0),
    (1000.0, 2000.0),
    (2000.0, 4000.0),
    (4000.0, 8000.0),
    (8000.0, 16000.0),
];






fn main() {
    App::new()
    .add_systems(Update, hello_world)
    .add_systems(Update, add_bars)
    .run();
}


fn hello_world() {
    println!("hello world!");
}


fn add_bars(mut commands: Commands) {
    let spacing = 40.0;
    let bar_width = 30.0;

    for (i, (min, max)) in BANDS.iter().enumerate() {
        commands.spawn((
            VisualizerBar,
            BandRange(*min, *max),
            BarValue::default(),
        ));
    }
}

fn process_bars(query: Query<(&BandRange, &BarValue), With<VisualizerBar>>) {
    for (range, value) in &query {
        println!(
            "Bar {:?}-{:?} has value {:?}",
            range.0, range.1, value.0
        );
    }
}



