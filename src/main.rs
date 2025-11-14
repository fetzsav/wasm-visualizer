use bevy::prelude::*;
use bevy::render::camera::OrthographicCameraBundle;
use bevy::sprite::SpriteBundle;

use rustfft::{FftPlanner, num_complex::Complex};

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::default;

#[derive(Resource)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub position: usize,
    pub fft: Vec<f32>,
    pub fft_size: usize,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            samples: vec![],
            position: 0,
            fft: vec![],
            fft_size: 2048,
        }
    }
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
        .add_plugins(DefaultPlugins)
        .init_resource::<AudioData>()
        // .add_systems(Startup, setup_camera)
        .add_systems(Startup, load_audio)
        // .add_systems(Startup, add_bars)
        .add_systems(Update, process_audio)
        // .add_systems(Update, apply_bands)
        // .add_systems(Update, update_bar_visuals)
        .run();
}

// fn setup_camera(mut commands: Commands) {
//     // Bevy 0.17 camera bundle lives here:
//     commands.spawn(OrthographicCameraBundle::new_2d());
// }

fn load_audio(mut audio: ResMut<AudioData>) {
    let file = std::fs::File::open("assets/track.mp3").expect("Failed to open audio file");

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let hint = symphonia::core::probe::Hint::new();

    let probed = default::get_probe()
        .format(&hint, mss, &Default::default(), &Default::default())
        .expect("Unrecognized audio format");

    let mut format = probed.format;
    let track = format.default_track().unwrap();

    let mut decoder = default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .unwrap();

    let mut pcm = vec![];

    while let Ok(packet) = format.next_packet() {
        let decoded = decoder.decode(&packet).unwrap();

        match decoded {
            AudioBufferRef::F32(buf) => {
                // `chan` comes from the `Signal` trait – imported above
                pcm.extend_from_slice(buf.chan(0));
            }
            _ => panic!("Only f32 PCM supported"),
        }
    }

    println!("Loaded {} samples", pcm.len());
    audio.samples = pcm;
}

// fn add_bars(mut commands: Commands) {
//     let spacing = 70.0;
//     let width = 40.0;
//     let base_y = -200.0;

//     for (i, (min, max)) in BANDS.iter().enumerate() {
//         let x = i as f32 * spacing - (BANDS.len() as f32 * spacing / 2.0);

//         commands.spawn((
//             VisualizerBar,
//             BandRange(*min, *max),
//             BarValue::default(),
//             SpriteBundle {
//                 sprite: Sprite {
//                     color: Color::srgba(0.1, 0.7, 1.0, 1.0),
//                     custom_size: Some(Vec2::new(width, 5.0)),
//                     ..Default::default()
//                 },
//                 transform: Transform::from_xyz(x, base_y, 0.0),
//                 ..Default::default()
//             },
//         ));
//     }
// }

fn process_audio(mut audio: ResMut<AudioData>) {
    if audio.samples.is_empty() {
        return;
    }

    let fft_size = audio.fft_size;

    if audio.position + fft_size >= audio.samples.len() {
        audio.position = 0;
    }

    let window = audio.samples[audio.position..audio.position + fft_size].to_vec();
    audio.position += 512;

    let mut buffer: Vec<Complex<f32>> = window.into_iter().map(|x| Complex::new(x, 0.0)).collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);
    fft.process(&mut buffer);

    audio.fft = buffer.iter().map(|c| c.norm()).collect();
}

// fn apply_bands(
//     audio: Res<AudioData>,
//     mut query: Query<(&BandRange, &mut BarValue), With<VisualizerBar>>,
// ) {
//     if audio.fft.is_empty() {
//         return;
//     }

//     let sample_rate = 44100.0;
//     let bin_hz = sample_rate / audio.fft_size as f32;

//     for (band, mut value) in &mut query {
//         let start_bin = (band.0 / bin_hz) as usize;
//         let end_bin = (band.1 / bin_hz) as usize;
//         let end_bin = end_bin.min(audio.fft.len() - 1);

//         let energy =
//             audio.fft[start_bin..=end_bin].iter().sum::<f32>() / (end_bin - start_bin + 1) as f32;

//         // Smooth a bit so it doesn't jitter like crazy
//         value.0 = value.0 * 0.7 + energy * 0.3;
//     }
// }

// fn update_bar_visuals(
//     mut query: Query<(&BarValue, &mut Sprite, &mut Transform), With<VisualizerBar>>,
// ) {
//     for (value, mut sprite, mut transform) in &mut query {
//         let height = (value.0 * 350.0).clamp(4.0, 500.0);

//         sprite.custom_size = Some(Vec2::new(40.0, height));
//         transform.translation.y = -200.0 + height / 2.0;

//         let intensity = (value.0 * 4.0).min(1.0);

//         sprite.color = Color::srgba(0.1, 0.6 + intensity * 0.4, 1.0, 1.0);
//     }
// }
