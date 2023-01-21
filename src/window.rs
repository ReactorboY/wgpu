use std::borrow::Cow;

use wgpu::{
    Backends, Color, CompositeAlphaMode, Device, Features, IndexFormat, Instance, Limits,
    PowerPreference, PresentMode, PrimitiveTopology, Queue, RenderPassColorAttachment,
    RenderPipeline, ShaderSource, Surface, SurfaceConfiguration, SurfaceError, TextureUsages,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct Inputs<'a> {
    pub source: ShaderSource<'a>,
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
}

pub struct State {
    surface: Surface,
    size: PhysicalSize<u32>,
    background: Color,
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    window: Window,
    render_pipeline: RenderPipeline,
}

pub async fn run() {
    env_logger::init();
    // event loop
    let event_loop = EventLoop::new();

    // window instance
    let window = Window::new(&event_loop).unwrap();

    window.set_title("My Window");

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() && !state.input(event) => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => {
                println!("Closing The window");
                *control_flow = ControlFlow::Exit
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if it's lost or outdated
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    state.resize(state.size)
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        }
        _ => {}
    });
}

impl State {
    fn update(&mut self) {}

    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(Backends::all());

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &config);

        let mut primitive_type = "triangle-list";
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 {
            primitive_type = &args[1];
        }

        let mut topology = wgpu::PrimitiveTopology::PointList;
        let mut index_format = None;
        if primitive_type == "line-list" {
            topology = wgpu::PrimitiveTopology::LineList;
            index_format = None;
        } else if primitive_type == "triangle-list" {
            topology = wgpu::PrimitiveTopology::TriangleList;
        } else if primitive_type == "triangle-strip" {
            topology = wgpu::PrimitiveTopology::TriangleStrip;
            index_format = Some(wgpu::IndexFormat::Uint32);
        } else if primitive_type == "line-strip" {
            topology = wgpu::PrimitiveTopology::LineStrip;
            index_format = Some(wgpu::IndexFormat::Uint32);
        }

        window.set_title(&*format!("{}: {}", "Primitive", primitive_type));

        let inputs = Inputs {
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader_triangle.wgsl"))),
            topology: topology,
            strip_index_format: index_format,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: inputs.source,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: inputs.topology,
                strip_index_format: inputs.strip_index_format,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        Self {
            background: wgpu::Color {
                r: 0.05,
                g: 0.062,
                b: 0.08,
                a: 1.0,
            },
            size,
            surface,
            config,
            device,
            queue,
            window,
            render_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            println!("Resizing {} {}", new_size.width, new_size.height);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // actual drawing started here
        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            _render_pass.set_pipeline(&self.render_pipeline);
            _render_pass.draw(0..9, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // WindowEvent::CursorMoved { position, .. } => {
            //     // println!("Change Color");
            //     // self.background = Color {
            //     //     r: position.x as f64 / self.size.width as f64,
            //     //     g: position.y as f64 / self.size.height as f64,
            //     //     b: 1.0,
            //     //     a: 1.0,
            //     // };
            //     true
            // }
            _ => false,
        }
    }
}
