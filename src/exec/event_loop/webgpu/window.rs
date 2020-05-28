use event::Event;
use std::{path::PathBuf, time};
use wgpu::{
    Adapter, Device, Instance, Queue, Surface, SwapChain, SwapChainDescriptor, SwapChainOutput,
};
use winit::{
    dpi::PhysicalSize,
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop as WinitEventLoop},
    window::Window as WinitWindow,
};

pub struct WindowContext {
    pub window: WinitWindow,
    _instance: Instance,
    _adapter: Adapter,
    surface: Surface,
    pub device: Device,
    pub queue: Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
}

impl WindowContext {
    async fn with_window(window: WinitWindow) -> WindowContext {
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

        // RUSTFIX KEEEELEEEEY (trace)
        let trace = PathBuf::from(&"trace".to_owned());
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: wgpu::Limits::default(),
                },
                Some(&*trace),
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
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

        // RUSTFIX there is a race condition here! we get a hang in this method... minimize this!!
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        WindowContext {
            window,
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        println!("Resizing to {:?}", size);
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn render<F: FnOnce(&SwapChainOutput, &Device, &Queue) -> wgpu::CommandBuffer>(
        &mut self,
        render_fn: F,
    ) {
        let frame = self
            .swap_chain
            .get_next_texture()
            .expect("Timeout when acquiring next swap chain texture");
        let command_buf = render_fn(&frame, &self.device, &self.queue);
        self.queue.submit(Some(command_buf));
    }
}

pub trait RendererBuilder {
    type Renderer: Renderer;

    fn build(
        self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self::Renderer, Option<wgpu::CommandBuffer>);
}

pub trait Renderer {
    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> wgpu::CommandBuffer;
}

pub trait Ticker<Output, Error> {
    type Builder: RendererBuilder;

    fn builder(&self) -> Self::Builder;

    fn tick(
        &mut self,
        renderer: &mut <Self::Builder as RendererBuilder>::Renderer,
        ctx: &WindowContext,
    ) -> Result<(), Error>;

    fn report(self, terminal: Option<Error>) -> Result<Output, Error>;
}

pub struct Window {
    event_loop: WinitEventLoop<()>,
    window: WinitWindow,
}

impl Window {
    pub fn new(title: &str) -> Self {
        let event_loop = WinitEventLoop::new();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title(title).with_visible(true);

        // RUSTFIX Enable this when the `wgpu` people do in their examples.
        #[cfg(windows_OFF)]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            builder = builder.with_no_redirection_bitmap(true);
        }

        let window = builder.build(&event_loop).unwrap();

        Self { event_loop, window }
    }

    #[inline]
    fn tick<Output, Error, T: Ticker<Output, Error>>(
        ctx: &mut WindowContext,
        event: Event<()>,
        control_flow: &mut ControlFlow,
        renderer: &mut <T::Builder as RendererBuilder>::Renderer,
        ticker: &mut T,
    ) -> Result<(), Error> {
        *control_flow = {
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
                ticker.tick(renderer, ctx)?;

                #[cfg(target_arch = "wasm32")]
                window.request_redraw();
            }
            event::Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                ctx.resize(size);
                ctx.render(|sw, dev, queue| renderer.render(sw, dev, queue));
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
                _ => (),
            },
            event::Event::RedrawRequested(_) => {
                ctx.render(|sw, dev, queue| renderer.render(sw, dev, queue));
            }
            _ => {}
        }

        Ok(())
    }

    async fn run_return_async<Output, Error, T: Ticker<Output, Error>>(
        mut self,
        mut ticker: T,
    ) -> Result<Output, Error> {
        let mut ctx = WindowContext::with_window(self.window).await;

        println!("Initializing the example...");
        let (mut renderer, init_command_buf) = ticker.builder().build(&ctx.sc_desc, &ctx.device);
        if init_command_buf.is_some() {
            ctx.queue.submit(init_command_buf);
        }

        ctx.window.set_visible(true);

        println!("Entering render loop...");
        use winit::platform::desktop::EventLoopExtDesktop;
        let mut tick_error = None;
        self.event_loop.run_return(|event, _, control_flow| {
            if tick_error.is_none() {
                if let Err(error) =
                    Self::tick(&mut ctx, event, control_flow, &mut renderer, &mut ticker)
                {
                    *control_flow = ControlFlow::Exit;
                    tick_error = Some(error);
                }
            }
        });

        ticker.report(tick_error)
    }

    pub fn run<Output, Error, T: Ticker<Output, Error>>(self, ticker: T) -> Result<Output, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            futures::executor::block_on(Self::run_return_async(self, ticker))
        }

        #[cfg(target_arch = "wasm32")]
        {
            // RUSTFIX make me compile!

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
            wasm_bindgen_futures::spawn_local(run_async::<R, _>(self, tick_fn));
        }
    }
}
