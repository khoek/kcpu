// RUSTFIX remove the need for `bytemuck`?
use bytemuck::{Pod, Zeroable};
use parking_lot::RwLock;
use std::sync::Arc;

// RUSTFIX update this as the `wgpu-rs` examples are updated

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        vertex([-1, -1, 0], [0, 0]),
        vertex([1, -1, 0], [1, 0]),
        vertex([1, 1, 0], [1, 1]),
        vertex([-1, 1, 0], [0, 1]),
    ];

    let index_data: &[u16] = &[0, 1, 2, 2, 3, 0];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_texels(size: usize, then: std::time::Instant) -> Vec<u8> {
    use std::iter;

    let k = 100f32 / (then.elapsed().as_secs_f32());

    (0..size * size)
        .flat_map(|_| {
            let count = 4;
            iter::once(0xFF - (count * ((5f32 * k) as u32)) as u8)
                .chain(iter::once(0xFF - (count * ((15f32 * k) as u32)) as u8))
                .chain(iter::once(0xFF - (count * ((50f32 * k) as u32)) as u8))
                .chain(iter::once(1))
        })
        .collect()
}

fn write_texels(size: usize, then: std::time::Instant, b: &mut [u8]) {
    let k = 100f32 / then.elapsed().as_secs_f32();

    for i in 0..size * size {
        b[4 * i + 0] = 0xFF - (4 * ((5f32 * k) as u32)) as u8;
        b[4 * i + 1] = 0xFF - (4 * ((15f32 * k) as u32)) as u8;
        b[4 * i + 2] = 0xFF - (4 * ((50f32 * k) as u32)) as u8;
        b[4 * i + 3] = 1;
    }
}

struct Example {
    texture: wgpu::Texture,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    buff: Arc<Box<RwLock<Vec<u8>>>>,
}

impl framework::Example for Example {
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        use std::mem;

        let then = std::time::Instant::now();

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let vertex_size = mem::size_of::<Vertex>();
        let (vertex_data, index_data) = create_vertices();

        let vertex_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&vertex_data),
            wgpu::BufferUsage::VERTEX,
        );

        let index_buf = device
            .create_buffer_with_data(bytemuck::cast_slice(&index_data), wgpu::BufferUsage::INDEX);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let size = 1024u32;
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_default_view();
        let texels = create_texels(size as usize, then);
        let pixeldata =
            device.create_buffer_with_data(texels.as_slice(), wgpu::BufferUsage::COPY_SRC);
        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &pixeldata,
                offset: 0,
                bytes_per_row: 4 * size,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texture_extent,
        );

        // Create other resources
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        // Create the render pipeline
        let vs_bytes = include_bytes!("shader.vert.spv");
        let fs_bytes = include_bytes!("shader.frag.spv");
        let vs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs_bytes[..])).unwrap());
        let fs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs_bytes[..])).unwrap());

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: vertex_size as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: 4 * 4,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let buff = Arc::new(Box::new(RwLock::new(texels)));

        // Done
        let this = Example {
            texture,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group,
            pipeline,
            buff,
        };

        let buff = Arc::clone(&this.buff);
        std::thread::spawn(move || {
            loop {
                // let mut lock = buff.write();
                // write_texels(size as usize, then, lock.as_mut());
                // drop(lock);

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        (this, Some(init_encoder.finish()))
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn resize(
        &mut self,
        _: &wgpu::SwapChainDescriptor,
        _: &wgpu::Device,
    ) -> Option<wgpu::CommandBuffer> {
        None
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
    ) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let size = 1024u32;
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };

        let lock = self.buff.read();
        let temp_buf = device.create_buffer_with_data(lock.as_slice(), wgpu::BufferUsage::COPY_SRC);
        drop(lock);

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                bytes_per_row: 4 * size,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texture_extent,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..));
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        encoder.finish()
    }
}

fn main() {
    framework::run::<Example>("image");
}

