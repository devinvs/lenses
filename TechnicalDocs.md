# Technical Documentation

This system traces light rays through a scene filled with lenses and solid
objects and then renders the result in an interactive environment.

## I promise I usually use vcs better than this

```
> git log

commit c5ac9e8260d3ee0943fd83af14dd3979d5f5f888 (HEAD -> main, origin/main)
Author: Devin Vander Stelt <devin@vstelt.dev>
Date:   Mon May 8 21:00:20 2023 -0400

    now the rest of it

commit ad92d5c6eb46a7a6cffb2bb2b38be44c44164aa8
Author: Devin Vander Stelt <devin@vstelt.dev>
Date:   Mon Apr 10 01:44:45 2023 -0400

    rework graphics code

commit 953e695798b53a35bf1ecc28b7b69c0f79dbf775
Author: Devin Vander Stelt <devin@vstelt.dev>
Date:   Tue Mar 7 10:26:36 2023 -0500

    teapot
```

## Structure

- src/main.rs
    + glue code, reads the yaml scene into a world, runs the raytracing, and
      then starts the render loop
- src/world.rs
    + contains all scene geometry and information. Contains code for software
      ray-tracing as well
- src/geometry.rs
    + Basic geometric definitions for triangles and rays. Includes intersection
      and tessellation code for them as well.
- src/lenses.rs
    + Definition of and tessellation code for lenses
- src/light.rs
    + Definition of lights and code for spawning rays
- ply.rs
    + Utility for reading ply files
- kdtree.rs
    + KDtree acceleration structure creation and traversal
- vulkan.rs
    + Code for creating and running the local illumination renderer
- vert.glsl
    + Vertex Shader
- frag.glsl
    + Fragment Shader

## Important Structures

```rust
/// Describes a model that has been uploaded to the gpu
pub struct Model {
    /// Index of the first triangle in this model in the vertex buffer
    pub index: u32,
    /// Number of triangles in this model
    pub count: u32
}
```

```rust
/// Holds the entire scene data and all models
pub struct World {
    // Per collidable entity
    pub models: Vec<Model>,
    pub colors: Vec<Vector4<f32>>,
    pub positions: Vec<Vector3<f32>>,
    pub scales: Vec<Vector3<f32>>,
    pub materials: Vec<Material>,

    // ray traced line segments
    pub lines: Vec<Model>,

    // Mappings for the models to triangles
    pub model_idx: Vec<Model>,
    pub model_data: Vec<Triangle>,

    // global
    pub vertex_buffer: Option<VertexBuffer>,
    pub index_buffer: Option<IndexBuffer>,
    pub normal_buffer: Option<NormalBuffer>,

    pub lights: Vec<Light>,

    pub kdtree: Option<KDNode>,

    pub fov: f32,
    pub rotx: f32,
    pub roty: f32,
    pub rotz: f32
}
```

```rust
pub struct VulkanState {
    /// Handle for vulkan instance
    _instance: Arc<Instance>,
    /// Handle for graphcis device
    device: Arc<Device>,
    /// Handle for graphics queue for rendering
    queue: Arc<Queue>,

    /// Handle of the rendering surface
    surface: Arc<Surface<Window>>,

    /// Dimensions of the rendering surface
    dimensions: [u32; 2],
    /// Swapchain of images to render to
    swapchain: Arc<Swapchain<Window>>,
    /// The images held in the swapchain
    _images: Vec<Arc<SwapchainImage<Window>>>,
    /// Handle on the framebuffer
    framebuffers: Vec<Arc<Framebuffer>>,
    /// Handle ont he render pass object used for rendering
    render_pass: Arc<RenderPass>,
    /// The graphics pipeline to bind and run the shaders on
    pipeline: Arc<GraphicsPipeline>,

    /// Vertex Shader
    vs: Arc<ShaderModule>,
    /// Fragment Shaer
    fs: Arc<ShaderModule>,

    // Various ring buffers for allocating new buffers
    vertex_buffer_pool: CpuBufferPool<Vertex>,
    index_buffer_pool: CpuBufferPool<u32>,
    normal_buffer_pool: CpuBufferPool<Normal>,
    uniform_buffer_pool: CpuBufferPool<vs::ty::Data>,

    /// use for screen resizes/ unoptimal image formats
    pub recreate_swapchain: bool,
    /// Future for previous frame end, used for synchronization
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,

    /// Color to fill clear the images with each render pass
    clear_color: [f32; 4],
}
```

```rust
#[derive(Serialize, Deserialize)]
pub struct Lens {
    pub radius: f32,
    pub left: LensSide,
    pub right: LensSide,
}

#[derive(Serialize, Deserialize)]
pub enum LensSide {
    Flat,
    Concave(f32),
    Convex(f32)
}
```

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum Light {
    // origin, direction
    Laser([f32; 3], [f32; 3]),
    // origin
    Point([f32; 3]),
}
```
