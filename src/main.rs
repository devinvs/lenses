use winit::event_loop::EventLoop;
use winit::event_loop::ControlFlow;
use winit::event::Event;
use winit::event::WindowEvent;

use lenses::vulkan::VulkanState;
use lenses::{VERTICES, INDICES, NORMALS};

fn main() {
    let event_loop = EventLoop::new();
    let mut vulkan = VulkanState::new(&event_loop);

    // load teapot into vram
    vulkan.transfer_object_data(VERTICES, INDICES, NORMALS);

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(_), ..
            } => {
                vulkan.recreate_swapchain = true;
            }
            Event::MainEventsCleared => {
                vulkan.draw();
            }
            _ => {}
        }
    });
}
