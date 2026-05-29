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

/// Number of UV grid divisions per axis. Lower = larger blocks = more pixelated.
/// E.g. 8.0 means the texture is sampled on an 8×8 grid.
@group(3) @binding(100) var<uniform> pixel_density: f32;
/// Discrete color levels per RGB channel. Lower = fewer colors = more pixelated.
/// 0.0 disables posterization.
@group(3) @binding(101) var<uniform> color_levels: f32;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Snap UV to the nearest grid cell so each cell samples the same texel,
    // producing the appearance of a lower-resolution texture.
    var snapped = in;
    if pixel_density > 0.0 {
        snapped.uv = floor(in.uv * pixel_density) / pixel_density;
    }

    var pbr_input = pbr_input_from_standard_material(snapped, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    // Posterize: snap each color channel to `color_levels` discrete steps.
    if color_levels > 0.0 {
        out.color = vec4<f32>(
            round(out.color.rgb * color_levels) / color_levels,
            out.color.a,
        );
    }

    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
