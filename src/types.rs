use serde::{ Serialize, Deserialize };

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