mod framework {
    use std::time;
    use winit::{
        event::{self, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::Window,
    };

    #[allow(dead_code)]
    pub fn cast_slice<T>(data: &[T]) -> &[u8] {
        use std::mem::size_of;
        use std::slice::from_raw_parts;

        unsafe { from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<T>()) }
    }

    #[allow(dead_code)]
    pub enum ShaderStage {
        Vertex,
        Fragment,
        Compute,
    }

    pub trait Example: 'static + Sized {
        fn init(
            sc_desc: &wgpu::SwapChainDescriptor,
            device: &wgpu::Device,
        ) -> (Self, Option<wgpu::CommandBuffer>);
        fn resize(
            &mut self,
            sc_desc: &wgpu::SwapChainDescriptor,
            device: &wgpu::Device,
        ) -> Option<wgpu::CommandBuffer>;
        fn update(&mut self, event: WindowEvent);
        fn render(
            &mut self,
            frame: &wgpu::SwapChainOutput,
            device: &wgpu::Device,
        ) -> wgpu::CommandBuffer;
    }

    async fn run_async<E: Example>(event_loop: EventLoop<()>, window: Window) {
        println!("Initializing the surface...");

        let instance = wgpu::Instance::new();
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(&window);
            (size, surface)
        };

        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let mut sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // TODO: Allow srgb unconditionally
            format: if cfg!(target_arch = "wasm32") {
                wgpu::TextureFormat::Bgra8Unorm
            } else {
                wgpu::TextureFormat::Bgra8UnormSrgb
            },
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

        println!("Initializing the example...");
        let (mut example, init_command_buf) = E::init(&sc_desc, &device);
        if init_command_buf.is_some() {
            queue.submit(init_command_buf);
        }

        #[cfg(not(target_arch = "wasm32"))]
        let mut last_update_inst = time::Instant::now();

        println!("Entering render loop...");
        event_loop.run(move |event, _, control_flow| {
            let _ = (&instance, &adapter); // force ownership by the closure
            *control_flow = if cfg!(feature = "metal-auto-capture") {
                ControlFlow::Exit
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ControlFlow::WaitUntil(time::Instant::now() + time::Duration::from_millis(10))
                }
                #[cfg(target_arch = "wasm32")]
                {
                    ControlFlow::Poll
                }
            };
            match event {
                event::Event::MainEventsCleared => {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if last_update_inst.elapsed() > time::Duration::from_millis(20) {
                            window.request_redraw();
                            last_update_inst = time::Instant::now();
                        }
                    }

                    #[cfg(target_arch = "wasm32")]
                    window.request_redraw();
                }
                event::Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    println!("Resizing to {:?}", size);
                    sc_desc.width = size.width;
                    sc_desc.height = size.height;
                    swap_chain = device.create_swap_chain(&surface, &sc_desc);
                    let command_buf = example.resize(&sc_desc, &device);
                    if command_buf.is_some() {
                        queue.submit(command_buf);
                    }

                    let frame = swap_chain
                        .get_next_texture()
                        .expect("Timeout when acquiring next swap chain texture");
                    let command_buf = example.render(&frame, &device);
                    queue.submit(Some(command_buf));
                }
                event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                                state: event::ElementState::Pressed,
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {
                        example.update(event);
                    }
                },
                event::Event::RedrawRequested(_) => {
                    let frame = swap_chain
                        .get_next_texture()
                        .expect("Timeout when acquiring next swap chain texture");
                    // println!("{:?} {:?}", std::time::Instant::now(), event);
                    let command_buf = example.render(&frame, &device);
                    queue.submit(Some(command_buf));
                }
                _ => {}
            }
        });
    }

    pub fn run<E: Example>(title: &str) {
        let event_loop = EventLoop::new();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title(title);
        #[cfg(windows_OFF)] //TODO
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            builder = builder.with_no_redirection_bitmap(true);
        }
        let window = builder.build(&event_loop).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        {
            futures::executor::block_on(run_async::<E>(event_loop, window));
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
            use winit::platform::web::WindowExtWebSys;
            // On wasm, append the canvas to the document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
            wasm_bindgen_futures::spawn_local(run_async::<E>(event_loop, window));
        }
    }
}
