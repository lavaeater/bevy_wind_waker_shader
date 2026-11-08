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

/// Number of UV grid divisions per axis. Lower = larger blocks = more pixelated.
/// E.g. 8.0 means the texture is sampled on an 8x8 grid.
@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> pixel_density: f32;
/// Discrete color levels per RGB channel. Lower = fewer colors = more pixelated.
/// 0.0 disables posterization.
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var<uniform> color_levels: f32;

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
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

@if(PREPASS_PIPELINE) {
    return deferred_output(in, pbr_input);
}

@if(!PREPASS_PIPELINE) {
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
    return out;
}
}
