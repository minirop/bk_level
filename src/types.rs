use byteorder::WriteBytesExt;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use serde::{ Serialize, Deserialize };
use std::fs::File;

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: std::default::Default> Vector3<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T: std::default::Default> Vector2<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

pub fn read_2_floats(f: &mut File) -> Vector2<f32> {
    let x = f.read_f32::<BigEndian>().unwrap();
    let y = f.read_f32::<BigEndian>().unwrap();
    Vector2 { x, y }
}

pub fn read_3_floats(f: &mut File) -> Vector3<f32> {
    let x = f.read_f32::<BigEndian>().unwrap();
    let y = f.read_f32::<BigEndian>().unwrap();
    let z = f.read_f32::<BigEndian>().unwrap();
    Vector3 { x, y, z }
}

pub fn read_3_u32(f: &mut File) -> Vector3<u32> {
    let x = f.read_u32::<BigEndian>().unwrap();
    let y = f.read_u32::<BigEndian>().unwrap();
    let z = f.read_u32::<BigEndian>().unwrap();
    Vector3 { x, y, z }
}

pub fn read_3_i16(f: &mut File) -> Vector3<i16> {
    let x = f.read_i16::<BigEndian>().unwrap();
    let y = f.read_i16::<BigEndian>().unwrap();
    let z = f.read_i16::<BigEndian>().unwrap();
    Vector3 { x, y, z }
}

pub fn read_2_i16(f: &mut File) -> Vector2<i16> {
    let x = f.read_i16::<BigEndian>().unwrap();
    let y = f.read_i16::<BigEndian>().unwrap();
    Vector2 { x, y }
}

pub fn read_3_u8(f: &mut File) -> Vector3<u8> {
    let x = f.read_u8().unwrap();
    let y = f.read_u8().unwrap();
    let z = f.read_u8().unwrap();
    Vector3 { x, y, z }
}

pub fn write_2_floats(f: &mut File, vec: &Vector2<f32>) {
    f.write_f32::<BigEndian>(vec.x).unwrap();
    f.write_f32::<BigEndian>(vec.y).unwrap();
}

pub fn write_3_floats(f: &mut File, vec: &Vector3<f32>) {
    f.write_f32::<BigEndian>(vec.x).unwrap();
    f.write_f32::<BigEndian>(vec.y).unwrap();
    f.write_f32::<BigEndian>(vec.z).unwrap();
}

pub fn write_3_i16(f: &mut File, vec: &Vector3<i16>) {
    f.write_i16::<BigEndian>(vec.x).unwrap();
    f.write_i16::<BigEndian>(vec.y).unwrap();
    f.write_i16::<BigEndian>(vec.z).unwrap();
}

pub fn write_2_i16(f: &mut File, vec: &Vector2<i16>) {
    f.write_i16::<BigEndian>(vec.x).unwrap();
    f.write_i16::<BigEndian>(vec.y).unwrap();
}

pub fn write_3_u32(f: &mut File, vec: &Vector3<u32>) {
    f.write_u32::<BigEndian>(vec.x).unwrap();
    f.write_u32::<BigEndian>(vec.y).unwrap();
    f.write_u32::<BigEndian>(vec.z).unwrap();
}

pub fn write_3_u8(f: &mut File, vec: &Vector3<u8>) {
    f.write_u8(vec.x).unwrap();
    f.write_u8(vec.y).unwrap();
    f.write_u8(vec.z).unwrap();
}
