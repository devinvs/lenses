use std::sync::Arc;

use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::rasterization::CullMode;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::format::Format;

use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::RenderPassBeginInfo;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::pipeline::Pipeline;
use std::convert::TryFrom;
use vulkano::swapchain::SwapchainCreateInfo;
use vulkano::shader::ShaderModule;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::cpu_pool::CpuBufferPoolChunk;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    CommandBufferUsage,
    SubpassContents
};
use vulkano::device::{
    Device,
    DeviceExtensions,
    Queue,
    DeviceCreateInfo,
    QueueCreateInfo
};
use vulkano::device::physical::PhysicalDevice;
use vulkano::image::{
    ImageAccess,
    ImageUsage,
    SwapchainImage,
    AttachmentImage
};
use vulkano::image::view::ImageView;
use vulkano::instance::Instance;
use vulkano::memory::pool::StdMemoryPool;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{
    Framebuffer,
    RenderPass,
    Subpass
};
use vulkano::swapchain::{
    AcquireError,
    Surface,
    Swapchain,
    SwapchainCreationError
};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::swapchain;
use vulkano::instance::InstanceCreateInfo;

use vulkano_win::VkSurfaceBuild;

use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

use crate::{Vertex, Normal};

pub type VertexBuffer = Arc<CpuBufferPoolChunk<Vertex, Arc<StdMemoryPool>>>;
pub type IndexBuffer = Arc<CpuBufferPoolChunk<u32, Arc<StdMemoryPool>>>;
pub type NormalBuffer = Arc<CpuBufferPoolChunk<Normal, Arc<StdMemoryPool>>>;
pub type Uniform = vs::ty::Data;

#[derive(Debug, Clone)]
pub struct Model {
    pub index: u32,
    pub count: u32
}


pub struct VulkanState {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,

    surface: Arc<Surface<Window>>,
    dimensions: [u32; 2],
    swapchain: Arc<Swapchain<Window>>,
    _images: Vec<Arc<SwapchainImage<Window>>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,

    vertex_buffer_pool: CpuBufferPool<Vertex>,
    index_buffer_pool: CpuBufferPool<u32>,
    normal_buffer_pool: CpuBufferPool<Normal>,
    uniform_buffer_pool: CpuBufferPool<vs::ty::Data>,

    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,

    clear_color: [f32; 4],
}

