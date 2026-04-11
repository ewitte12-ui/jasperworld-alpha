// Alpha-silhouette fragment shader.
// Samples the sprite texture for its alpha mask only — RGB is discarded.
// Outputs a pure solid HDR yellow wherever the sprite is opaque, giving a
// clean contour outline with no colour-multiply artefacts.
#import bevy_pbr::forward_io::VertexOutput

// #{MATERIAL_BIND_GROUP} is replaced by Bevy's shader preprocessor with the
// correct group index (3 in Bevy 0.18 — group 2 is reserved for mesh bindings).
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var sprite_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var sprite_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(sprite_texture, sprite_sampler, in.uv).a;
    // HDR yellow (values > 1.0 cooperate with bloom if enabled).
    return vec4<f32>(2.0, 1.7, 0.0, alpha);
}
