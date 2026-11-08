import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
};

@if(PREPASS_PIPELINE)
import bevy_pbr::{
    prepass::io::{VertexOutput, FragmentOutput},
    deferred::functions::deferred_output,
};
@else
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var mask: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var mask_sampler: sampler;
/// Multiplier applied to shadowed areas (e.g. dark gray = subtle shadow).
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var<uniform> shadow_tint: vec4<f32>;
/// Multiplier applied to lit areas (white = preserve original color).
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var<uniform> highlight_tint: vec4<f32>;
/// Number of discrete color levels per channel for palette quantization.
/// 0.0 = disabled (continuous color), 6.0 = 216-color palette, 8.0 = 512-color palette, etc.
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var<uniform> color_levels: f32;
/// Light level counted as fully lit; below it the surface darkens in bands. 0.0 = off.
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var<uniform> dark_level: f32;
/// How much the light's own colour tints the surface. 0.0 = brightness only.
@group(constants::MATERIAL_BIND_GROUP) @binding(106) var<uniform> light_tint: f32;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

@if(PREPASS_PIPELINE) {
    return deferred_output(in, pbr_input);
}

@if(!PREPASS_PIPELINE) {
    // Preserve texture; run PBR with white base to get a pure lighting value.
    let texture = pbr_input.material.base_color;
    pbr_input.material.base_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    // The brightest channel is the light level, so a blue or red light bands like a
    // white one of the same strength.
    let light = out.color.rgb;
    let level = max(light.r, max(light.g, light.b));

    // Sample ZAtoon mask at the lighting intensity to get toon quantization.
    let uv = vec2<f32>(level, 0.0);
    let quantization = textureSample(mask, mask_sampler, uv);

    // Key difference from Wind Waker: multiply tint against the original texture
    // rather than replacing it, so model colors are fully preserved.
    var tint = mix(shadow_tint, highlight_tint, quantization);
    if light_tint > 0.0 {
        let hue = light / max(level, 0.0001);
        tint = vec4<f32>(tint.rgb * mix(vec3<f32>(1.0), hue, light_tint), tint.a);
    }
    out.color = texture * tint;

    // Palette quantization: posterize each RGB channel to `color_levels` discrete steps.
    // This maps the continuous color space to a limited palette, giving a Sable-like look.
    if color_levels > 0.0 {
        out.color = vec4<f32>(
            round(out.color.rgb * color_levels) / color_levels,
            out.color.a,
        );
    }

    // After the posterizing, which would otherwise round the dim bands' weakest channels
    // to zero and turn a warm lamp's edge red.
    if dark_level > 0.0 {
        // Three hard steps down to black, rounded so the faint spill at a light's range
        // limit drops out instead of lighting a whole band.
        let reach = clamp(level / dark_level, 0.0, 1.0);
        let banded = floor(reach * 3.0 + 0.5) / 3.0;
        out.color = vec4<f32>(out.color.rgb * banded, out.color.a);
    }

    pbr_input.material.base_color = texture;
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
}
