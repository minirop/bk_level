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

fn s(memory: &Vec<u8>, left: usize, right: usize) -> i32 {
    return ((memory[left as usize] as i32) << 8) + (memory[right as usize] as i32);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SetupFile {
}

impl SetupFile {
    pub fn read_bin(filename: &str) -> std::io::Result<SetupFile> {
        let mut f = File::open(filename)?;
        let header = f.read_u16::<BigEndian>()?;
        let negative_x_voxel_count = f.read_i32::<BigEndian>()?;
        let negative_y_voxel_count = f.read_i32::<BigEndian>()?;
        let negative_z_voxel_count = f.read_i32::<BigEndian>()?;
        let positive_x_voxel_count = f.read_i32::<BigEndian>()?;
        let positive_y_voxel_count = f.read_i32::<BigEndian>()?;
        let positive_z_voxel_count = f.read_i32::<BigEndian>()?;
        let x_voxel_count = negative_x_voxel_count.abs() + positive_x_voxel_count + 1;
        let y_voxel_count = negative_y_voxel_count.abs() + positive_y_voxel_count + 1;
        let z_voxel_count = negative_z_voxel_count.abs() + positive_z_voxel_count + 1;

        let mut num10 = 0;
        let mut num11 = 0;
        let mut yz_voxel_count = y_voxel_count*z_voxel_count;
        let mut num13 = 0;
        let mut num14 = 0;
        let mut num15 = 0;
        let mut num16 = 0;
        let mut num17 = 0;

        let file_size = std::fs::metadata(filename).unwrap().len();
        let mut setupfile = vec![0; (file_size - 26) as usize];
        f.read(&mut setupfile).unwrap();

        let mut index = 0;
        let mut zone_id = 0;
        while zone_id < x_voxel_count*y_voxel_count*z_voxel_count {
            if index >= setupfile.len() { println!("break"); break; }

            if num13 > yz_voxel_count {
                num13 = 0;
                num11 += 1;
                num14 += 1;
                num15 = 0;
            }
            if num13 - num16 * z_voxel_count > z_voxel_count {
                if num16 < z_voxel_count {
                    num16 += 1;
                }
                if num16 >= z_voxel_count - 1 {
                    num16 = 0;
                    num15 += 1;
                }
            }

            if setupfile[index] == 3 && setupfile[index+1] == 10 { // Start Of A Voxel With Objects
                if setupfile[index+3] == 11 { // Start Of Complex Object List 
                    let num18 = setupfile[index+2];
                    let mut index2 = index + 4;

                    for _ in 0..num18 {
                        let value = ((setupfile[index2+6] as u16) << 8) + (setupfile[index2+7] as u16);
                        match value {
                            6 | 8 | 12 | 14 | 16 | 18 | 20 => {
                                let x = s(&setupfile, index2, index2+1);
                                let y = s(&setupfile, index2+2, index2+3);
                                let z = s(&setupfile, index2+4, index2+5);
                                let specialscript = s(&setupfile, index2+6, index2+7);
                                let object_id = s(&setupfile, index2+8, index2+9);
                                let unk10 = setupfile[index2+10];
                                let unk11 = setupfile[index2+11];
                                let rot = setupfile[index2+12] * 2;
                                let unk13 = setupfile[index2+13];
                                let size = s(&setupfile, index2+14, index2+15);
                                let uid = s(&setupfile, index2+16, index2+17);
                                let unk18 = setupfile[index2+18];
                                let unk19 = setupfile[index2+19];
                                let zone_id = value;
                            },
                            _ => {
                                let bytes: [u8;4] = [
                                    setupfile[index2+3],
                                    setupfile[index2+2],
                                    setupfile[index2+1],
                                    setupfile[index2]
                                ];
                                let val = f32::from_be_bytes(bytes);
                                let w1 = u32::from_be_bytes([ setupfile[index2+4], setupfile[index2+5], setupfile[index2+6], setupfile[index2+7]]);
                                let w2 = u32::from_be_bytes([ setupfile[index2+8], setupfile[index2+9], setupfile[index2+10], setupfile[index2+11]]);
                                let w3 = u32::from_be_bytes([ setupfile[index2+12], setupfile[index2+13], setupfile[index2+14], setupfile[index2+15]]);
                                let uid = s(&setupfile, index2+16, index2+17);
                                let unk18 = setupfile[index2+18];
                                let unk19 = setupfile[index2+19];

                                println!("w: 0x{:x}, 0x{:x}, 0x{:x}", w1, w2, w3);
                                println!("val: {}, uid: {}", val, uid);
                                let x = negative_x_voxel_count * 1000 + num14 * 1000 + 500;
                                let y = negative_y_voxel_count * 1000 + num15 * 1000 + 500;
                                let z = negative_z_voxel_count * 1000 + num16 * 1000 + 500;
                                println!("pos: {}, {}, {}", x, y, z);
                            },
                        };

                        index2 += 20;
                        zone_id += 1;
                    }

                    index = index + 4 + (num18 as usize) * 20;
                } else {
                    println!("03 0A {:X} {:X}", setupfile[index+2], setupfile[index+3]);
                    index += 3;
                }
            } else if setupfile[index] == 8 {
                if setupfile[index+2] == 9 {
                    panic!("LOL2");
                } else {
                    index += 2;
                    zone_id += 1;
                }
            } else {
                //println!("> 0x{:X}", setupfile[index]);
            }

            index += 1;
            num13 += 1;
        }

        for i in index..setupfile.len() {
            print!("0x{} ", setupfile[i]);
        }
        println!("");

        Ok(SetupFile {
        })
    }

    pub fn read_yaml(filename: &str) -> SetupFile {
        SetupFile {
        }
    }

    pub fn write_bin(&self, filename: &str) {

    }

    pub fn write_yaml(&self, filename: &str) {

    }
}