impl VulkanState {
    pub fn new(event_loop: &EventLoop<()>) -> VulkanState {
        // Required extensions for rendering to a window
        let required_extensions = vulkano_win::required_extensions();

        // Create vulkan instance with required extensions
        let instance = Instance::new(
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                enumerate_portability: true,
                ..Default::default()
            }
        ).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let surface = WindowBuilder::new()
            .with_resizable(true)
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        let (physical, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| {
                p.supported_extensions().is_superset_of(&device_extensions)
            })
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| {
                        q.supports_graphics() && q.supports_surface(&surface).unwrap_or(false)
                    })
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                }
            })
            .expect("No suitable physical device found");

        let (device, mut queues) = Device::new(
            physical,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                ..Default::default()
            }
        ).unwrap();

        // The only queue we need right now is for rendering, may need transfer queue later
        let queue = queues.next().unwrap();

        // Load shaders
        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        // Create swapchain
        let (swapchain, images) = {
            let caps = physical.surface_capabilities(&surface, Default::default()).unwrap();
            //let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
            // Internal format for images
            let format = Some(
                physical
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0]
                    .0,
            );

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_format: format,
                    image_extent: surface.window().inner_size().into(),
                    image_usage: ImageUsage::color_attachment(),
                    composite_alpha: caps
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                }
            ).unwrap()
        };

        let dimensions: [u32; 2] = surface.window().inner_size().into();

        // We now create a buffer that will store the shape of our square
        let vertex_buffer_pool = CpuBufferPool::vertex_buffer(device.clone());
        let index_buffer_pool = CpuBufferPool::new(device.clone(), BufferUsage::index_buffer());
        let normal_buffer_pool = CpuBufferPool::vertex_buffer(device.clone());
        let uniform_buffer_pool = CpuBufferPool::uniform_buffer(device.clone());

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap();


        // Actual framebuffers to draw to
        let (framebuffers, pipeline) = VulkanState::window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone());
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        VulkanState {
            _instance: instance,
            device,
            queue,
            dimensions,
            surface,
            swapchain,
            _images: images,
            framebuffers,
            render_pass,
            pipeline,
            vs,
            fs,
            vertex_buffer_pool,
            index_buffer_pool,
            normal_buffer_pool,
            uniform_buffer_pool,
            recreate_swapchain: false,
            previous_frame_end,
            clear_color: [0.605, 0.607, 0.795, 1.0],
        }
    }

    pub fn draw<'a>(
        &mut self,
        vertex_buffer: VertexBuffer,
        index_buffer: IndexBuffer,
        normal_buffer: NormalBuffer,
        mut models: impl Iterator<Item=&'a Model>,
        uniforms: &Vec<vs::ty::Data>
    ) {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            self.recreate_swapchain();
        }

        // Descriptor set
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        // Acquire image from swapchain
        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
    
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some(self.clear_color.into()), Some(1f32.into())],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_num].clone())
                },
                SubpassContents::Inline,
            ).unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, (vertex_buffer.clone(), normal_buffer.clone()))
            .bind_index_buffer(index_buffer.clone());

        for i in 0..uniforms.len() {
            let uniform_buffer_subbuffer = {
                self.uniform_buffer_pool.next(uniforms[i]).unwrap()
            };

            let set = PersistentDescriptorSet::new(
                layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer),
                ]
            ).unwrap();

            let model = models.next().unwrap();

            builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                set.clone()
            )
            .draw_indexed(model.count * 3, 1, model.index * 3, 0, 0)
            .unwrap();

        }

        builder
            .end_render_pass()
            .unwrap();
        let command_buffer = builder.build().unwrap();

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();
        
        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(sync::FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        }
    }

    pub fn transfer_object_data<I, J, K>(&mut self, vertices: I, indices: J, normals: K) -> (
        VertexBuffer,
        IndexBuffer,
        NormalBuffer
    )
    where
        I: IntoIterator<Item = Vertex>,
        I::IntoIter: ExactSizeIterator,
        J: IntoIterator<Item = u32>,
        J::IntoIter: ExactSizeIterator,
        K: IntoIterator<Item = Normal>,
        K::IntoIter: ExactSizeIterator,
    {
        let vertex_buffer = self.vertex_buffer_pool.chunk(vertices).unwrap();
        let index_buffer = self.index_buffer_pool.chunk(indices).unwrap();
        let normal_buffer = self.normal_buffer_pool.chunk(normals).unwrap();

        (vertex_buffer, index_buffer, normal_buffer)
    }

    pub fn recreate_swapchain(&mut self) {
        // Get the new dimensions of the window.
        self.dimensions = self.surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self.swapchain.recreate(SwapchainCreateInfo {
                image_extent: self.dimensions.into(),
                ..self.swapchain.create_info()
            }) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        self.swapchain = new_swapchain;
        // Because framebuffers contains an Arc on the old swapchain, we need to
        // recreate framebuffers as well.
        (self.framebuffers, self.pipeline) = VulkanState::window_size_dependent_setup(self.device.clone(), &self.vs, &self.fs, &new_images,self.render_pass.clone());
        self.recreate_swapchain = false;
    }

    fn window_size_dependent_setup(
        device: Arc<Device>,
        vs: &ShaderModule,
        fs: &ShaderModule,
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<RenderPass>,
    ) -> (Vec<Arc<Framebuffer>>, Arc<GraphicsPipeline>) {
        let dimensions: [u32; 2] = images[0].dimensions().width_height();

        let depth_buffer = ImageView::new_default(
            AttachmentImage::transient(device.clone(), dimensions, Format::D16_UNORM).unwrap()
        ).unwrap();

        let fbs = images.iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
                        ..Default::default()
                    }
                ).unwrap()
            }).collect::<Vec<_>>();

        let pipeline = GraphicsPipeline::start()
        .vertex_input_state(
            BuffersDefinition::new()
                .vertex::<Vertex>()
                .vertex::<Normal>(),
        )
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
            Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            },
        ]))
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .color_blend_state(ColorBlendState::default().blend_alpha())
        .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap();

        (fbs, pipeline)
    }

    pub fn logical_size(&self) -> (f32, f32) {
        let size = self.surface.window().inner_size().to_logical(self.surface.window().scale_factor());

        (size.width, size.height)
    }

    pub fn physical_size(&self) -> (u32, u32) {
        let size = self.surface.window().inner_size();
        (size.width, size.height)
    }

    pub fn scale_factor(&self) -> f32 {
        self.surface.window().scale_factor() as f32
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/vert.glsl",
        types_meta: { use bytemuck::{Pod, Zeroable}; #[derive(Copy, Clone, Pod, Zeroable, Default)] },
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/frag.glsl"
    }
}
