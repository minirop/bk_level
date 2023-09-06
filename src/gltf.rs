
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Extension {
    KhrTextureTransform {
        scale: [f32; 2],
        offset: [f32; 2],
        rotation: f32,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimationPath {
    Position,
    Rotation,
    Scale,
    Weight,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gltf {
    pub asset: Asset,
    pub accessors: Vec<Accessor>,
    pub animations: Vec<Animation>,
    pub buffers: Vec<Buffer>,
    pub buffer_views: Vec<BufferView>,
    pub images: Vec<Image>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
    pub samplers: Vec<Sampler>,
    pub scenes: Vec<Scene>,
    pub textures: Vec<Texture>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessor {
    pub buffer_view: usize,
    pub byte_offset: u32,
    pub component_type: u32,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<[f32; 3]>,
    pub normalized: bool,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animation {
    channels: Vec<AnimationChannel>,
    samplers: Vec<AnimationSampler>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationChannel {
    sampler: usize,
    target: AnimationChannelTarget,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationChannelTarget {
    node: usize,
    path: AnimationPath,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationSampler {
    input: usize,
    output: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub generator: String,
    pub version: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    pub byte_length: u32,
    pub uri: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    pub buffer: usize,
    pub byte_length: u32,
    pub byte_offset: u32,
    pub byte_stride: u32,
    pub target: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub uri: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Material {
    pub pbr_metallic_roughness: PbrMetallicRoughness,
    pub alpha_mode: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f32; 3]>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PbrMetallicRoughness {
    pub base_color_texture: TextureInfo,
    pub metallic_factor: f32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Primitive {
    pub attributes: HashMap<String, usize>,
    pub material: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sampler {
    pub mag_filter: u32,
    pub min_filter: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub nodes: Vec<usize>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Texture {
    pub sampler: usize,
    pub source: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureInfo {
    pub index: usize,
    pub extensions: HashMap<String, Extension>,
}