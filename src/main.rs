use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
};
use bevy::audio::{AudioPlayer, PlaybackSettings};
use symphonia::{core::audio::AudioBufferRef, default};
use symphonia::core::audio::Signal;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::io::MediaSourceStream;
use rustfft::{num_complex::Complex, FftPlanner};

#[derive(Component)]
pub struct VisualizerBar;

#[derive(Component)]
pub struct BandRange(pub f32, pub f32);

#[derive(Component, Default)]
pub struct BarValue(pub f32);

#[derive(Resource)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub position: usize,
    pub fft: Vec<f32>,
    pub fft_size: usize,
    pub sample_rate: f32,
    pub level: f32, // smoothed music intensity 0..~1+
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            samples: vec![],
            position: 0,
            fft: vec![],
            fft_size: 2048,
            sample_rate: 44100.0,
            level: 0.0,
        }
    }
}

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
        .add_systems(Startup, (setup, load_audio, start_music))
        .add_systems(
            Update,
            (
                process_audio,
                update_bloom_settings, // uses audio + input
                add_bars,
                hello_world,
            ),
        )
        .run();
}

fn start_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    let audio = asset_server.load("track.ogg");

    commands.spawn((
        AudioPlayer::new(audio),
        PlaybackSettings::LOOP, // or PlaybackSettings::ONCE
    ));
}

fn hello_world() {
    println!("i like blowjobs");
}

fn process_audio(mut audio: ResMut<AudioData>, time: Res<Time>) {
    if audio.samples.is_empty() {
        return;
    }

    let fft_size = audio.fft_size;

    if audio.position + fft_size >= audio.samples.len() {
        audio.position = 0;
    }

    let window = audio.samples[audio.position..audio.position + fft_size].to_vec();
    audio.position += 512; // hop size

    let mut buffer: Vec<Complex<f32>> = window
        .into_iter()
        .map(|x| Complex::new(x, 0.0))
        .collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);
    fft.process(&mut buffer);

    audio.fft = buffer.iter().map(|c| c.norm()).collect();

    // Compute a basic "level" from FFT
    let sample_rate = audio.sample_rate;
    let bin_hz = sample_rate / fft_size as f32;

    let min_hz = 20.0;
    let max_hz = 8000.0;

    let start_bin = (min_hz / bin_hz).max(0.0) as usize;
    let end_bin = (max_hz / bin_hz).min(audio.fft.len().saturating_sub(1) as f32) as usize;

    if end_bin > start_bin {
        let slice = &audio.fft[start_bin..=end_bin];
        let energy = slice.iter().sum::<f32>() / (slice.len() as f32);

        // compression & smoothing
        let raw_level = (energy / 10.0).sqrt();
        let target = raw_level.clamp(0.0, 5.0);

        let smooth = 10.0;
        let dt = time.delta_secs();
        let alpha = (1.0 - (-smooth * dt).exp()).clamp(0.0, 1.0);

        audio.level = audio.level * (1.0 - alpha) + target * alpha;
    }
}

fn load_audio(mut audio: ResMut<AudioData>) {
    let file = std::fs::File::open("assets/track.mp3").expect("Failed to open audio file");

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let hint = symphonia::core::probe::Hint::new();

    let probed = default::get_probe()
        .format(&hint, mss, &Default::default(), &Default::default())
        .expect("Unrecognized audio format");

    let mut format = probed.format;
    let track = format.default_track().unwrap();

    // get sample rate if present
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100) as f32;
    audio.sample_rate = sample_rate;

    let mut decoder = default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .unwrap();

    let mut pcm = vec![];

    while let Ok(packet) = format.next_packet() {
        let decoded = decoder.decode(&packet).unwrap();

        match decoded {
            AudioBufferRef::F32(buf) => {
                pcm.extend_from_slice(buf.chan(0));
            }
            _ => panic!("Only f32 PCM supported"),
        }
    }

    println!("Loaded {} samples", pcm.len());
    audio.samples = pcm;
}

fn add_bars(mut commands: Commands) {
    let _spacing = 40.0;
    let _bar_width = 30.0;

    for (_i, (min, max)) in BANDS.iter().enumerate() {
        commands.spawn((
            VisualizerBar,
            BandRange(*min, *max),
            BarValue::default(),
        ));
    }
}

// fn process_bars(query: Query<(&BandRange, &BarValue), With<VisualizerBar>>) {
//     for (range, value) in &query {
//         println!("Bar {:?}-{:?} has value {:?}", range.0, range.1, value.0);
//     }
// }

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 2D camera with bloom
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        DebandDither::Enabled,
    ));

    // Bright sprite
    commands.spawn(Sprite {
        image: asset_server.load("branding/bevy_bird_dark.png"),
        color: Color::srgb(5.0, 5.0, 5.0),
        custom_size: Some(Vec2::splat(160.0)),
        ..default()
    });

    // Circle mesh in center, slightly behind
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(100.))),
        MeshMaterial2d(materials.add(Color::srgb(7.5, 0.0, 7.5))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    // Hexagon mesh in same spot, slightly in front
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(100., 6))),
        MeshMaterial2d(materials.add(Color::srgb(6.25, 9.4, 9.1))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));
}

fn update_bloom_settings(
    camera: Single<(Entity, &Tonemapping, Option<&mut Bloom>), With<Camera>>,
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    audio: Res<AudioData>,
) {
    let (camera_entity, tonemapping, bloom) = camera.into_inner();
    let dt = time.delta_secs();

    match bloom {
        Some(mut bloom) => {
            // manual keyboard-driven intensity
            if keycode.pressed(KeyCode::KeyA) {
                bloom.intensity -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyQ) {
                bloom.intensity += dt / 10.0;
            }
            bloom.intensity = bloom.intensity.clamp(0.0, 1.0);

            // music modulation
            let music_level = (audio.level * 0.3).min(1.0);
            let base = bloom.intensity;
            let music_mix = 0.6;
            bloom.intensity = (1.0 - music_mix) * base + music_mix * music_level;

            if keycode.just_pressed(KeyCode::Space) {
                println!("Toggling bloom OFF");
                commands.entity(camera_entity).remove::<Bloom>();
            }
        }
        None => {
            if keycode.just_pressed(KeyCode::Space) {
                println!("Toggling bloom ON");
                commands.entity(camera_entity).insert(Bloom::default());
            }
        }
    }

    if keycode.just_pressed(KeyCode::KeyO) {
        println!("Cycling tonemapper: {tonemapping:?}");
        commands
            .entity(camera_entity)
            .insert(next_tonemap(tonemapping));
    }
}

fn next_tonemap(tonemapping: &Tonemapping) -> Tonemapping {
    match tonemapping {
        Tonemapping::None => Tonemapping::AcesFitted,
        Tonemapping::AcesFitted => Tonemapping::AgX,
        Tonemapping::AgX => Tonemapping::BlenderFilmic,
        Tonemapping::BlenderFilmic => Tonemapping::Reinhard,
        Tonemapping::Reinhard => Tonemapping::ReinhardLuminance,
        Tonemapping::ReinhardLuminance => Tonemapping::SomewhatBoringDisplayTransform,
        Tonemapping::SomewhatBoringDisplayTransform => Tonemapping::TonyMcMapface,
        Tonemapping::TonyMcMapface => Tonemapping::None,
    }
}
