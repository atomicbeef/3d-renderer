use std::{
    borrow::Cow,
    fs::{self},
    path::Path,
};

use serde::Deserialize;

use crate::{color::Color, mesh::Mesh, texture::Texture, vector::Vec3};

use super::Object;

#[derive(Deserialize)]
struct SceneObject {
    mesh_path: String,
    texture_path: String,
    translation_x: f32,
    translation_y: f32,
    translation_z: f32,
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
}

pub enum SceneDeserializeError<'a> {
    ReadError(Cow<'a, str>),
    JsonError(serde_json::Error),
}

impl std::fmt::Display for SceneDeserializeError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(path) => {
                write!(f, "could not open scene file for reading at {path}",)
            }
            Self::JsonError(err) => {
                write!(f, "{err}")
            }
        }
    }
}

pub fn read_objects_from_scene(path: &Path) -> Result<Vec<Object>, SceneDeserializeError> {
    let json = fs::read_to_string(path)
        .or_else(|_| Err(SceneDeserializeError::ReadError(path.to_string_lossy())))?;

    let serialized_scene: Vec<SceneObject> =
        serde_json::from_str(&json).map_err(|e| SceneDeserializeError::JsonError(e))?;

    let mut objects = Vec::new();

    for scene_object in serialized_scene.iter() {
        let mesh_path = Path::new("assets/").join(&scene_object.mesh_path);
        let mut mesh = Mesh::from_obj(&mesh_path);

        mesh.translation = Vec3::new(
            scene_object.translation_x,
            scene_object.translation_y,
            scene_object.translation_z,
        );
        mesh.rotation = Vec3::new(
            scene_object.rotation_x,
            scene_object.rotation_y,
            scene_object.rotation_z,
        );
        mesh.scale = Vec3::new(
            scene_object.scale_x,
            scene_object.scale_y,
            scene_object.scale_z,
        );

        let texture_path = Path::new("assets/").join(&scene_object.texture_path);
        let texture = Texture::from_png(&texture_path).unwrap_or_else(|err| {
            eprintln!("Error reading texture: {err}");
            Texture::from_color(1, 1, Color::new(0xFF, 0x00, 0xFF))
        });

        objects.push(Object { mesh, texture });
    }

    Ok(objects)
}
