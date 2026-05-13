#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(3) @binding(100) var mask: texture_2d<f32>;
@group(3) @binding(101) var mask_sampler: sampler;
/// Multiplier applied to shadowed areas (e.g. dark gray = subtle shadow).
@group(3) @binding(102) var<uniform> shadow_tint: vec4<f32>;
/// Multiplier applied to lit areas (white = preserve original color).
@group(3) @binding(103) var<uniform> highlight_tint: vec4<f32>;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    // Preserve texture; run PBR with white base to get a pure lighting value.
    let texture = pbr_input.material.base_color;
    pbr_input.material.base_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    // Sample ZAtoon mask at the lighting intensity to get toon quantization.
    let uv = vec2<f32>(out.color.r, 0.0);
    let quantization = textureSample(mask, mask_sampler, uv);

    // Key difference from Wind Waker: multiply tint against the original texture
    // rather than replacing it, so model colors are fully preserved.
    let tint = mix(shadow_tint, highlight_tint, quantization);
    out.color = texture * tint;

    pbr_input.material.base_color = texture;
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
