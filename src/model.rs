#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use crate::types::*;
use crate::types::{ Vector2, Vector3 };
use std::collections::HashSet;
use image::RgbaImage;
use std::env::args;
use byteorder::{ ReadBytesExt, BigEndian };
use clap::Parser;
use serde::{ Serialize, Deserialize };
use std::path::Path;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Write;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    textures: Vec<Texture>,
    commands: Vec<F3dex>,
    vertices: Vec<Vertex>,
    collisions: Collisions,
    geometry: Vec<Geometry>,
    unk14: ModelUnk14,
    unk20: Unknown20,
    unk28: u32,
    mesh_list: Vec<Mesh>,
    geometry_type: u16,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Unknown20 {
    unk1: u8,
    unk2: i16,
    unk3: i16,
    unk4: i16,
    unk5: i16,
    unk6: i16,
    unk7: i16,
    unk8: u8,
    unk9: u8,
}

impl Unknown20 {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
    id: u16,
    vertices: Vec<u16>,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    position: Vector3<i16>,
    flag: u16,
    uv: Vector2<i16>,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Vertex {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Texture {
    offset: u32,
    format: TextureFormat,
    unknown: u16,
    width: u8,
    height: u8,
    size: u32,
    #[serde(skip_serializing)]
    hratio: f32,
    #[serde(skip_serializing)]
    wratio: f32,
    palette: Option<Vec<u8>>,
}

impl Texture {
    fn new() -> Self {
        Self {
            offset: 0,
            format: TextureFormat::C4,
            unknown: 0,
            width: 0,
            height: 0,
            size: 0,
            hratio: 1.0,
            wratio: 1.0,
            palette: None,
        }
    }

    fn set_ratio(&mut self, sscale: f32, tscale: f32) {
        self.wratio = sscale / ((self.width as f32) * 32.0);
        self.hratio = tscale / ((self.height as f32) * 32.0);
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GeoColl {
    start_tri_index: u16,
    tri_count: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TriColl {
    vtx_indx_1: u16,
    vtx_indx_2: u16,
    vtx_indx_3: u16,
    unk: u16,
    flags: u32,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collisions {
    min: Vector3<i16>,
    max: Vector3<i16>,
    stride: Vector2<i16>,
    scale: u16,
    geo: Vec<GeoColl>,
    tri: Vec<TriColl>,
    unknown: Option<u32>,
}

impl Collisions {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelUnk14_0 {
    unk1: Vector3<i16>,
    unk2: Vector3<i16>,
    unk3: Vector3<i16>,
    unk4: Vector3<u8>,
    unk5: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelUnk14_1 {
    unk1: u16,
    unk2: u16,
    unk3: Vector3<i16>,
    unk4: Vector3<u8>,
    unk5: u8,
    unk6: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelUnk14_2 {
    unk1: u16,
    unk2: Vector3<i16>,
    unk3: u8,
    unk4: u8,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelUnk14 {
    unk14_0: Vec<ModelUnk14_0>,
    unk14_1: Vec<ModelUnk14_1>,
    unk14_2: Vec<ModelUnk14_2>,
}

impl ModelUnk14 {
    fn new() -> Self {
        Default::default()
    }
}

struct Command {
    id: u8,
    b1: u8,
    b2: u8,
    b3: u8,
    b4: u8,
    b5: u8,
    b6: u8,
    b7: u8,
    value: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ColourFormat {
    Rgba,
    Yuv,
    Palette,
    GrayscaleAlpha,
    Grayscale,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TextureFormat {
    C4,
    C8,
    Rgba16,
    Rgba32,
    IA8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum F3dex {
    SPNoOp,
    Vertex { index: u16, count: u8, address: u32 },
    DisplayList { store_ra: bool, address: u32 },
    Triangle2 { v1: u8, v2: u8, v3: u8, v4: u8, v5: u8, v6: u8 },
    ClearGeometryMode(u32),
    SetGeometryMode(u32),
    EndDisplayList,
    SetOtherModeL { amount: u8, count: u8, mode: u32 }, // to improve
    SetOtherModeH { amount: u8, count: u8, mode: u32 }, // to improve
    Texture { mipmaps: u8, descriptor: u8, enable: bool, scalex: f32, scaley: f32 },
    PopMatrix { unk1: u8, unk2: u8, unk3: u8, count: u32 },
    Triangle1 { v1: u8, v2: u8, v3: u8 },
    RdpLoadSync,
    RdpPipeSync,
    LoadTlut { descriptor: u8 },
    SetTileSize { upper_left_s: u16, upper_left_t: u16, descriptor: u8, width: u16, height: u16 },
    LoadBlock { upper_left_s: u16, upper_left_t: u16, descriptor: u8, texels_count: u16, dxt: u16 },
    LoadTile { upper_left_s: u16, upper_left_t: u16, descriptor: u8, lower_right_s: u16, lower_right_t: u16 },
    SetTile { format: ColourFormat, depth: u8, values_per_row: u16,
        tmem_offset: u16, descriptor: u8, palette: u8,
        clamp_mirror_y: u8, unwrapped_y: u8, perspective_div_y: u8,
        clamp_mirror_x: u8, unwrapped_x: u8, perspective_div_x: u8 },
    SetCombine {},
    SettImg { format: ColourFormat, depth: u8, address: u32 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    Unknown0x00 { len: u32 },
    Sort { x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32,
        draw_only_nearest: bool, offset2: u16, len: u32 },
    Bone { address: u16, len: u8, id: u8, unk: u16 },
    LoadDisplayList { len: u32, offset: u16, tri_count: u16 },
    Skinning,
    Lod { layout_offset: u32, max_dist: f32, min_dist: f32, test_x: f32, test_y: f32, test_z: f32 },
    ReferencePoint { index: u16, bone: u16, x: f32, y: f32, z: f32 },
    Selector { selector: u16, indices: Vec<i32>, commands: Vec<Geometry> },
    DrawDistance { len: u16, min_x: i16, min_y: i16, min_z: i16, max_x: i16, max_y: i16, max_z: i16, unk1: u32, unk2: u16 },
    Unknown0x0e,
    Group0x0f { header: Vec<u32>, commands: Vec<Geometry> },
    Unknown0x10 { len: u32, unk1: u32, unk2: u32 },
}

impl Model {
    pub fn read_bin_obj(filename: &str) -> std::io::Result<()> {
        let file_size = std::fs::metadata(filename).unwrap().len();
        let mut f = File::open(filename)?;
        let header = f.read_u32::<BigEndian>()?;
        assert_eq!(header, 0x0B);

        let output_dir = Path::new(filename);
        let output_dir = output_dir.file_stem().unwrap();
        let output_dir = output_dir.to_str().unwrap();
        std::fs::create_dir_all(output_dir)?;

        // HEADER
        let coll_start = f.read_u32::<BigEndian>()? + 24;
        let unk0x08_0x0b = f.read_u32::<BigEndian>()?;
        let f3d_start = f.read_u32::<BigEndian>()? + 8;
        let vert_start = f.read_u32::<BigEndian>()? + 24;

        f.seek(SeekFrom::Start((f3d_start - 6) as u64))?;
        let f3d_commands_count = f.read_u16::<BigEndian>()?;

        f.seek(SeekFrom::Start(50))?;
        let vt_count = f.read_u16::<BigEndian>()?;
        f.seek(SeekFrom::Start(57))?;
        let top_length = f.read_u8()?;
        let middle_length = f.read_u8()?;
        let bottom_length = f.read_u8()?;
        let length = ((top_length as u32) << 16) + ((middle_length as u32) << 8) + (bottom_length as u32);
        let tex_count = f.read_u16::<BigEndian>()?;

        if tex_count == 0 {
            panic!("According to BB, 0 textures mean 1. But for now, panic!");
        }

        // READ VERTICES
        f.seek(SeekFrom::Start((vert_start) as u64))?;

        let mut vertices: Vec<Vertex> = vec![];
        for _ in 0..vt_count {
            let mut vert = Vertex::new();

            vert.position.x = f.read_i16::<BigEndian>()?;
            vert.position.y = f.read_i16::<BigEndian>()?;
            vert.position.z = f.read_i16::<BigEndian>()?;
            vert.flag = f.read_u16::<BigEndian>()?;
            vert.uv.x = f.read_i16::<BigEndian>()?;
            vert.uv.y = f.read_i16::<BigEndian>()?;
            vert.r = (f.read_u8()? as f32) / 256.0;
            vert.g = (f.read_u8()? as f32) / 256.0;
            vert.b = (f.read_u8()? as f32) / 256.0;
            vert.a = (f.read_u8()? as f32) / 256.0;

            vertices.push(vert);
        }

        // READ TEXTURES
        f.seek(SeekFrom::Start(64))?;

        let mut textures: Vec<Texture> = vec![];
        for i in 0..tex_count {
            let mut tex = Texture::new();

            tex.offset = f.read_u32::<BigEndian>()?;
            let unknown = f.read_u32::<BigEndian>()?;
            tex.width = f.read_u8()?;
            tex.height = f.read_u8()?;
            let unknown = f.read_u16::<BigEndian>()?;
            let unknown = f.read_u32::<BigEndian>()?;

            textures.push(tex);
        }

        let length2 = length - (tex_count as u32 * 16) - 8;
        if length2 > 0x80000000 { panic!("negative length2?!!"); }

        if length2 > 0 {
            let file_pos = f.stream_position()?;
            let remaining = (file_size - file_pos) as u32;
            if length2 <= remaining {
                // LOL? useless code
            } else {
                panic!("ERROR");
            }
        }

        // READ COMMANDS
        f.seek(SeekFrom::Start(f3d_start as u64))?;

        let mut commands: Vec<Command> = vec![];
        for i in 0..f3d_commands_count {
            let id = f.read_u8()?;
            let b1 = f.read_u8()?;
            let b2 = f.read_u8()?;
            let b3 = f.read_u8()?;
            let value = f.read_u32::<BigEndian>()?;
            let b4 = ((value >> 24) & 0xFF) as u8;
            let b5 = ((value >> 16) & 0xFF) as u8;
            let b6 = ((value >> 8) & 0xFF) as u8;
            let b7 = ((value >> 0) & 0xFF) as u8;
            commands.push(Command { id, b1, b2, b3, b4, b5, b6, b7, value });

            let file_pos = f.stream_position()?;
            let remaining = (file_size - file_pos) as u32;
            if remaining < 8 {
                println!("END AT COMMAND {} ({} bytes remaining)", i, remaining);
            }
        }

        // EXEC COMMANDS
        let mut cache = [0u32; 32usize];

        let mut output_mtl = File::create(format!("{}/output.mtl", output_dir)).expect("Unable to create file");
        for id in 0..tex_count {
            let mtlname = format!("material_{:04}", id);
            writeln!(output_mtl, "newmtl {}", mtlname)?;
            writeln!(output_mtl, "map_Kd image_{:04}.png\n", id)?;
        }
        writeln!(output_mtl, "newmtl material_null")?;

        let mut output = File::create(format!("{}/output.obj", output_dir)).expect("Unable to create file");

        writeln!(output, "mtllib output.mtl")?;

        for v in &vertices {
            writeln!(output, "v {} {} {}", v.position.x, v.position.y, v.position.z)?;
        }

        let mut new_texture = false;
        let mut texture_is_null = true;
        let mut current_texture = 0;
        let mut sscale = 0.0;
        let mut tscale = 0.0;
        let mut vt_index = 1;
        let mut texture_format = 0;
        let mut texel_size = 0;
        let mut line_size = 0;
        let mut created_textures = HashSet::<String>::new();

        for (index1, command) in commands.iter().enumerate() {
            match command.id {
                4 => {
                    let start = command.b1 >> 1;
                    let start = if start > 63 { 63 } else { start };
                    let count = command.b2 >> 2;
                    let mut index1 = ((command.value & 0x00FFFFFF) / 16) as usize;
                    assert!(start+count <= 32);

                    for index2 in start..(start+count) {
                        if index1 < vertices.len() {
                            cache[index2 as usize] = index1 as u32;
                        }
                        index1 += 1;
                    }

                    if new_texture {
                        let mtlpath = format!("image_{:04}.png", current_texture);
                        if !created_textures.contains(&mtlpath) {
                            let texture = &textures[current_texture];
                            let start_offset = texture.offset + 32 + 64 + (textures.len() * 16) as u32;
                            let pixels_size = (texture.width as usize) * (texture.height as usize) * 4;
                            let mut pixels: Vec<u8> = vec![0; pixels_size];

                            f.seek(SeekFrom::Start(start_offset as u64))?;
                            if texture_format == 0 {
                                if texel_size == 2 {
                                    let mut encoded_pixels: Vec<u8> = vec![0; pixels_size / 2];
                                    f.read(&mut encoded_pixels).unwrap();

                                    let mut index1 = 0;
                                    let mut index3 = 0;

                                    for _ in 0..texture.height {
                                        for _ in 0..texture.width {
                                            let num2 = ((encoded_pixels[index3] as u16) << 8) + (encoded_pixels[index3+1] as u16);

                                            pixels[index1] = ((num2 & 0xF800) >> 8) as u8;
                                            pixels[index1+1] = ((num2 & 0x07C0) >> 3) as u8;
                                            pixels[index1+2] = ((num2 & 0x003E) << 2) as u8;
                                            pixels[index1+3] = if num2 & 1 == 1 { 255u8 } else { 0u8 };

                                            index3 += 2;
                                            index1 += 4;
                                        }
                                        if line_size > 0 { panic!("line size: {}", line_size); }
                                        //index3 += (uint) (this.lineSize * 4 - texture.textureWidth);
                                    }
                                } else if texel_size == 3 {
                                    // raw pixels, doesn't have a palette
                                    f.seek(SeekFrom::Current(-32))?;
                                    f.read(&mut pixels).unwrap();
                                } else {
                                    panic!("0/{}", texel_size);
                                }
                            } else if texture_format == 2 {
                                if texel_size == 0 {
                                    let mut encoded_pixels: Vec<u8> = vec![0; pixels_size / 2];
                                    f.read(&mut encoded_pixels).unwrap();

                                    let mut index1 = 0;
                                    let mut index7 = 0;
                                    let Some(palette) = &texture.palette else { todo!() };

                                    for _ in 0..texture.height {
                                        for _ in 0..(texture.width/2) {
                                            let index10 = (encoded_pixels[index7] >> 4) as usize;
                                            let index11 = (encoded_pixels[index7] & 0xF) as usize;

                                            let red10 = palette[index10 * 2] & 0xF8;
                                            let red11 = palette[index11 * 2] & 0xF8;
                                            let green10 = ((palette[index10 * 2] & 0x07) << 5) + ((palette[index10 * 2 + 1] & 0xC0) >> 3);
                                            let green11 = ((palette[index11 * 2] & 0x07) << 5) + ((palette[index11 * 2 + 1] & 0xC0) >> 3);
                                            let blue10 = (palette[index10 * 2 + 1] & 0x3E) << 2;
                                            let blue11 = (palette[index11 * 2 + 1] & 0x3E) << 2;
                                            let alpha10 = if palette[index10 * 2 + 1] & 1 == 1 { 255u8 } else { 0u8 };
                                            let alpha11 = if palette[index11 * 2 + 1] & 1 == 1 { 255u8 } else { 0u8 };

                                            pixels[index1] = red10;
                                            pixels[index1+1] = green10;
                                            pixels[index1+2] = blue10;
                                            pixels[index1+3] = alpha10;
                                            pixels[index1+4] = red11;
                                            pixels[index1+5] = green11;
                                            pixels[index1+6] = blue11;
                                            pixels[index1+7] = alpha11;

                                            index1 += 8;
                                            index7 += 1;
                                        }
                                    }
                                } else if texel_size == 1 {
                                    panic!("2/1");
                                } else {
                                    panic!("2/{}", texel_size);
                                }
                            } else if texture_format == 3 {
                                if texel_size == 1 {
                                    let mut encoded_pixels: Vec<u8> = vec![0; pixels_size / 2];
                                    f.read(&mut encoded_pixels).unwrap();

                                    let mut index1 = 0;
                                    let mut index2 = 0;
                                    for _ in 0..texture.height {
                                        for _ in 0..texture.width {
                                            let num1 = (encoded_pixels[index2] >> 4) as u8;
                                            let num2 = (encoded_pixels[index2] & 0x0F) as u8;
                                            pixels[index1] = num1 * 17;
                                            pixels[index1+1] = num1 * 17;
                                            pixels[index1+2] = num1 * 17;
                                            pixels[index1+3] = num2 * 17;
                                            index1 += 4;
                                            index2 += 1;
                                        }

                                        if (line_size as u8) * 8 - texture.width > 0 {
                                            panic!("texture has padding");
                                        }
                                    }
                                } else if texel_size == 2 {
                                    panic!("3/2");
                                } else {
                                    panic!("3/{}", texel_size);
                                }
                            } else {
                                panic!("{}/{}", texture_format, texel_size);
                            }

                            if pixels.len() > 0 {
                                let mtlfullpath = format!("{}/{}", output_dir, mtlpath);
                                RgbaImage::from_raw(texture.width as u32, texture.height as u32, pixels).unwrap().save(&mtlfullpath).unwrap();
                                created_textures.insert(mtlpath);
                            } else {
                                panic!("pixels are empty!!");
                            }
                        }
                        writeln!(output, "usemtl material_{:04}", current_texture)?;
                        new_texture = false;
                    }

                    if texture_is_null {
                        writeln!(output, "usemtl material_null")?;
                        texture_is_null = false;
                    }
                },
                6 => {
                    writeln!(output, "g {:x}", index1)?;
                },
                177 => {
                    let index1 = (command.b1 / 2) as usize;
                    let index2 = (command.b2 / 2) as usize;
                    let index3 = (command.b3 / 2) as usize;
                    let index5 = (command.b5 / 2) as usize;
                    let index6 = (command.b6 / 2) as usize;
                    let index7 = (command.b7 / 2) as usize;

                    let v1 = &vertices[ cache[ index1 ] as usize ];
                    let v2 = &vertices[ cache[ index2 ] as usize ];
                    let v3 = &vertices[ cache[ index3 ] as usize ];
                    let v5 = &vertices[ cache[ index5 ] as usize ];
                    let v6 = &vertices[ cache[ index6 ] as usize ];
                    let v7 = &vertices[ cache[ index7 ] as usize ];
                    let tex = &textures[current_texture];

                    writeln!(output, "vt {} {}", (v1.uv.x as f32) * tex.wratio, (v1.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v2.uv.x as f32) * tex.wratio, (v2.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v3.uv.x as f32) * tex.wratio, (v3.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "f {}/{} {}/{} {}/{}", cache[index1]+1, vt_index, cache[index2]+1, vt_index+1, cache[index3]+1, vt_index+2)?;
                    vt_index += 3;

                    writeln!(output, "vt {} {}", (v5.uv.x as f32) * tex.wratio, (v5.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v6.uv.x as f32) * tex.wratio, (v6.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v7.uv.x as f32) * tex.wratio, (v7.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "f {}/{} {}/{} {}/{}", cache[index5]+1, vt_index, cache[index6]+1, vt_index+1, cache[index7]+1, vt_index+2)?;
                    vt_index += 3;
                },
                182 => {
                    new_texture = false;
                    texture_is_null = true;
                },
                183 => { /* G_SETGEOMETRYMODE */ },
                184 => { /* G_ENDDL */ },
                186 => { /* G_SetOtherMode_H */ },
                187 => {
                    sscale = ((command.value >> 16) as f32) / 65536.0;
                    tscale = ((command.value & 0xFFFF) as f32) / 65536.0;
                },
                191 => {
                    let index5 = (command.b5 / 2) as usize;
                    let index6 = (command.b6 / 2) as usize;
                    let index7 = (command.b7 / 2) as usize;

                    let v5 = &vertices[ cache[ index5 ] as usize ];
                    let v6 = &vertices[ cache[ index6 ] as usize ];
                    let v7 = &vertices[ cache[ index7 ] as usize ];
                    let tex = &textures[current_texture];

                    writeln!(output, "vt {} {}", (v5.uv.x as f32) * tex.wratio, (v5.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v6.uv.x as f32) * tex.wratio, (v6.uv.y as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v7.uv.x as f32) * tex.wratio, (v7.uv.y as f32) * tex.hratio * -1.0)?;

                    writeln!(output, "f {}/{} {}/{} {}/{}", cache[index5]+1, vt_index, cache[index6]+1, vt_index+1, cache[index7]+1, vt_index+2)?;
                    vt_index += 3;
                },
                230 => { /* G_RDPLOADSYNC */ },
                231 => { /* G_RDPPIPESYNC */ },
                240 => {
                    let pal_size = ((command.value & 0xFFF000) >> 14) * 2 + 2;
                    //assert_eq!(pal_size, 32); // don't handle other sizes

                    let mut cur_tex = &mut textures[current_texture];
                    let mut palette = vec![];
                    for index1 in 0..pal_size {
                        let texture_offset = cur_tex.offset + 64 + index1;
                        f.seek(SeekFrom::Start((texture_offset + tex_count as u32 * 16) as u64))?;
                        palette.push(f.read_u8()?);
                    }
                    cur_tex.palette = Some(palette);

                    if commands[index1 + 4].id == 186 {
                        new_texture = true;
                        panic!("186");
                    }
                },
                242 => { /* G_SETTILESIZE */ },
                243 => { /* G_LOADBLOCK */ },
                245 => {
                    let num1 = command.value;
                    let num2 = ((command.b1 as u32) << 16) + ((command.b2 as u32) << 8) + (command.b3 as u32);

                    texture_format = command.b1 >> 5;
                    texel_size = (command.b1 >> 3) & 0b11;
                    line_size = (num2 >> 9) & 0xF;
                },
                252 => { /* G_SETCOMBINE */ },
                253 => {
                    let num = command.value & 0x00FFFFFF;

                    current_texture = 0xFFFFFFFF;
                    for index in 0..tex_count {
                        if textures[index as usize].offset == num || textures[index as usize].offset + 32 == num {
                            current_texture = index as usize;
                            break;
                        }
                    }

                    if current_texture != 0xFFFFFFFF {
                        textures[current_texture].set_ratio(sscale, tscale);
                    } else {
                        //println!("TEXTURE NOT FOUND!!!");
                        current_texture = 0;
                    }

                    new_texture = true;
                    texture_is_null = false;
                },
                _ => panic!("UNKNOWN COMMAND: {:?}", command.id),
            };
        }

        Ok(())
    }

    pub fn read_bin(filename: &str) -> std::io::Result<Self> {
        let file_size = std::fs::metadata(filename).unwrap().len();
        let mut f = File::open(filename)?;
        let header = f.read_u32::<BigEndian>()?; assert_eq!(header, 0x0B);

        let geometry_offset = f.read_u32::<BigEndian>()?;
        let texture_setup_offset = f.read_u16::<BigEndian>()?;
        let geometry_type = f.read_u16::<BigEndian>()?;
        let display_list_setup_offset = f.read_u32::<BigEndian>()?;
        let vertex_store_setup_offset = f.read_u32::<BigEndian>()?;
        let unk14_offset = f.read_u32::<BigEndian>()?;
        let animation_setup = f.read_u32::<BigEndian>()?;
        let collision_setup = f.read_u32::<BigEndian>()?;
        let unk20 = f.read_u32::<BigEndian>()?;
        let effects_setup = f.read_u32::<BigEndian>()?;
        let unk28 = f.read_u32::<BigEndian>()?;
        let animated_textures_offset = f.read_u32::<BigEndian>()?;

        // order not sure
        println!("===============================");
        println!("unk20 {:#X}", unk20);

        println!("texture_setup_offset {:#X}", texture_setup_offset);
        println!("display_list_setup_offset {:#X}", display_list_setup_offset);
        println!("vertex_store_setup_offset {:#X}", vertex_store_setup_offset);
        println!("unk14_offset {:#X}", unk14_offset);
        println!("collision_setup {:#X}", collision_setup);
        println!("effects_setup {:#X}", effects_setup);
        println!("unk28 {:#X}", unk28);
        println!("animation_setup {:#X}", animation_setup);
        println!("animated_textures_offset {:#X}", animated_textures_offset);
        println!("geometry_offset {:#X}", geometry_offset);
        println!("===============================\n");
        
        let unk = f.read_u16::<BigEndian>()?;
        println!("unk (vertices_count): {}", unk);
        let vertices_count = f.read_u16::<BigEndian>()?;
        
        let unk2 = f.read_f32::<BigEndian>()?; // scale?

        // TEXTURES
        assert!(texture_setup_offset != 0);
        assert_eq!(texture_setup_offset as u64, f.seek(SeekFrom::Current(0))?);

        let bytes_count = f.read_u32::<BigEndian>()?;
        let textures_count = f.read_u16::<BigEndian>()?;
        let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);

        let mut textures: Vec<Texture> = vec![];
        for i in 0..textures_count {
            let mut tex = Texture::new();

            tex.offset = f.read_u32::<BigEndian>()?;
            let format = f.read_u16::<BigEndian>()?;
            tex.format = match format {
                1 => TextureFormat::C4,
                2 => TextureFormat::C8,
                4 => TextureFormat::Rgba16,
                8 => TextureFormat::Rgba32,
                16 => TextureFormat::IA8,
                _ => panic!("Unknown texture format {}.", format),
            };
            tex.unknown = f.read_u16::<BigEndian>()?;
            tex.width = f.read_u8()?;
            tex.height = f.read_u8()?;
            let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);
            let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);

            if i > 0 {
                textures[(i - 1) as usize].size = tex.offset - textures[(i - 1) as usize].offset;
            }

            textures.push(tex);
        }
        if textures_count > 0 {
            let start = f.seek(SeekFrom::Current(0))? as u32;
            let i = textures.len();
            textures[(i - 1) as usize].size = display_list_setup_offset - start - textures[(i - 1) as usize].offset;
        }

        let start = f.seek(SeekFrom::Current(0))? as u32;
        for texture in &textures {
            assert_eq!(texture.offset + start, f.seek(SeekFrom::Current(0))? as u32);
            f.seek(SeekFrom::Current(texture.size as i64))?;
        }

        // DISPLAY LIST
        assert!(display_list_setup_offset != 0);
        assert_eq!(display_list_setup_offset as u64, f.seek(SeekFrom::Current(0))?);

        let commands_count = f.read_u32::<BigEndian>()?;
        let unk = f.read_u32::<BigEndian>()?;
        println!("unk (commands count): 0x{:X}", unk);

        let mut commands = vec![];

        let mut debug_prev_pos = f.seek(SeekFrom::Current(0))? - 8;

        for _ in 0..commands_count {
            // check I read 8 bytes each time
            assert_eq!(debug_prev_pos + 8, f.seek(SeekFrom::Current(0))?);
            debug_prev_pos = f.seek(SeekFrom::Current(0))?;

            let cmd = f.read_u8()?;

            let command = match cmd {
                0x00 => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);
                    let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);
                    
                    F3dex::SPNoOp
                },
                0x04 => {
                    let index = (f.read_u8()? as u16) * 2;
                    let data = f.read_u16::<BigEndian>()?;
                    let address = f.read_u32::<BigEndian>()?;

                    let count = (data >> 10) as u8;
                    let size = data & 0x3FF;
                    assert_eq!((count as u16) * 0x10 - 1, size);

                    F3dex::Vertex { index, count, address }
                },
                0x06 => {
                    let store_ra = f.read_u8()? != 0;
                    let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);
                    let address = f.read_u32::<BigEndian>()?;

                    F3dex::DisplayList { store_ra, address }
                },
                0xB1 => {
                    let v1 = f.read_u8()? / 2;
                    let v2 = f.read_u8()? / 2;
                    let v3 = f.read_u8()? / 2;
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let v4 = f.read_u8()? / 2;
                    let v5 = f.read_u8()? / 2;
                    let v6 = f.read_u8()? / 2;

                    F3dex::Triangle2 { v1, v2, v3, v4, v5, v6 }
                },
                0xB6 => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let flags = f.read_u32::<BigEndian>()?;

                    F3dex::ClearGeometryMode(flags)
                },
                0xB7 => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let flags = f.read_u32::<BigEndian>()?;

                    F3dex::SetGeometryMode(flags)
                },
                0xB8 => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);
                    let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);

                    F3dex::EndDisplayList
                },
                0xB9 => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let amount = f.read_u8()?;
                    let count = f.read_u8()?;
                    let mode = f.read_u32::<BigEndian>()?;

                    // TODO
                    F3dex::SetOtherModeL { amount, count, mode }
                },
                0xBA => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let amount = f.read_u8()?;
                    let count = f.read_u8()?;
                    let mode = f.read_u32::<BigEndian>()?;

                    // TODO
                    F3dex::SetOtherModeH { amount, count, mode }
                },
                0xBB => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let flags = f.read_u8()?;
                    let enable = f.read_u8()? != 0;
                    let frac_x = f.read_u16::<BigEndian>()?;
                    let frac_y = f.read_u16::<BigEndian>()?;

                    let mipmaps = flags >> 3;
                    let descriptor = flags & 0b111;
                    let scalex = (frac_x as f32) / (0xFFFF as f32);
                    let scaley = (frac_y as f32) / (0xFFFF as f32);

                    F3dex::Texture { mipmaps, descriptor, enable, scalex, scaley }
                },
                0xBD => {
                    let unk1 = f.read_u8()?;
                    let unk2 = f.read_u8()?;
                    let unk3 = f.read_u8()?;
                    let count = f.read_u32::<BigEndian>()?;

                    F3dex::PopMatrix { unk1, unk2, unk3, count }
                },
                0xBF => {
                    let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);
                    let v1 = f.read_u8()? / 2;
                    let v2 = f.read_u8()? / 2;
                    let v3 = f.read_u8()? / 2;

                    F3dex::Triangle1 { v1, v2, v3 }
                },
                0xE6 => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);
                    let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);
                    
                    F3dex::RdpLoadSync
                },
                0xE7 => {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);
                    let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);
                    
                    F3dex::RdpPipeSync
                },
                0xF0 => {
                    let descriptor = f.read_u32::<BigEndian>()?; assert_eq!(descriptor & 0xFFFFFFF0, 0);
                    let colour_count = f.read_u16::<BigEndian>()?;
                    let padding = f.read_u8()?; assert_eq!(padding, 0);

                    let descriptor = descriptor as u8;
                    let colour_count = (((colour_count >> 4) - 1) & 0x3FF) * 4;

                    F3dex::LoadTlut { descriptor }
                },
                0xF2 => {
                    let s = f.read_u8()? as u16;
                    let st = f.read_u16::<BigEndian>()?;
                    let descriptor = f.read_u8()?;
                    let w = f.read_u8()? as u16;
                    let wh = f.read_u16::<BigEndian>()?;

                    let upper_left_s = (s << 4) + (st >> 12);
                    let upper_left_t = st & 0x0FFF;
                    let width = ((w << 4) + (wh >> 12)) / 4 + 1;
                    let height = (wh & 0x0FFF) / 4 + 1;

                    F3dex::SetTileSize { upper_left_s, upper_left_t, descriptor, width, height }
                },
                0xF3 => {
                    let s = f.read_u8()? as u16;
                    let st = f.read_u16::<BigEndian>()?;
                    let descriptor = f.read_u8()?;
                    let t = f.read_u8()? as u16;
                    let td = f.read_u16::<BigEndian>()?;

                    let upper_left_s = (s << 4) + (st >> 12);
                    let upper_left_t = st & 0x0FFF;
                    let texels_count = (t << 4) + (td >> 12);
                    let dxt = td & 0x0FFF;

                    F3dex::LoadBlock { upper_left_s, upper_left_t, descriptor, texels_count, dxt }
                },
                0xF4 => {
                    let s = f.read_u8()? as u16;
                    let st = f.read_u16::<BigEndian>()?;
                    let descriptor = f.read_u8()?;
                    let w = f.read_u8()? as u16;
                    let wh = f.read_u16::<BigEndian>()?;

                    let upper_left_s = (s << 4) + (st >> 12);
                    let upper_left_t = st & 0x0FFF;
                    let lower_right_s = ((w << 4) + (wh >> 12)) / 4 + 1;
                    let lower_right_t = (wh & 0x0FFF) / 4 + 1;

                    F3dex::LoadTile { upper_left_s, upper_left_t, descriptor, lower_right_s, lower_right_t }
                },
                0xF5 => {
                    let b1 = f.read_u8()?;
                    let b2 = f.read_u8()?;
                    let b3 = f.read_u8()?;
                    let b4 = f.read_u8()?;
                    let b5 = f.read_u8()?;
                    let b6 = f.read_u8()?;
                    let b7 = f.read_u8()?;

                    let format = match b1 >> 5 {
                        0 => ColourFormat::Rgba,
                        1 => ColourFormat::Yuv,
                        2 => ColourFormat::Palette,
                        3 => ColourFormat::GrayscaleAlpha,
                        4 => ColourFormat::Grayscale,
                        _ => panic!("Unknown texture format."),
                    };
                    let depth = 4 * 2u8.pow(((b1 >> 3) & 0b11) as u32);
                    let values_per_row = (((b1 & 0b11) << 7) as u16) + ((b2 >> 1) as u16);
                    let tmem_offset = (((b2 & 0b1) as u16) << 8) + (b3 as u16);
                    let descriptor = b4; assert_eq!(b4 & 0xF8, 0);
                    let palette = b5 >> 4;
                    let clamp_mirror_y = (b5 >> 2) & 0b11;
                    let unwrapped_y = ((b5 << 2) + (b6 >> 6)) & 0b1111;
                    let perspective_div_y = (b6 >> 2) & 0b1111;
                    let clamp_mirror_x = b6 & 0b11;
                    let unwrapped_x = b7 >> 4;
                    let perspective_div_x = b7 & 0b1111;

                    F3dex::SetTile { format, depth, values_per_row, tmem_offset, descriptor,
                        palette, clamp_mirror_y, unwrapped_y, perspective_div_y,
                        clamp_mirror_x, unwrapped_x, perspective_div_x }
                },
                0xFC => {
                    let todo = f.read_u8()?;
                    let todo = f.read_u16::<BigEndian>()?;
                    let todo = f.read_u32::<BigEndian>()?;

                    F3dex::SetCombine {}
                },
                0xFD => {
                    let flags = f.read_u8()?;
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                    let unk = f.read_u8()?;
                    let address = f.read_u32::<BigEndian>()?;

                    let format = match flags >> 5 {
                        0 => ColourFormat::Rgba,
                        1 => ColourFormat::Yuv,
                        2 => ColourFormat::Palette,
                        3 => ColourFormat::GrayscaleAlpha,
                        4 => ColourFormat::Grayscale,
                        _ => panic!("Unknown texture format."),
                    };

                    let depth = 4 * 2u8.pow(((flags >> 3) & 0b11) as u32);

                    F3dex::SettImg { format, depth, address }
                },
                _ => panic!("Unknown F3DEX command 0x{:X}", cmd),
            };

            commands.push(command);
        }

        // VERTEX STORE
        assert!(vertex_store_setup_offset != 0);
        assert_eq!(vertex_store_setup_offset as u64, f.seek(SeekFrom::Current(0))?);

        let neg_x = f.read_i16::<BigEndian>()?;
        let neg_y = f.read_i16::<BigEndian>()?;
        let neg_z = f.read_i16::<BigEndian>()?;
        let pos_x = f.read_i16::<BigEndian>()?;
        let pos_y = f.read_i16::<BigEndian>()?;
        let pos_z = f.read_i16::<BigEndian>()?;

        let coord_range1 = f.read_i16::<BigEndian>()?;
        let coord_range2 = f.read_i16::<BigEndian>()?;

        let coll_range1 = f.read_i16::<BigEndian>()?;
        let coll_range2 = f.read_i16::<BigEndian>()?;

        let vertices_count_2 = f.read_u16::<BigEndian>()?;
        assert_eq!(vertices_count, vertices_count_2);

        let unk = f.read_u16::<BigEndian>()?;

        let mut vertices: Vec<Vertex> = vec![];
        for _ in 0..vertices_count {
            let mut vertex = Vertex::new();

            vertex.position.x = f.read_i16::<BigEndian>()?;
            vertex.position.y = f.read_i16::<BigEndian>()?;
            vertex.position.z = f.read_i16::<BigEndian>()?;
            let padding = f.read_u16::<BigEndian>()?;
            assert_eq!(padding, 0);
            vertex.uv.x = f.read_i16::<BigEndian>()?;
            vertex.uv.y = f.read_i16::<BigEndian>()?;
            vertex.r = (f.read_u8()? as f32) / 256.0;
            vertex.g = (f.read_u8()? as f32) / 256.0;
            vertex.b = (f.read_u8()? as f32) / 256.0;
            vertex.a = (f.read_u8()? as f32) / 256.0;

            vertices.push(vertex);
        }

        let mut unk14 = ModelUnk14::new();

        if unk14_offset > 0 {
            assert_eq!(unk14_offset as u64, f.seek(SeekFrom::Current(0))?);

            let unk14_0_count = f.read_u16::<BigEndian>()?;
            let unk14_1_count = f.read_u16::<BigEndian>()?;
            let unk14_2_count = f.read_u16::<BigEndian>()?;
            let unk14_3 = f.read_u16::<BigEndian>()?;

            for _ in 0..unk14_0_count {
                let unk1 = read_3_i16(&mut f);
                let unk2 = read_3_i16(&mut f);
                let unk3 = read_3_i16(&mut f);
                let unk4 = read_3_u8(&mut f);
                let unk5 = f.read_u8()?;
                let padding = f.read_u8()?;
                let padding = f.read_u8()?;

                unk14.unk14_0.push(ModelUnk14_0 {
                    unk1, unk2, unk3, unk4, unk5
                });
            }

            for _ in 0..unk14_1_count {
                let unk1 = f.read_u16::<BigEndian>()?;
                let unk2 = f.read_u16::<BigEndian>()?;
                let unk3 = read_3_i16(&mut f);
                let unk4 = read_3_u8(&mut f);
                let unk5 = f.read_u8()?;
                let unk6 = f.read_u8()?;
                let unk7 = f.read_u8()?;

                unk14.unk14_1.push(ModelUnk14_1 {
                    unk1, unk2, unk3, unk4, unk5, unk6
                });
            }

            for _ in 0..unk14_2_count {
                let unk1 = f.read_u16::<BigEndian>()?;
                let unk2 = read_3_i16(&mut f);
                let unk3 = f.read_u8()?;
                let unk4 = f.read_u8()?;
                let padding = f.read_u16::<BigEndian>()?;
            }

            // 8 bit alignment?
            let alignment = 8 - (f.seek(SeekFrom::Current(0))? % 8);
            if alignment < 8 {
                for _ in 0..alignment {
                    let padding = f.read_u8()?;
                }
            }
        }

        let mut collisions = Collisions::new();
        if collision_setup > 0 {
            assert_eq!(collision_setup as u64, f.seek(SeekFrom::Current(0))?);

            let min_x = f.read_i16::<BigEndian>()?;
            let min_y = f.read_i16::<BigEndian>()?;
            let min_z = f.read_i16::<BigEndian>()?;
            let max_x = f.read_i16::<BigEndian>()?;
            let max_y = f.read_i16::<BigEndian>()?;
            let max_z = f.read_i16::<BigEndian>()?;

            collisions.min = Vector3 { x: min_x, y: min_y, z: min_z };
            collisions.max = Vector3 { x: max_x, y: max_y, z: max_z };

            let y_stride = f.read_i16::<BigEndian>()?;
            let z_stride = f.read_i16::<BigEndian>()?;

            collisions.stride = Vector2 { x: y_stride, y: z_stride };

            let geo_count = f.read_i16::<BigEndian>()?;
            collisions.scale = f.read_u16::<BigEndian>()?;
            let tri_count = f.read_i16::<BigEndian>()?;
            let padding = f.read_i16::<BigEndian>()?; assert_eq!(padding, 0);

            for _ in 0..geo_count {
                let start_tri_index = f.read_u16::<BigEndian>()?;
                let tri_count = f.read_u16::<BigEndian>()?;

                collisions.geo.push(GeoColl {
                    start_tri_index,
                    tri_count,
                });
            }

            for _ in 0..tri_count {
                let vtx_indx_1 = f.read_u16::<BigEndian>()?;
                let vtx_indx_2 = f.read_u16::<BigEndian>()?;
                let vtx_indx_3 = f.read_u16::<BigEndian>()?;
                let unk = f.read_u16::<BigEndian>()?;
                let flags = f.read_u32::<BigEndian>()?;

                collisions.tri.push(TriColl {
                    vtx_indx_1,
                    vtx_indx_2,
                    vtx_indx_3,
                    unk,
                    flags,
                });
            }

            // maybe 8 bytes alignments?
            let alignment = 8 - (f.seek(SeekFrom::Current(0))? % 8);
            if alignment < 8 {
                for _ in 0..alignment {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                }
            }
        }

        let mut mesh_list = vec![];
        if effects_setup > 0 {
            assert_eq!(effects_setup as u64, f.seek(SeekFrom::Current(0))?);

            let mesh_count = f.read_u16::<BigEndian>()?;

            for _ in 0..mesh_count {
                let id = f.read_u16::<BigEndian>()?;
                let vtx_count = f.read_u16::<BigEndian>()?;

                let mut vertices = vec![];
                for _ in 0..vtx_count {
                    let vtx = f.read_u16::<BigEndian>()?;
                    vertices.push(vtx);
                }

                mesh_list.push(Mesh {
                    id,
                    vertices,
                });
            }

            // maybe 8 bytes alignments?
            let alignment = 8 - (f.seek(SeekFrom::Current(0))? % 8);
            if alignment < 8 {
                for _ in 0..alignment {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                }
            }
        }

        if unk28 > 0 {
            assert_eq!(unk28 as u64, f.seek(SeekFrom::Current(0))?);

            let count = f.read_u16::<BigEndian>()?;
            let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);

            for _ in 0..count {
                let coord = read_3_i16(&mut f);
                let anim_index = f.read_u8()?;
                let vtx_count = f.read_u8()?;
                let mut vtx_list = vec![];

                for _ in 0..vtx_count {
                    let vtx_id = f.read_u16::<BigEndian>()?;
                    vtx_list.push(vtx_id);
                }
            }

            // maybe 8 bytes alignments?
            let alignment = 8 - (f.seek(SeekFrom::Current(0))? % 8);
            if alignment < 8 {
                for _ in 0..alignment {
                    let padding = f.read_u8()?; assert_eq!(padding, 0);
                }
            }
        }

        // before or after unk20?
        if animation_setup > 0 {
            assert_eq!(animation_setup as u64, f.seek(SeekFrom::Current(0))?);

            let unk1 = f.read_f32::<BigEndian>()?;
            let count = f.read_u16::<BigEndian>()?;
            let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);

            for _ in 0..count {
                let unk11 = f.read_f32::<BigEndian>()?;
                let unk12 = f.read_f32::<BigEndian>()?;
                let unk13 = f.read_f32::<BigEndian>()?;
                let u1 = f.read_i16::<BigEndian>()?;
                let u2 = f.read_i16::<BigEndian>()?;
            }

        }

        let mut unknown20 = Unknown20::new();
        if unk20 > 0 {
            assert_eq!(unk20 as u64, f.seek(SeekFrom::Current(0))?);

            let count = f.read_u8()?;
            unknown20.unk1 = f.read_u8()?;
            for _ in 0..count {
                unknown20.unk2 = f.read_i16::<BigEndian>()?;
                unknown20.unk3 = f.read_i16::<BigEndian>()?;
                unknown20.unk4 = f.read_i16::<BigEndian>()?;
                unknown20.unk5 = f.read_i16::<BigEndian>()?;
                unknown20.unk6 = f.read_i16::<BigEndian>()?;
                unknown20.unk7 = f.read_i16::<BigEndian>()?;
                unknown20.unk8 = f.read_u8()?;
                unknown20.unk9 = f.read_u8()?;
            }

            let alignment = 8 - (f.seek(SeekFrom::Current(0))? % 8);
            if alignment < 8 {
                for _ in 0..alignment {
                    let padding = f.read_u8()?;
                }
            }
        }

        if animated_textures_offset > 0 {
            assert_eq!(animated_textures_offset as u64, f.seek(SeekFrom::Current(0))?);

            for _ in 0..4 {
                let size = f.read_u16::<BigEndian>()?;
                let count = f.read_u16::<BigEndian>()?;
                let rate = f.read_f32::<BigEndian>()?;
            }
        }

        assert_eq!(geometry_offset as u64, f.seek(SeekFrom::Current(0))?);
        let geometry = read_geometry_layout(&mut f)?;

        Ok(Self {
            textures,
            commands,
            vertices,
            collisions,
            geometry,
            unk14,
            unk20: unknown20,
            unk28,
            mesh_list,
            geometry_type,
        })
    }

    pub fn read_yaml(filename: &str) -> Option<Self> {
        let f = File::open(filename).expect(&format!("Can't open {}", filename));
        let ret: Result<Self, serde_yaml::Error> = serde_yaml::from_reader(f);
        match ret {
            Ok(file) => Some(file),
            Err(_) => None,
        }
    }

    pub fn write_bin(&self, filename: &str) -> std::io::Result<()> {
        Ok(())
    }

    pub fn write_yaml(&self, filename: &str) {
        let f = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(filename).unwrap();
        serde_yaml::to_writer(f, &self).unwrap();
    }

    pub fn write_obj(&self, filename: &str) -> std::io::Result<()> {
        Ok(())
    }
}

