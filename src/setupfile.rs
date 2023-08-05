#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use crate::types::read_3_u32;
use crate::types::write_3_u32;
use crate::types::read_2_floats;
use crate::types::read_3_floats;
use crate::types::write_3_floats;
use crate::types::write_2_floats;
use std::collections::HashSet;
use image::RgbaImage;
use std::env::args;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use clap::Parser;
use serde::{ Serialize, Deserialize };
use std::path::Path;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Write;

use crate::types::{ Vector3, Vector2 };

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SetupFile {
    pub cameras: Vec<Camera>,
    pub voxels: Vec<Voxel>,
    pub lightings: Vec<Lighting>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Camera {
    Type0 { id: u16 },
    Type1 { id: u16, position: Vector3<f32>, speed: Vector2<f32>, rot_acc: Vector2<f32>, angles: Vector3<f32>, unk: u32 },
    Type2 { id: u16, position: Vector3<f32>, angles: Vector3<f32> },
    Type3 { id: u16, position: Vector3<f32>, speed: Vector2<f32>, rot_acc: Vector2<f32>, angles: Vector3<f32>, unk: u32, distances: Vector2<f32> },
    Type4 { id: u16, unk: u32 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum SmallObject {
    Sprite { object: u16, size: u16, x: u16, y: u16, z: u16, unk1: u8, unk2: u8 },
    Static { object: u16, y_rot: u8, xz_rot: u8, x: u16, y: u16, z: u16, size: u8, unk: u8 },
    Unknown { object: u16, unk1: u8, unk2: u8, unk3: u8, unk4: u8, unk5: u8, unk6: u8, unk7: u8, unk8: u8, unk9: u8, unk10: u8 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ComplexObject {
    Actor { x: u16, y: u16, z: u16,
        script: u16, object: u16, unk_0a: u8,
        unk_0b: u8, rotation: u16, unk_0d: u8,
        size: u16, current: u16, next: u16,
        end_indicator: u8 },
    Timed { x: u16, y: u16, z: u16,
        script: u16, object: u16, unk_0a: u8,
        unk_0b: u8, timer: u8, unk_0d: u8,
        unk_0e: u8, unk_0f: u8,
        current: u16, next: u16, end_indicator: u8 },
    Script { x: u16, y: u16, z: u16,
        script: u16, object: u16, unk_0a: u8,
        unk_0b: u8, unk_0c: u8, unk_0d: u8,
        unk_0e: u8, unk_0f: u8,
        current: u16, next: u16, end_indicator: u8 },
    Radius { x: u16, y: u16, z: u16,
        radius: u16, object: u8, associated: u16,
        unk_0a: u8, unk_0b: u8, unk_0c: u8,
        unk_0d: u8, unk_0e: u8, unk_0f: u8,
        current: u16, next: u16,
        end_indicator: u8 },
    Unknown { x: u16, y: u16, z: u16,
        script: u16, object: u16, unk_0a: u8,
        unk_0b: u8, unk_0c: u8, unk_0d: u8,
        unk_0e: u8, unk_0f: u8,
        current: u16, next: u16, end_indicator: u8 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Voxel {
    pub position: Vector3<i32>,
    pub complex_objects: Vec<ComplexObject>,
    pub small_objects: Vec<SmallObject>,
    pub missing: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Lighting {
    position: Vector3<f32>,
    unk: Vector2<f32>,
    colours: Vector3<u32>,
}

fn write_voxel(f: &mut File, voxel: &Voxel) -> std::io::Result<()> {
    let has_complex = voxel.complex_objects.len() > 0;
    let has_small = voxel.small_objects.len() > 0;

    if !voxel.missing {
        f.write_u8(3)?;
        f.write_u8(10)?;
        f.write_u8(voxel.complex_objects.len() as u8)?;

        if has_complex {
            f.write_u8(11)?;

            let mut nx: u16 = 0;
            let mut ny: u16 = 0;
            let mut nz: u16 = 0;
            let mut nscript: u16 = 0;
            let mut nobject: u16 = 0;
            let mut nunk_0a: u8 = 0;
            let mut nunk_0b: u8 = 0;
            let mut nunk_0c: u8 = 0;
            let mut nunk_0d: u8 = 0;
            let mut nunk_0e: u8 = 0;
            let mut nunk_0f: u8 = 0;
            let mut ncurrent: u16 = 0;
            let mut nnext: u16 = 0;
            let mut nend: u8 = 0;

            for complex in &voxel.complex_objects {
                match complex {
                    ComplexObject::Actor { x, y, z, script, object, unk_0a, unk_0b, rotation, unk_0d, size, current, next, end_indicator } => {
                        nx = *x;
                        ny = *y;
                        nz = *z;
                        nscript = *script;
                        nobject = *object;
                        nunk_0a = *unk_0a;
                        nunk_0b = *unk_0b;
                        nunk_0c = (*rotation / 2) as u8;
                        nunk_0d = *unk_0d;
                        nunk_0e = (*size >> 8) as u8;
                        nunk_0f = (*size & 0xFF) as u8;
                        ncurrent = *current;
                        nnext = *next;
                        nend = *end_indicator;
                    },
                    ComplexObject::Timed { x, y, z, script, object, unk_0a, unk_0b, timer, unk_0d, unk_0e, unk_0f, current, next, end_indicator } => {
                        nx = *x;
                        ny = *y;
                        nz = *z;
                        nscript = *script;
                        nobject = *object;
                        nunk_0a = *unk_0a;
                        nunk_0b = *unk_0b;
                        nunk_0c = *timer;
                        nunk_0d = *unk_0d;
                        nunk_0e = *unk_0e;
                        nunk_0f = *unk_0f;
                        ncurrent = *current;
                        nnext = *next;
                        nend = *end_indicator;
                    },
                    ComplexObject::Script { x, y, z, script, object, unk_0a, unk_0b, unk_0c, unk_0d, unk_0e, unk_0f, current, next, end_indicator } => {
                        nx = *x;
                        ny = *y;
                        nz = *z;
                        nscript = *script;
                        nobject = *object;
                        nunk_0a = *unk_0a;
                        nunk_0b = *unk_0b;
                        nunk_0c = *unk_0c;
                        nunk_0d = *unk_0d;
                        nunk_0e = *unk_0e;
                        nunk_0f = *unk_0f;
                        ncurrent = *current;
                        nnext = *next;
                        nend = *end_indicator;
                    },
                    ComplexObject::Radius { x, y, z, radius, object, associated, unk_0a, unk_0b, unk_0c, unk_0d, unk_0e, unk_0f, current, next, end_indicator } => {
                        nx = *x;
                        ny = *y;
                        nz = *z;
                        nscript = ((*radius / 2) << 8) + (*object as u16);
                        nobject = *associated;
                        nunk_0a = *unk_0a;
                        nunk_0b = *unk_0b;
                        nunk_0c = *unk_0c;
                        nunk_0d = *unk_0d;
                        nunk_0e = *unk_0e;
                        nunk_0f = *unk_0f;
                        ncurrent = *current;
                        nnext = *next;
                        nend = *end_indicator;
                    },
                    ComplexObject::Unknown { x, y, z, script, object, unk_0a, unk_0b, unk_0c, unk_0d, unk_0e, unk_0f, current, next, end_indicator } => {
                        nx = *x;
                        ny = *y;
                        nz = *z;
                        nscript = *script;
                        nobject = *object;
                        nunk_0a = *unk_0a;
                        nunk_0b = *unk_0b;
                        nunk_0c = *unk_0c;
                        nunk_0d = *unk_0d;
                        nunk_0e = *unk_0e;
                        nunk_0f = *unk_0f;
                        ncurrent = *current;
                        nnext = *next;
                        nend = *end_indicator;
                    },
                };

                let nc = (ncurrent >> 4) as u8;
                let ncn = (ncurrent << 12) + nnext;

                f.write_u16::<BigEndian>(nx)?;
                f.write_u16::<BigEndian>(ny)?;
                f.write_u16::<BigEndian>(nz)?;
                f.write_u16::<BigEndian>(nscript)?;
                f.write_u16::<BigEndian>(nobject)?;
                f.write_u8(nunk_0a)?;
                f.write_u8(nunk_0b)?;
                f.write_u8(nunk_0c)?;
                f.write_u8(nunk_0d)?;
                f.write_u8(nunk_0e)?;
                f.write_u8(nunk_0f)?;
                f.write_u8(nc)?;
                f.write_u16::<BigEndian>(ncn)?;
                f.write_u8(nend)?;
            }
        }

        f.write_u8(8)?;
        f.write_u8(voxel.small_objects.len() as u8)?;

        if voxel.small_objects.len() > 0 {
            f.write_u8(9)?;

            for small in &voxel.small_objects {
                match small {
                    SmallObject::Sprite { object, size, x, y, z, unk1, unk2 } => {
                        f.write_u16::<BigEndian>(*object)?;
                        f.write_u16::<BigEndian>(*size)?;
                        f.write_u16::<BigEndian>(*x)?;
                        f.write_u16::<BigEndian>(*y)?;
                        f.write_u16::<BigEndian>(*z)?;
                        f.write_u8(*unk1)?;
                        f.write_u8(*unk2)?;
                    },
                    SmallObject::Static { object, y_rot, xz_rot, x, y, z, size, unk } => {
                        f.write_u16::<BigEndian>(*object)?;
                        f.write_u8(*y_rot)?;
                        f.write_u8(*xz_rot)?;
                        f.write_u16::<BigEndian>(*x)?;
                        f.write_u16::<BigEndian>(*y)?;
                        f.write_u16::<BigEndian>(*z)?;
                        f.write_u8(*size)?;
                        f.write_u8(*unk)?;
                    },
                    SmallObject::Unknown { object, unk1, unk2, unk3, unk4, unk5, unk6, unk7, unk8, unk9, unk10 } => {
                        f.write_u16::<BigEndian>(*object)?;
                        f.write_u8(*unk1)?;
                        f.write_u8(*unk2)?;
                        f.write_u8(*unk3)?;
                        f.write_u8(*unk4)?;
                        f.write_u8(*unk5)?;
                        f.write_u8(*unk6)?;
                        f.write_u8(*unk7)?;
                        f.write_u8(*unk8)?;
                        f.write_u8(*unk9)?;
                        f.write_u8(*unk10)?;
                    },
                }
            }
        }
    }

    Ok(())
}

fn write_camera(f: &mut File, camera: &Camera) -> std::io::Result<()> {
    match camera {
        Camera::Type0 { id } => {
            f.write_u16::<BigEndian>(*id)?;
            f.write_u8(2)?;
        },
        Camera::Type1 { id, position, speed, rot_acc, angles, unk } => {
            f.write_u16::<BigEndian>(*id)?;
            f.write_u8(2)?;
            f.write_u8(1)?;

            f.write_u8(1)?;
            write_3_floats(f, position);
            f.write_u8(2)?;
            write_2_floats(f, speed);
            f.write_u8(3)?;
            write_2_floats(f, rot_acc);
            f.write_u8(4)?;
            write_3_floats(f, angles);
            f.write_u8(5)?;
            f.write_u32::<BigEndian>(*unk)?;
        },
        Camera::Type2 { id, position, angles } => {
            f.write_u16::<BigEndian>(*id)?;
            f.write_u8(2)?;
            f.write_u8(2)?;

            f.write_u8(1)?;
            write_3_floats(f, position);
            f.write_u8(2)?;
            write_3_floats(f, angles);
        },
        Camera::Type3 { id, position, speed, rot_acc, angles, unk, distances } => {
            f.write_u16::<BigEndian>(*id)?;
            f.write_u8(2)?;
            f.write_u8(3)?;

            f.write_u8(1)?;
            write_3_floats(f, position);
            f.write_u8(2)?;
            write_2_floats(f, speed);
            f.write_u8(3)?;
            write_2_floats(f, rot_acc);
            f.write_u8(4)?;
            write_3_floats(f, angles);
            f.write_u8(5)?;
            f.write_u32::<BigEndian>(*unk)?;
            f.write_u8(6)?;
            write_2_floats(f, distances);
        },
        Camera::Type4 { id, unk } => {
            f.write_u16::<BigEndian>(*id)?;
            f.write_u8(2)?;
            f.write_u8(4)?;

            f.write_u8(1)?;
            f.write_u32::<BigEndian>(*unk)?;
        },
    }

    f.write_u8(0)?;

    Ok(())
}

// complex objects
const ACTORS_ID: [u16; 351] = [0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A, 0x000B, 0x000C, 0x000F, 0x0011, 0x0012, 0x001E, 0x0020, 0x0021, 0x0022, 0x0023, 0x0025, 0x0026, 0x0027, 0x0028, 0x0029, 0x002A, 0x002D, 0x002F, 0x0030, 0x0031, 0x003A, 0x003C, 0x003D, 0x003E, 0x0041, 0x0043, 0x0046, 0x0047, 0x0049, 0x0050, 0x0052, 0x0055, 0x0056, 0x0057, 0x005B, 0x005E, 0x005F, 0x0060, 0x0061, 0x0062, 0x0067, 0x0069, 0x0070, 0x0081, 0x0082, 0x0083, 0x0084, 0x0085, 0x0088, 0x008C, 0x00BC, 0x00C5, 0x00C6, 0x00C7, 0x00CA, 0x00CB, 0x00CC, 0x00CD, 0x00CE, 0x00D0, 0x00D1, 0x00D5, 0x00D7, 0x00E4, 0x00E6, 0x00E8, 0x00EE, 0x00EF, 0x00F0, 0x00F1, 0x00F2, 0x00F5, 0x00F6, 0x00F7, 0x00F9, 0x00FA, 0x00FB, 0x00FC, 0x00FD, 0x00FE, 0x00FF, 0x0100, 0x0101, 0x0102, 0x0109, 0x010A, 0x010C, 0x010D, 0x010E, 0x0110, 0x0111, 0x0114, 0x0115, 0x0116, 0x0117, 0x0119, 0x011A, 0x011B, 0x011C, 0x011D, 0x011E, 0x01F9, 0x0120, 0x0121, 0x0123, 0x0124, 0x0129, 0x012B, 0x012E, 0x0130, 0x0131, 0x0132, 0x0133, 0x0134, 0x0135, 0x0137, 0x0139, 0x013A, 0x013B, 0x013E, 0x013F, 0x0142, 0x0143, 0x0144, 0x0145, 0x0146, 0x0147, 0x014D, 0x014E, 0x0150, 0x0151, 0x0152, 0x0153, 0x015F, 0x0160, 0x0161, 0x0162, 0x0163, 0x0167, 0x0168, 0x0169, 0x016A, 0x016B, 0x016C, 0x016D, 0x016F, 0x0181, 0x0182, 0x0185, 0x018B, 0x018F, 0x0191, 0x0192, 0x0194, 0x019E, 0x01A3, 0x01A4, 0x01BF, 0x01C0, 0x01C1, 0x01C2, 0x01C3, 0x01C4, 0x01C6, 0x01C8, 0x01C9, 0x01CA, 0x01CC, 0x01CD, 0x01D8, 0x01D9, 0x01DA, 0x01E2, 0x01E9, 0x01E3, 0x01E4, 0x01EA, 0x01EB, 0x01EC, 0x01ED, 0x01EE, 0x01EF, 0x01F0, 0x01F1, 0x01F2, 0x01F3, 0x01F6, 0x01F7, 0x01FD, 0x01FE, 0x01FA, 0x01FB, 0x01FC, 0x0203, 0x0204, 0x0206, 0x0208, 0x020B, 0x020D, 0x020E, 0x020F, 0x0210, 0x0211, 0x0212, 0x0213, 0x0214, 0x0215, 0x0216, 0x0217, 0x0218, 0x0219, 0x021A, 0x021B, 0x021D, 0x0221, 0x0222, 0x0223, 0x0226, 0x0227, 0x0229, 0x022B, 0x022C, 0x0230, 0x0231, 0x0234, 0x0235, 0x0236, 0x0237, 0x0239, 0x023B, 0x023C, 0x023D, 0x023F, 0x0243, 0x0246, 0x0247, 0x0248, 0x0256, 0x0257, 0x025B, 0x025C, 0x025D, 0x025E, 0x0266, 0x0267, 0x0268, 0x027A, 0x027B, 0x027C, 0x027D, 0x027E, 0x027F, 0x0280, 0x0285, 0x0286, 0x0287, 0x0288, 0x0289, 0x028A, 0x0292, 0x0296, 0x0297, 0x0299, 0x029C, 0x029D, 0x029F, 0x02A1, 0x02A2, 0x02A4, 0x02A6, 0x02A7, 0x02A8, 0x02A9, 0x02AA, 0x02AB, 0x02AC, 0x02DB, 0x02DE, 0x02E2, 0x02E3, 0x02E4, 0x02E5, 0x02E7, 0x02E8, 0x02E9, 0x02EA, 0x02F4, 0x02F5, 0x030D, 0x030F, 0x0311, 0x0312, 0x0315, 0x031A, 0x031D, 0x033A, 0x033B, 0x033C, 0x033D, 0x033F, 0x0340, 0x0348, 0x034D, 0x034E, 0x034F, 0x0350, 0x0354, 0x0355, 0x0356, 0x0357, 0x0361, 0x0362, 0x0363, 0x0364, 0x0367, 0x0365, 0x0366, 0x0368, 0x0369, 0x036A, 0x036B, 0x036C, 0x036D, 0x036E, 0x036F, 0x0370, 0x0372, 0x0375, 0x037A, 0x037B, 0x037D, 0x037E, 0x037F, 0x0380, 0x0381, 0x0383, 0x0387, 0x038B, 0x03B7, 0x03BC, 0x03BF, 0x03C0, 0x03C1, 0x03C2];
const TIMERS_ID: [u16; 2] = [0x002C, 0x0065];
const SCRIPTS_ID: [u16; 50] = [0x0000, 0x0001, 0x0002, 0x0002, 0x0012, 0x0015, 0x0016, 0x0017, 0x0013, 0x0037, 0x0071, 0x0072, 0x0075, 0x0076, 0x0076, 0x0077, 0x0078, 0x0079, 0x007A, 0x007B, 0x007C, 0x007D, 0x007E, 0x007F, 0x0103, 0x0104, 0x0105, 0x0106, 0x0149, 0x014A, 0x016E, 0x01B0, 0x01CF, 0x0349, 0x0373, 0x0373, 0x0373, 0x0373, 0x0373, 0x0373, 0x0376, 0x0379, 0x0379, 0x0379, 0x0379, 0x03B9, 0x03BA, 0x03BD, 0x03BE, 0x03C3];
// small objects
const SPRITES_ID: [u16; 41] = [0x00E0, 0x00E7, 0x0380, 0x0387, 0x0460, 0x0465, 0x0467, 0x0470, 0x0477, 0x0500, 0x0540, 0x0541, 0x0544, 0x0550, 0x0551, 0x0552, 0x0554, 0x0555, 0x0556, 0x0940, 0x0970, 0x0D60, 0x1210, 0x13F0, 0x13F2, 0x1400, 0x1403, 0x1404, 0x1407, 0x1410, 0x1413, 0x1450, 0x15F0, 0x1600, 0x1610, 0x1620, 0x1630, 0x1640, 0x1650, 0x1657, 0x1660];
const STATICS_ID: [u16; 63] = [0x0022, 0x0090, 0x0092, 0x0093, 0x0097, 0x00A2, 0x00A4, 0x00B4, 0x00C0, 0x00C1, 0x00C2, 0x00C4, 0x00C5, 0x00C7, 0x00E5, 0x00E6, 0x00F0, 0x00F2, 0x00F6, 0x00F7, 0x0100, 0x0101, 0x0104, 0x0105, 0x0107, 0x010A, 0x0123, 0x0124, 0x0136, 0x0170, 0x0172, 0x0175, 0x01C4, 0x0206, 0x0260, 0x0263, 0x0264, 0x0267, 0x0270, 0x02A0, 0x02A1, 0x02A5, 0x02B0, 0x02B6, 0x02B7, 0x02E4, 0x02E5, 0x02E7, 0x0370, 0x0604, 0x0610, 0x0614, 0x0630, 0x0640, 0x0642, 0x0644, 0x0645, 0x0647, 0x06F2, 0x0706, 0x0712, 0x0762, 0x07A2];

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

        let mut cameras = vec![];
        let mut voxels = vec![];
        let mut lightings = vec![];

        let mut loc_x = negative_x_voxel_count;
        let mut loc_y = negative_y_voxel_count;
        let mut loc_z = negative_z_voxel_count;

        let file_size = std::fs::metadata(filename).unwrap().len();
        while f.seek(SeekFrom::Current(0))? < file_size {
            let header = f.read_u8()?;

            if (header == 3 || header == 1) && voxels.len() > 0 {
                assert!(loc_x <= positive_x_voxel_count);
                loc_z += 1;

                if loc_z > positive_z_voxel_count {
                    loc_z = negative_z_voxel_count;

                    loc_y += 1;
                    
                    if loc_y > positive_y_voxel_count {
                        loc_y = negative_y_voxel_count;

                        loc_x += 1;
                    }
                }
            }

            if header == 3 { // Start Of A Voxel With Objects
                let subheader = f.read_u8()?;
                assert_eq!(subheader, 10);

                let mut voxel = Voxel {
                    position: Vector3 { x: loc_x, y: loc_y, z: loc_z },
                    complex_objects: vec![],
                    small_objects: vec![],
                    missing: false,
                };

                let big_objects_count = f.read_u8()?;

                let mut list_type = f.read_u8()?;
                if list_type == 11 { // complex objects list
                    for i in 0..big_objects_count {
                        let x = f.read_u16::<BigEndian>()?;
                        let y = f.read_u16::<BigEndian>()?;
                        let z = f.read_u16::<BigEndian>()?;
                        let script = f.read_u16::<BigEndian>()?;
                        let object = f.read_u16::<BigEndian>()?;
                        let unk_0a = f.read_u8()?;
                        let unk_0b = f.read_u8()?;
                        let unk_0c = f.read_u8()?;
                        let unk_0d = f.read_u8()?;
                        let unk_0e = f.read_u8()?;
                        let unk_0f = f.read_u8()?;
                        let c = f.read_u8()? as u16;
                        let cn = f.read_u16::<BigEndian>()?;
                        let end_indicator = f.read_u8()?;

                        let current = (c << 4) + (cn >> 12);
                        let next = cn & 0x0FFF;

                        if ACTORS_ID.contains(&object) {
                            let rotation = (unk_0c as u16) * 2;
                            let size = ((unk_0e as u16) << 8) + (unk_0f as u16);

                            voxel.complex_objects.push(ComplexObject::Actor {
                                x, y, z,
                                script, object, unk_0a, unk_0b,
                                rotation, unk_0d, size,
                                current, next, end_indicator
                            });
                        } else if TIMERS_ID.contains(&object) {
                            let timer = unk_0c;

                            voxel.complex_objects.push(ComplexObject::Timed {
                                x, y, z,
                                script, object, unk_0a, unk_0b,
                                timer, unk_0d, unk_0e, unk_0f,
                                current, next, end_indicator
                            });
                        } else if SCRIPTS_ID.contains(&object) {
                            voxel.complex_objects.push(ComplexObject::Script {
                                x, y, z,
                                script, object, unk_0a, unk_0b,
                                unk_0c, unk_0d, unk_0e, unk_0f,
                                current, next, end_indicator
                            });
                        } else {
                            let associated = object;
                            let radius = (script >> 8) * 2;
                            let object = (script & 0xFF) as u8;
                            match object {
                                0x06 | 0x08 | 0x0E | 0x12 | 0x14 | 0x4C | 0x4D | 0x86 | 0x88 | 0x8E | 0x92 | 0x94 => {
                                    voxel.complex_objects.push(ComplexObject::Radius {
                                        x, y, z,
                                        radius, object, associated,
                                        unk_0a, unk_0b, unk_0c,
                                        unk_0d, unk_0e, unk_0f,
                                        current, next,
                                        end_indicator
                                    });
                                },
                                _ => {
                                    voxel.complex_objects.push(ComplexObject::Unknown {
                                        x, y, z,
                                        script, object: associated, unk_0a, unk_0b,
                                        unk_0c, unk_0d, unk_0e, unk_0f, current,
                                        next, end_indicator
                                    });
                                }
                            };
                        }
                    }

                    list_type = f.read_u8()?;
                }

                assert_eq!(list_type, 8);

                let small_objects_count = f.read_u8()?;

                if small_objects_count > 0 {
                    let list_type = f.read_u8()?;
                    assert_eq!(list_type, 9);

                    for _ in 0..small_objects_count {
                        let object = f.read_u16::<BigEndian>()?;

                        if SPRITES_ID.contains(&object) {
                            let size = f.read_u16::<BigEndian>()?;
                            let x = f.read_u16::<BigEndian>()?;
                            let y = f.read_u16::<BigEndian>()?;
                            let z = f.read_u16::<BigEndian>()?;
                            let unk1 = f.read_u8()?;
                            let unk2 = f.read_u8()?;

                            voxel.small_objects.push(SmallObject::Sprite { object, size, x, y, z, unk1, unk2 });
                        } else if STATICS_ID.contains(&object) {
                            let y_rot = f.read_u8()?;
                            let xz_rot = f.read_u8()?;
                            let x = f.read_u16::<BigEndian>()?;
                            let y = f.read_u16::<BigEndian>()?;
                            let z = f.read_u16::<BigEndian>()?;
                            let size = f.read_u8()?;
                            let unk = f.read_u8()?;

                            voxel.small_objects.push(SmallObject::Static { object, y_rot, xz_rot, x, y, z, size, unk });
                        } else {
                            let unk1 = f.read_u8()?;
                            let unk2 = f.read_u8()?;
                            let unk3 = f.read_u8()?;
                            let unk4 = f.read_u8()?;
                            let unk5 = f.read_u8()?;
                            let unk6 = f.read_u8()?;
                            let unk7 = f.read_u8()?;
                            let unk8 = f.read_u8()?;
                            let unk9 = f.read_u8()?;
                            let unk10 = f.read_u8()?;

                            voxel.small_objects.push(SmallObject::Unknown {
                                object, unk1, unk2, unk3, unk4, unk5,
                                unk6, unk7, unk8, unk9, unk10 });
                        }
                    }
                }

                voxels.push(voxel);

                let voxels_separator = f.read_u8()?;
                assert_eq!(voxels_separator, 1);
            } else if header == 0 {
                let subheader = f.read_u8()?;
                assert_eq!(subheader, 3);

                let mut start_of_camera = f.read_u8()?;
                while start_of_camera == 1 {
                    let id = f.read_u16::<BigEndian>()?;
                    let camera_two = f.read_u8()?;
                    assert_eq!(camera_two, 2);
                    let camera_type = f.read_u8()?;

                    let camera = match camera_type {
                        0 => {
                            Camera::Type0 { id }
                        },
                        1 | 3 => {
                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 1);
                            let position = read_3_floats(&mut f);

                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 2);
                            let speed = read_2_floats(&mut f);
                            
                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 3);
                            let rot_acc = read_2_floats(&mut f);
                            
                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 4);
                            let angles = read_3_floats(&mut f);
                            
                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 5);
                            let unk = f.read_u32::<BigEndian>()?;

                            if camera_type == 3 {
                                let section_id = f.read_u8()?;
                                assert_eq!(section_id, 6);
                                let distances = read_2_floats(&mut f);

                                Camera::Type3 { id, position, speed, rot_acc, angles, unk, distances }
                            } else {
                                Camera::Type1 { id, position, speed, rot_acc, angles, unk }
                            }
                        },
                        2 => {
                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 1);
                            let position = read_3_floats(&mut f);

                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 2);
                            let angles = read_3_floats(&mut f);

                            Camera::Type2 { id, position, angles }
                        },
                        4 => {
                            let section_id = f.read_u8()?;
                            assert_eq!(section_id, 1);
                            let unk = f.read_u32::<BigEndian>()?;

                            Camera::Type4 { id, unk }
                        },
                        _ => panic!("camera_type: {:?}", camera_type)
                    };

                    cameras.push(camera);

                    if camera_type != 0 {
                        let end_of_camera = f.read_u8()?;
                        assert_eq!(end_of_camera, 0);
                    }

                    start_of_camera = f.read_u8()?;
                }
                assert_eq!(start_of_camera, 0); // end of list
            } else if header == 4 {
                let mut first_section_id = f.read_u8()?;
                while first_section_id == 1 {
                    let section_id = f.read_u8()?;
                    assert_eq!(section_id, 2);
                    let position = read_3_floats(&mut f);

                    let section_id = f.read_u8()?;
                    assert_eq!(section_id, 3);
                    let unk = read_2_floats(&mut f);

                    let section_id = f.read_u8()?;
                    assert_eq!(section_id, 4);
                    let colours = read_3_u32(&mut f);

                    lightings.push(Lighting { position, unk, colours });

                    first_section_id = f.read_u8()?;
                }
                first_section_id = f.read_u8()?;
                assert_eq!(first_section_id, 0);
            } else if header == 1 {
                voxels.push(Voxel {
                    position: Vector3 { x: loc_x, y: loc_y, z: loc_z },
                    complex_objects: vec![],
                    small_objects: vec![],
                    missing: true,
                });
            } else {
                panic!("> header = 0x{:X}", header);
            }
        }

        Ok(SetupFile {
            cameras,
            voxels,
            lightings,
        })
    }

    pub fn read_yaml(filename: &str) -> Option<SetupFile> {
        let f = File::open(filename).expect(&format!("Can't open {}", filename));
        let ret: Result<SetupFile, serde_yaml::Error> = serde_yaml::from_reader(f);
        match ret {
            Ok(file) => Some(file),
            Err(_) => None,
        }
    }

    pub fn write_bin(&self, filename: &str) -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(filename).unwrap();

        let mut neg_x = 0;
        let mut neg_y = 0;
        let mut neg_z = 0;
        let mut pos_x = 0;
        let mut pos_y = 0;
        let mut pos_z = 0;

        for voxel in &self.voxels {
            neg_x = if voxel.position.x < neg_x { voxel.position.x } else { neg_x };
            neg_y = if voxel.position.y < neg_y { voxel.position.y } else { neg_y };
            neg_z = if voxel.position.z < neg_z { voxel.position.z } else { neg_z };
            pos_x = if voxel.position.x > pos_x { voxel.position.x } else { pos_x };
            pos_y = if voxel.position.y > pos_y { voxel.position.y } else { pos_y };
            pos_z = if voxel.position.z > pos_z { voxel.position.z } else { pos_z };
        }

        f.write_u16::<BigEndian>(0x0101)?;
        f.write_i32::<BigEndian>(neg_x)?;
        f.write_i32::<BigEndian>(neg_y)?;
        f.write_i32::<BigEndian>(neg_z)?;
        f.write_i32::<BigEndian>(pos_x)?;
        f.write_i32::<BigEndian>(pos_y)?;
        f.write_i32::<BigEndian>(pos_z)?;

        for voxel in &self.voxels {
            write_voxel(&mut f, voxel)?;
            f.write_u8(1)?;
        }
        f.write_u8(0)?;

        f.write_u8(3)?;
        for cam in &self.cameras {
            f.write_u8(1)?;
            write_camera(&mut f, cam)?;
        }
        f.write_u8(0)?;

        f.write_u8(4)?;
        for lighting in &self.lightings {
            f.write_u8(1)?;
            f.write_u8(2)?;
            write_3_floats(&mut f, &lighting.position);
            f.write_u8(3)?;
            write_2_floats(&mut f, &lighting.unk);
            f.write_u8(4)?;
            write_3_u32(&mut f, &lighting.colours);
        }
        f.write_u8(0)?;
        f.write_u8(0)?;

        Ok(())
    }

    pub fn write_yaml(&self, filename: &str) {
        let f = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(filename).unwrap();
        serde_yaml::to_writer(f, &self).unwrap();
    }
}
