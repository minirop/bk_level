#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unreachable_code)]

use std::collections::HashMap;
use crate::types::*;
use crate::gltf;
use byteorder::{ ReadBytesExt, BigEndian, WriteBytesExt, LittleEndian };
use serde::{ Serialize, Deserialize };
use std::path::Path;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Write;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Animation {
	start_frame: u16,
	end_frame: u16,
	sections: Vec<AnimationSection>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSection {
	bone: u16,
    transformation: Transformation,
	values: Vec<AnimationCommand>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Transformation {
    XRotation,
    YRotation,
    ZRotation,
    XScale,
    YScale,
    ZScale,
    XTranslation,
    YTranslation,
    ZTranslation,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationCommand {
    unknown: u8,
    frame: u16,
    factor: f32,
}

impl Animation {
    pub fn read_bin(filename: &str) -> std::io::Result<Self> {
        let mut f = File::open(filename)?;
        let start_frame = f.read_u16::<BigEndian>()?;
        let end_frame = f.read_u16::<BigEndian>()?;
        let section_count = f.read_u16::<BigEndian>()?;
        let padding = f.read_u16::<BigEndian>()?; assert_eq!(padding, 0);

        let mut sections = vec![];

        for _ in 0..section_count {
	        let f1 = f.read_u8()? as u16;
            let f2 = f.read_u8()? as u16;
            let bone = (f1 << 4) + (f2 >> 4);
            let transformation = match f2 & 0xF {
                0 => Transformation::XRotation,
                1 => Transformation::YRotation,
                2 => Transformation::ZRotation,
                3 => Transformation::XScale,
                4 => Transformation::YScale,
                5 => Transformation::ZScale,
                6 => Transformation::XTranslation,
                7 => Transformation::YTranslation,
                8 => Transformation::ZTranslation,
                _ => panic!("Unknown transformation: {:?}", f2 & 0xF),
            };
	        let value_count = f.read_u16::<BigEndian>()?;
        	let mut values = vec![];
	        for _ in 0..value_count {
                let frame = f.read_u16::<BigEndian>()?;
                let factor = (f.read_i16::<BigEndian>()? as f32) / 64.0;
                let unknown = (frame >> 14) as u8;
                let frame = frame & 0x3FFF;

	        	values.push(AnimationCommand {
                    unknown, frame, factor
                });
	        }

        	sections.push(AnimationSection {
        		bone,
                transformation,
        		values,
        	});
        }

    	Ok(Animation {
    		start_frame,
    		end_frame,
    		sections,
    	})
    }

    pub fn write_gltf(&self, output_dir: &str) {
        let mut root = gltf::Gltf {
            asset: gltf::Asset {
                version: "2.0".to_string(),
                generator: "bk_level".to_string(),
            },
            accessors: vec![],
            animations: vec![],
            buffers: vec![],
            buffer_views: vec![],
            images: vec![],
            materials: vec![],
            meshes: vec![],
            nodes: vec![],
            samplers: vec![],
            scenes: vec![],
            textures: vec![],
        };

        #[derive(Debug, Clone)]
        struct MergedAnimation {
            translation: Vector3<Option<f32>>,
            rotation: Vector3<Option<f32>>,
            scale: Vector3<Option<f32>>,
        }

        #[derive(Debug)]
        struct FramedMergedAnimation {
            frames: HashMap<u16, MergedAnimation>,
        }

        let mut merged_animations = HashMap::new();

        for section in &self.sections {
            let mut framed_merged_animation = if merged_animations.contains_key(&section.bone) {
                merged_animations.remove(&section.bone).unwrap()
            } else {
                FramedMergedAnimation {
                    frames: HashMap::new(),
                }
            };

            for cmd in &section.values {
                if !framed_merged_animation.frames.contains_key(&cmd.frame) {
                    framed_merged_animation.frames.insert(cmd.frame, MergedAnimation {
                        translation: Vector3 { x: None, y: None, z: None },
                        rotation: Vector3 { x: None, y: None, z: None },
                        scale: Vector3 { x: None, y: None, z: None },
                    });
                }

                match section.transformation {
                    Transformation::XRotation => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().rotation.x = Some(cmd.factor);
                    },
                    Transformation::YRotation => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().rotation.y = Some(cmd.factor);
                    },
                    Transformation::ZRotation => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().rotation.z = Some(cmd.factor);
                    },
                    Transformation::XScale => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().scale.x = Some(cmd.factor);
                    },
                    Transformation::YScale => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().scale.y = Some(cmd.factor);
                    },
                    Transformation::ZScale => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().scale.z = Some(cmd.factor);
                    },
                    Transformation::XTranslation => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().translation.x = Some(cmd.factor);
                    },
                    Transformation::YTranslation => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().translation.y = Some(cmd.factor);
                    },
                    Transformation::ZTranslation => {
                        framed_merged_animation.frames.get_mut(&cmd.frame).unwrap().translation.z = Some(cmd.factor);
                    },
                };
            }

            merged_animations.insert(section.bone, framed_merged_animation);
        }

        for (id, fm) in &mut merged_animations {
            let mut ct = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut cr = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut cs = Vector3 { x: 1.0, y: 1.0, z: 1.0 };

            let frames = fm.frames.clone();
            let mut keys: Vec<_> = frames.keys().collect();
            keys.sort_unstable();

            for k in keys {
                let mut f = fm.frames.get_mut(&k).unwrap();

                ct.x = if let Some(x) = f.translation.x {
                    x
                } else {
                    f.translation.x = Some(ct.x);
                    ct.x
                };
                ct.y = if let Some(y) = f.translation.y {
                    y
                } else {
                    f.translation.y = Some(ct.y);
                    ct.y
                };
                ct.z = if let Some(z) = f.translation.z {
                    z
                } else {
                    f.translation.z = Some(ct.z);
                    ct.z
                };

                cs.x = if let Some(x) = f.scale.x {
                    x
                } else {
                    f.scale.x = Some(cs.x);
                    cs.x
                };
                cs.y = if let Some(y) = f.scale.y {
                    y
                } else {
                    f.scale.y = Some(cs.y);
                    cs.y
                };
                cs.z = if let Some(z) = f.scale.z {
                    z
                } else {
                    f.scale.z = Some(cs.z);
                    cs.z
                };

                cr.x = if let Some(x) = f.rotation.x {
                    x
                } else {
                    f.rotation.x = Some(cr.x);
                    cr.x
                };
                cr.y = if let Some(y) = f.rotation.y {
                    y
                } else {
                    f.rotation.y = Some(cr.y);
                    cr.y
                };
                cr.z = if let Some(z) = f.rotation.z {
                    z
                } else {
                    f.rotation.z = Some(cr.z);
                    cr.z
                };
            }
        }

        let writer = File::create(format!("{}/anim.gltf", output_dir)).unwrap();
        serde_json::to_writer_pretty(writer, &root).unwrap();
    }

    pub fn read_yaml(filename: &str) -> Option<Self> {
        let f = File::open(filename).expect(&format!("Can't open {}", filename));
        let ret: Result<Self, serde_yaml::Error> = serde_yaml::from_reader(f);
        match ret {
            Ok(file) => Some(file),
            Err(_) => None,
        }
    }

    pub fn write_yaml(&self, filename: &str) {
        let f = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(filename).unwrap();
        serde_yaml::to_writer(f, &self).unwrap();
    }
}
