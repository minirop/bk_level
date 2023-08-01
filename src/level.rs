#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use std::collections::HashSet;
use image::RgbaImage;
use std::env::args;
use byteorder::{ReadBytesExt, BigEndian};
use clap::Parser;
use serde::{ Serialize, Deserialize };
use std::path::Path;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
struct Vertex {
    x: i16,
    y: i16,
    z: i16,
    unk: i16,
    u: i16,
    v: i16,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Vertex {
    fn new() -> Vertex {
        Vertex {
            x: 0, y: 0, z: 0,
            unk: 0, u: 0, v: 0,
            r: 0.0, g: 0.0, b: 0.0, a: 0.0,
        }
    }
}

struct Texture {
    offset: u32,
    unknown: u32,
    width: u8,
    height: u8,
    padding1: u16,
    padding2: u32,
    hratio: f32,
    wratio: f32,
    palette: [u8; 32]
}

impl Texture {
    fn new() -> Texture {
        Texture {
            offset: 0,
            unknown: 0,
            width: 0,
            height: 0,
            padding1: 0,
            padding2: 0,
            hratio: 1.0,
            wratio: 1.0,
            palette: [0; 32],
        }
    }

    fn set_ratio(&mut self, sscale: f32, tscale: f32) {
        self.wratio = sscale / ((self.width as f32) * 32.0);
        self.hratio = tscale / ((self.height as f32) * 32.0);
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

pub struct Level {
}

impl Level {
    pub fn read_bin(filename: &str) -> std::io::Result<Level> {
        let file_size = std::fs::metadata(filename).unwrap().len();
        let mut f = File::open(filename)?;
        let header = f.read_u32::<BigEndian>()?;
        assert_eq!(header, 0x0B);

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

            vert.x = f.read_i16::<BigEndian>()?;
            vert.y = f.read_i16::<BigEndian>()?;
            vert.z = f.read_i16::<BigEndian>()?;
            vert.unk = f.read_i16::<BigEndian>()?;
            vert.u = f.read_i16::<BigEndian>()?;
            vert.v = f.read_i16::<BigEndian>()?;
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
            tex.unknown = f.read_u32::<BigEndian>()?;
            tex.width = f.read_u8()?;
            tex.height = f.read_u8()?;
            tex.padding1 = f.read_u16::<BigEndian>()?;
            tex.padding2 = f.read_u32::<BigEndian>()?;

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

        let mut output_mtl = File::create("output.mtl").expect("Unable to create file");
        for id in 0..tex_count {
            let mtlname = format!("material_{:04}", id);
            writeln!(output_mtl, "newmtl {}", mtlname)?;
            writeln!(output_mtl, "map_Kd image_{:04}.png\n", id)?;
        }
        writeln!(output_mtl, "newmtl material_null")?;

        let mut output = File::create("output.obj").expect("Unable to create file");

        writeln!(output, "mtllib output.mtl")?;

        for v in &vertices {
            writeln!(output, "v {} {} {}", v.x, v.y, v.z)?;
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
                                    let palette = &texture.palette;

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
                                RgbaImage::from_raw(texture.width as u32, texture.height as u32, pixels).unwrap().save(&mtlpath).unwrap();
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

                    let v1 = vertices[ cache[ index1 ] as usize ];
                    let v2 = vertices[ cache[ index2 ] as usize ];
                    let v3 = vertices[ cache[ index3 ] as usize ];
                    let v5 = vertices[ cache[ index5 ] as usize ];
                    let v6 = vertices[ cache[ index6 ] as usize ];
                    let v7 = vertices[ cache[ index7 ] as usize ];
                    let tex = &textures[current_texture];

                    writeln!(output, "vt {} {}", (v1.u as f32) * tex.wratio, (v1.v as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v2.u as f32) * tex.wratio, (v2.v as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v3.u as f32) * tex.wratio, (v3.v as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "f {}/{} {}/{} {}/{}", cache[index1]+1, vt_index, cache[index2]+1, vt_index+1, cache[index3]+1, vt_index+2)?;
                    vt_index += 3;

                    writeln!(output, "vt {} {}", (v5.u as f32) * tex.wratio, (v5.v as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v6.u as f32) * tex.wratio, (v6.v as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v7.u as f32) * tex.wratio, (v7.v as f32) * tex.hratio * -1.0)?;
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

                    let v5 = vertices[ cache[ index5 ] as usize ];
                    let v6 = vertices[ cache[ index6 ] as usize ];
                    let v7 = vertices[ cache[ index7 ] as usize ];
                    let tex = &textures[current_texture];

                    writeln!(output, "vt {} {}", (v5.u as f32) * tex.wratio, (v5.v as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v6.u as f32) * tex.wratio, (v6.v as f32) * tex.hratio * -1.0)?;
                    writeln!(output, "vt {} {}", (v7.u as f32) * tex.wratio, (v7.v as f32) * tex.hratio * -1.0)?;

                    writeln!(output, "f {}/{} {}/{} {}/{}", cache[index5]+1, vt_index, cache[index6]+1, vt_index+1, cache[index7]+1, vt_index+2)?;
                    vt_index += 3;
                },
                230 => { /* G_RDPLOADSYNC */ },
                231 => { /* G_RDPPIPESYNC */ },
                240 => {
                    let pal_size = ((command.value & 0xFFF000) >> 14) * 2 + 2;
                    assert_eq!(pal_size, 32); // don't handle other sizes

                    let mut cur_tex = &mut textures[current_texture];
                    for index1 in 0..pal_size {
                        let texture_offset = cur_tex.offset + 64 + index1;
                        f.seek(SeekFrom::Start((texture_offset + tex_count as u32 * 16) as u64))?;
                        cur_tex.palette[index1 as usize] = f.read_u8()?;
                    }

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

                    if current_texture == 0xFFFFFFFF {
                        panic!("ERROR");
                    }

                    textures[current_texture].set_ratio(sscale, tscale);

                    new_texture = true;
                    texture_is_null = false;
                },
                _ => panic!("UNKNOWN COMMAND: {:?}", command.id),
            };
        }

        Ok(Level {})
    }

    pub fn write_yaml(&self, filename: &str) {

    }
}
