@group(1) @binding(0)
var<uniform> neon_color: vec4<f32>;

@fragment
fn fragment_main() -> @location(0) vec4<f32> {
    return neon_color;
}

