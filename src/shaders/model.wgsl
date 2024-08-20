struct CameraUniform {
    view_projection: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) texture_index: u32,
    @location(2) @interpolate(perspective) light: vec3f,
    @location(3) apply_dark_texture: u32,
}

struct ModelQuad {
    vertex_positions: array<vec3<f32>, 4>,
    normal: vec3f,
    uv: array<vec2f, 4>
}

struct Face {
    lighting: array<LightLevel, 4>,
    block_position: array<u32, 3>,
    texture_index: u32,
    quad_index: u32,
}

struct LightLevel {
    block: u32,
    sky: u32
}

@group(2) @binding(0) var<uniform> camera: CameraUniform;
@group(3) @binding(0) var<storage, read> face_buffer: array<vec4u>;
@group(1) @binding(0) var<storage, read> quad_buffer: array<ModelQuad>;
@group(4) @binding(0) var<uniform> translation: vec2i;

const TEXTURE_ATLAS_WIDTH: u32 = 256;
const TEXTURE_ATLAS_HEIGHT: u32 = 256;

fn construct_light_level(raw_data: u32) -> LightLevel {
    var out: LightLevel;
    out.block = raw_data & 15u;
    out.sky = raw_data >> 4u & 15u;

    return out;
}

fn construct_face(vertex_index: u32) -> Face {
    var face: Face;
    let face_raw_data = face_buffer[vertex_index >> 2u];
    var lighting: array<LightLevel, 4>;

    lighting[0] = construct_light_level(face_raw_data.x);
    lighting[1] = construct_light_level(face_raw_data.x >> 8u);
    lighting[2] = construct_light_level(face_raw_data.x >> 16u);
    lighting[3] = construct_light_level(face_raw_data.x >> 24u);
    face.lighting = lighting;

    var block_position: array<u32, 3>;
    block_position[0] = (face_raw_data.y) & 31u;
    block_position[1] = (face_raw_data.y >> 5u & 31u);
    block_position[2] = (face_raw_data.y >> 10u & 31u);
    face.block_position = block_position;

    face.texture_index = face_raw_data.y >> 16u & 65535u;
    face.quad_index = (face_raw_data.z & 65535u);
    
    return(face);
}

@group(0) @binding(2) var light_map_tex: texture_2d<f32>;
@group(0) @binding(3) var light_map_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) i: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var face = construct_face(i);
    let quad = quad_buffer[face.quad_index];
    
    let i_mod_4 = i % 4u;

    var vertex_positions = quad.vertex_positions;
    var vertex = vertex_positions[i_mod_4] + vec3f(f32(face.block_position[0]), f32(face.block_position[1]), f32(face.block_position[2]));

    vertex.x += f32(translation.x * 32);
    vertex.z += f32(translation.y * 32);
    vertex.y += f32(instance_index);
    var uvs = quad.uv;
    let uv = uvs[i_mod_4];

    var out: VertexOutput;

    out.clip_position = camera.view_projection * vec4f(vertex, 1.0);
    out.uv = uv;
    out.texture_index = face.texture_index;
    let light_level = face.lighting[i_mod_4];
    let light_color = textureLoad(light_map_tex, vec2u(light_level.block, light_level.sky), 0);
    out.light = light_color.xyz;
    out.apply_dark_texture = u32(quad.normal.x > 0.0 || quad.normal.x < 0.0);
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
const BLOCK_SIZE: f32 = 1.0 / 16.0;
const DARK_TEXTURE_COORDS: vec2u = vec2u(11u % 16u, 11u / 16u);
const DARK_TEXTURE_BASE_UV: vec2f = vec2f(f32(DARK_TEXTURE_COORDS.x) * BLOCK_SIZE, f32(DARK_TEXTURE_COORDS.y) * BLOCK_SIZE);
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let texture_coords = vec2u(in.texture_index % 16u, in.texture_index / 16u);
    let base_uv = vec2f(f32(texture_coords.x) * BLOCK_SIZE, f32(texture_coords.y) * BLOCK_SIZE);
    let scaled_uv = in.uv * vec2f(BLOCK_SIZE, BLOCK_SIZE);
    let uv = base_uv + scaled_uv;
    var color = textureSample(t_diffuse, s_diffuse, uv) * vec4(in.light, 1.0);
    if color.w == 0.0 { discard; }
    if in.apply_dark_texture > 0 {
        let dark_tex_uv = DARK_TEXTURE_BASE_UV + scaled_uv;
        color *= textureSample(t_diffuse, s_diffuse, dark_tex_uv);
    }
    return color;
}