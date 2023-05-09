use std::env::args;

use winit::event_loop::EventLoop;
use winit::event_loop::ControlFlow;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event::ElementState;

use lenses::vulkan::VulkanState;
use lenses::world::World;
use lenses::THE_BOX;
use lenses::light::Light;

use lenses::lenses::Lens;
use lenses::world::Material;

use cgmath::Vector4;
use cgmath::Vector3;

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
struct FileFormat {
    lenses: Vec<Lens>,
    lights: Vec<Light>
}

fn main() {
    let event_loop = EventLoop::new();
    let mut vulkan = VulkanState::new(&event_loop);

    // Load the scene
    let fname = args().nth(1).unwrap();
    let s = std::fs::read_to_string(fname).unwrap();

    let scene_file: FileFormat = serde_yaml::from_str(&s).unwrap();
    let mut world = World::new();

    // Add box
    let box_model = world.add_model(THE_BOX.to_vec());
    world.add_entity(
        box_model,
        Vector3::new(-2.5, -1.0, -2.5),
        Material::Solid,
        Vector3::new(1.0,  1.0, 1.0),
        Vector4::new(0.2, 0.2, 0.2, 1.0)
    );

    for light in scene_file.lights {
        world.add_light(light);
    }

    for lens in scene_file.lenses {
        let tris = lens.tesselate();
        let model = world.add_model(tris);
        world.add_entity(
            model,
            Vector3::new(0.0, -0.1, 0.0),
            Material::Glass(1.3),
            Vector3::new(1.0,  1.0, 1.0),
            Vector4::new(0.209, 0.282, 0.686, 0.4),
        );
    }

    // Build kdtree
    world.build_kdtree();

    // Run ray tracer
    world.trace();

    // upload geometry
    world.upload_models(&mut vulkan);

    // Render scene
    let mut mouse_pressed = false;
    let mut mouse_pos = (0.0, 0.0);

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match ev {
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. }, ..
            } => {
                let zoom = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                world.zoom(-zoom*0.3);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, .. },
                ..
            } => {
                *&mut mouse_pressed = state == ElementState::Pressed;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                if mouse_pressed {
                    let lng_d = (mouse_pos.0 - position.x) / 500.0;
                    let lat_d = (mouse_pos.1 - position.y) / 500.0;

                    world.rotate(lat_d as f32, lng_d as f32, 0.0);
                }
                *&mut mouse_pos = (position.x, position.y);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_), ..
            } => {
                vulkan.recreate_swapchain = true;
            }
            Event::MainEventsCleared => {
                world.draw(&mut vulkan);
            }
            _ => {}
        }
    });
}
