// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

pub mod vulkan;
pub mod world;
pub mod geometry;
pub mod ply;
pub mod kdtree;
pub mod lenses;
pub mod light;

use bytemuck::{Pod, Zeroable};
use vulkano::impl_vertex;

use geometry::Triangle;
use cgmath::Vector3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 3],
}
impl_vertex!(Vertex, position);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Normal {
    normal: [f32; 3],
}
impl_vertex!(Normal, normal);

pub const THE_BOX: [Triangle; 12] = [
    // left wall
    Triangle {
        v0: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 0.0, y: 0.0, z: 5.0 },
        v2: Vector3 { x: 0.0, y: 5.0, z: 5.0 },
    },
    Triangle {
        v0: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 0.0, y: 5.0, z: 5.0 },
        v2: Vector3 { x: 0.0, y: 5.0, z: 0.0 },
    },
    // Right wall
    Triangle {
        v2: Vector3 { x: 5.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 5.0, y: 0.0, z: 5.0 },
        v0: Vector3 { x: 5.0, y: 5.0, z: 5.0 },
    },
    Triangle {
        v2: Vector3 { x: 5.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 5.0, y: 5.0, z: 5.0 },
        v0: Vector3 { x: 5.0, y: 5.0, z: 0.0 },
    },
    // Floor
    Triangle {
        v2: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 0.0, y: 0.0, z: 5.0 },
        v0: Vector3 { x: 5.0, y: 0.0, z: 5.0 },
    },
    Triangle {
        v2: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 5.0, y: 0.0, z: 5.0 },
        v0: Vector3 { x: 5.0, y: 0.0, z: 0.0 },
    },
    // Ceil
    Triangle {
        v0: Vector3 { x: 0.0, y: 5.0, z: 0.0 },
        v1: Vector3 { x: 0.0, y: 5.0, z: 5.0 },
        v2: Vector3 { x: 5.0, y: 5.0, z: 5.0 },
    },
    Triangle {
        v0: Vector3 { x: 0.0, y: 5.0, z: 0.0 },
        v1: Vector3 { x: 5.0, y: 5.0, z: 5.0 },
        v2: Vector3 { x: 5.0, y: 5.0, z: 0.0 },
    },
    // back
    Triangle {
        v2: Vector3 { x: 0.0, y: 0.0, z: 5.0 },
        v1: Vector3 { x: 0.0, y: 5.0, z: 5.0 },
        v0: Vector3 { x: 5.0, y: 5.0, z: 5.0 },
    },
    Triangle {
        v2: Vector3 { x: 0.0, y: 0.0, z: 5.0 },
        v1: Vector3 { x: 5.0, y: 5.0, z: 5.0 },
        v0: Vector3 { x: 5.0, y: 0.0, z: 5.0 },
    },
    // front
    Triangle {
        v0: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 0.0, y: 5.0, z: 0.0 },
        v2: Vector3 { x: 5.0, y: 5.0, z: 0.0 },
    },
    Triangle {
        v0: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        v1: Vector3 { x: 5.0, y: 5.0, z: 0.0 },
        v2: Vector3 { x: 5.0, y: 0.0, z: 0.0 },
    },
];
