use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;
use std::{env, time::Instant};

use camera::Camera;
use color::Color;
use depth_buffer::DepthBuffer;
use matrix::Mat4;
use minifb::{Key, KeyRepeat, Window, WindowOptions};

mod camera;
mod color;
mod color_buffer;
mod depth_buffer;
mod drawing;
mod matrix;
mod mesh;
mod obj;
mod plane;
mod polygon;
mod render;
mod scene;
mod texture;
mod triangle;
mod vector;

use color_buffer::ColorBuffer;
use mesh::Mesh;
use render::{prepare_triangles, render, RenderMode, RenderSettings};
use scene::reader::read_objects_from_scene;
use scene::{Object, Scene};
use texture::Texture;
use triangle::Triangle;
use vector::{Vec2, Vec3};

const RENDER_WIDTH: usize = 512;
const RENDER_HEIGHT: usize = 384;
const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 768;

const FRAME_RATE: f32 = 60.0;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Error: Incorrect arguments specified!");
        println!("Usage: software-renderer [mesh or scene]");
        return ExitCode::from(1);
    }

    // Window setup
    let mut color_buffer = ColorBuffer::new(RENDER_WIDTH, RENDER_HEIGHT);
    let mut depth_buffer = DepthBuffer::new(RENDER_WIDTH, RENDER_HEIGHT);

    let mut window = Window::new(
        "3D Renderer",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions {
            resize: true,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..Default::default()
        },
    )
    .expect("Error: Window could not be created!");

    window.limit_update_rate(Some(Duration::from_secs_f32(1.0 / FRAME_RATE)));

    // Scene setup
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, -5.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.0,
        0.0,
        std::f32::consts::FRAC_PI_3,
        0.1,
        100.0,
    );

    let mut scene = Scene::new(camera);

    if args[1].ends_with(".obj") {
        // An OBJ file was specified
        let mesh_path = Path::new(&args[1]);
        let mesh = Mesh::from_obj(mesh_path);

        let texture_path = Path::new(&args[1]).with_extension("png");
        let texture = Texture::from_png(&texture_path).unwrap_or_else(|err| {
            eprintln!("Error reading texture: {err}");
            Texture::from_color(1, 1, Color::new(0xFF, 0x00, 0xFF))
        });

        scene.add_object(Object { mesh, texture });
    } else {
        // Assume a scene file was specified
        let scene_path = Path::new(&args[1]);
        let objects = match read_objects_from_scene(scene_path) {
            Ok(objects) => objects,
            Err(e) => {
                eprintln!("Error reading scene file: {e}");
                return ExitCode::from(1);
            }
        };

        for object in objects {
            scene.add_object(object);
        }
    }

    // Main loop preparation
    let mut triangles_to_render: Vec<Triangle> = Vec::new();

    let projection_matrix = Mat4::projection(
        std::f32::consts::FRAC_PI_2,
        RENDER_HEIGHT as f32 / RENDER_WIDTH as f32,
        0.1,
        100.0,
    );

    let mut render_settings = RenderSettings {
        render_mode: RenderMode::Textured,
        backface_cull: true,
        shaded: true,
        translate: false,
        rotate: true,
        rotation: Vec3::new(0.0, 0.01, 0.0),
        scale: false,
        flip_uvs_vertically: false,
        render_width: RENDER_WIDTH,
        render_height: RENDER_HEIGHT,
    };

    let start_time = Instant::now();
    let mut last_frame_time = start_time;

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Key1) {
            render_settings.render_mode = RenderMode::WireframeVertex;
        } else if window.is_key_down(Key::Key2) {
            render_settings.render_mode = RenderMode::Wireframe;
        } else if window.is_key_down(Key::Key3) {
            render_settings.render_mode = RenderMode::Filled;
        } else if window.is_key_down(Key::Key4) {
            render_settings.render_mode = RenderMode::WireframeFilled;
        } else if window.is_key_down(Key::Key5) {
            render_settings.render_mode = RenderMode::Textured;
        } else if window.is_key_down(Key::Key6) {
            render_settings.render_mode = RenderMode::WireframeTextured;
        }

        if window.is_key_pressed(Key::C, KeyRepeat::No) {
            render_settings.backface_cull = !render_settings.backface_cull;
        }

        if window.is_key_down(Key::L) {
            render_settings.shaded = true;
        } else if window.is_key_down(Key::U) {
            render_settings.shaded = false;
        }

        if window.is_key_pressed(Key::T, KeyRepeat::No) {
            render_settings.translate = !render_settings.translate;
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            render_settings.rotate = !render_settings.rotate;
        }
        if window.is_key_pressed(Key::G, KeyRepeat::No) {
            render_settings.scale = !render_settings.scale;
        }

        if window.is_key_pressed(Key::X, KeyRepeat::No) {
            render_settings.rotation.x = if render_settings.rotation.x > 0.0 {
                0.0
            } else {
                0.01
            };
        }
        if window.is_key_pressed(Key::Y, KeyRepeat::No) {
            render_settings.rotation.y = if render_settings.rotation.y > 0.0 {
                0.0
            } else {
                0.01
            };
        }
        if window.is_key_pressed(Key::Z, KeyRepeat::No) {
            render_settings.rotation.z = if render_settings.rotation.z > 0.0 {
                0.0
            } else {
                0.01
            };
        }

        if window.is_key_pressed(Key::F, KeyRepeat::No) {
            render_settings.flip_uvs_vertically = !render_settings.flip_uvs_vertically;
        }

        let delta_time = last_frame_time.elapsed().as_secs_f32();
        last_frame_time = Instant::now();

        scene.update(
            &render_settings,
            &mut window,
            start_time.elapsed().as_secs_f32(),
            delta_time,
        );

        color_buffer.draw_grid();

        for object in scene.objects() {
            prepare_triangles(
                &mut triangles_to_render,
                projection_matrix,
                &object.mesh,
                &scene.camera,
                &render_settings,
            );

            render(
                &mut color_buffer,
                &mut depth_buffer,
                &triangles_to_render,
                &render_settings,
                &object.texture,
            );
        }

        window
            .update_with_buffer(
                color_buffer.buffer(),
                color_buffer.width(),
                color_buffer.height(),
            )
            .unwrap();

        depth_buffer.clear(1.0);
        color_buffer.clear(Color::new(0, 0, 0));
    }

    return ExitCode::from(0);
}