fn read_geometry_layout_command(f: &mut File) -> std::io::Result<Geometry> {
    let file_size = f.metadata().unwrap().len();
    let offset = f.seek(SeekFrom::Current(0))?;
    let geocode = f.read_u32::<BigEndian>()?;

    let geocmd = match geocode {
        0x0 => {
            let len = f.read_u32::<BigEndian>()?;
            let unk1 = f.read_u16::<BigEndian>()?;
            let unk2 = f.read_u16::<BigEndian>()?;
            let unk3 = read_3_floats(f);

            Geometry::Unknown0x00 { len: 0 }
        },
        0x1 => {
            let size = f.read_u32::<BigEndian>()?;
            let x1 = f.read_f32::<BigEndian>()?;
            let y1 = f.read_f32::<BigEndian>()?;
            let z1 = f.read_f32::<BigEndian>()?;
            let x2 = f.read_f32::<BigEndian>()?;
            let y2 = f.read_f32::<BigEndian>()?;
            let z2 = f.read_f32::<BigEndian>()?;
            let draw_only_nearest = f.read_u16::<BigEndian>()? > 0;
            let offset2 = f.read_u16::<BigEndian>()?;
            let len = f.read_u32::<BigEndian>()?;

            Geometry::Sort { x1, y1, z1, x2, y2, z2, draw_only_nearest, offset2, len }
        },
        0x2 => {
            let address = f.read_u32::<BigEndian>()? as u16;
            let len = f.read_u8()?;
            let id = f.read_u8()?;
            let unk = f.read_u16::<BigEndian>()?;

            // only the last one doesn't have "padding"
            if f.seek(SeekFrom::Current(0))? < file_size {
                let padding = f.read_u32::<BigEndian>()?;
            }

            Geometry::Bone { address, len, id, unk }
        },
        0x3 => {
            let len = f.read_u32::<BigEndian>()?;
            let offset = f.read_u16::<BigEndian>()?;
            let tri_count = f.read_u16::<BigEndian>()?;

            // only the last one doesn't have "padding"
            if f.seek(SeekFrom::Current(0))? < file_size {
                let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);
            }

            Geometry::LoadDisplayList { len, offset, tri_count }
        },
        0x5 => {
            let len = f.read_u32::<BigEndian>()?;
            
            for _ in 0..8 {
                if f.seek(SeekFrom::Current(0))? < file_size {
                    f.read_u16::<BigEndian>()?;
                }
            }

            Geometry::Skinning
        },
        0x8 => {
            let layout_offset = f.read_u32::<BigEndian>()?;
            let max_dist = f.read_f32::<BigEndian>()?;
            let min_dist = f.read_f32::<BigEndian>()?;
            let test_x = f.read_f32::<BigEndian>()?;
            let test_y = f.read_f32::<BigEndian>()?;
            let test_z = f.read_f32::<BigEndian>()?;
            let len = f.read_u32::<BigEndian>()?; assert_eq!(len, 0x20);

            Geometry::Lod { layout_offset, max_dist, min_dist, test_x, test_y, test_z }
        },
        0xA => {
            let len = f.read_u32::<BigEndian>()?;
            let index = f.read_u16::<BigEndian>()?;
            let bone = f.read_u16::<BigEndian>()?;
            let x = f.read_f32::<BigEndian>()?;
            let y = f.read_f32::<BigEndian>()?;
            let z = f.read_f32::<BigEndian>()?;

            Geometry::ReferencePoint { index, bone, x, y, z }
        },
        0xC => {
            let len = f.read_u32::<BigEndian>()?;
            let child_count = f.read_u16::<BigEndian>()?;
            let selector = f.read_u16::<BigEndian>()?;

            let mut indices = vec![];

            for _ in 0..child_count {
                let index = f.read_i32::<BigEndian>()?;
                indices.push(index);
            }

            // there are no good way to detect that data so
            // looking for the next command is done for now
            let mut last_word_read = f.read_u32::<BigEndian>()?;
            while last_word_read != 2 && last_word_read != 3 {
                last_word_read = f.read_u32::<BigEndian>()?;
            }

            f.seek(SeekFrom::Current(-4))?;

            let mut commands = vec![];

            Geometry::Selector { selector, indices, commands }
        },
        0xD => {
            let unk1 = f.read_u32::<BigEndian>()?;
            let min_x = f.read_i16::<BigEndian>()?;
            let min_y = f.read_i16::<BigEndian>()?;
            let min_z = f.read_i16::<BigEndian>()?;
            let max_x = f.read_i16::<BigEndian>()?;
            let max_y = f.read_i16::<BigEndian>()?;
            let max_z = f.read_i16::<BigEndian>()?;
            let len = f.read_u16::<BigEndian>()?;
            let unk2 = f.read_u16::<BigEndian>()?;

            if unk1 == 0x28 {
                read_geometry_layout_command(f)?;
            }

            Geometry::DrawDistance { len, min_x, min_y, min_z, max_x, max_y, max_z, unk1, unk2 }
        },
        0xE => {
            let len = f.read_u32::<BigEndian>()?;
            let vec1 = read_3_i16(f);
            let vec2 = read_3_i16(f);
            let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);

            let mut curr = f.seek(SeekFrom::Current(0))?;
            while curr < offset + (len as u64) {
                let command = read_geometry_layout_command(f);
                curr = f.seek(SeekFrom::Current(0))?;
            }

            Geometry::Unknown0x0e
        },
        0xF => {
            let len = f.read_u32::<BigEndian>()?;

            let mut header = vec![];
            let header_size = f.read_i16::<BigEndian>()?;
            let count = f.read_u8()?;
            let unk = f.read_u8()?;
            for _ in 12..header_size {
                let unk = f.read_u8()?;
            }

            let mut commands = vec![];
            if len > 0 {
                while f.seek(SeekFrom::Current(0))? < offset + (len as u64)  {
                    let geocmd = read_geometry_layout_command(f)?;
                    commands.push(geocmd);
                }
            }

            Geometry::Group0x0f { header, commands }
        },
        0x10 => {
            let len = f.read_u32::<BigEndian>()?;
            let unk1 = f.read_u32::<BigEndian>()?;
            let unk2 = 0; //f.read_u32::<BigEndian>()?;

            // only the last one doesn't have "padding"
            if f.seek(SeekFrom::Current(0))? < file_size {
                let padding = f.read_u32::<BigEndian>()?; assert_eq!(padding, 0);
            }

            Geometry::Unknown0x10 { len, unk1, unk2 }
        },
        _ => panic!("Unknown geometry command 0x{:X} at offset 0x{:X}", geocode, f.seek(SeekFrom::Current(0))?),
    };

    Ok(geocmd)
}

fn read_geometry_layout(f: &mut File) -> std::io::Result<Vec<Geometry>> {
    let file_size = f.metadata().unwrap().len();

    let mut geometry = vec![];
    while f.seek(SeekFrom::Current(0))? < file_size {
        let geocmd = read_geometry_layout_command(f)?;

        geometry.push(geocmd);
    }

    Ok(geometry)
}
