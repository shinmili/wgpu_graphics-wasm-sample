mod utils;

use piston::RenderArgs;
use wasm_bindgen::prelude::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub async fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Initialize logger");
    utils::set_panic_hook();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        body.append_child(&canvas)
            .expect("Append canvas to HTML body");
    }

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                limits: wgpu::Limits {
                    max_storage_buffer_binding_size: 0,
                    max_storage_textures_per_shader_stage: 0,
                    max_storage_buffers_per_shader_stage: 0,
                    max_dynamic_storage_buffers_per_pipeline_layout: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: window.inner_size().width as u32,
        height: window.inner_size().height as u32,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &surface_config);

    let mut wgpu2d = wgpu_graphics::Wgpu2d::new(&device, &surface_config);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        #[cfg(target_arch = "wasm32")]
        log::debug!("{:?}", event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let surface_texture = surface.get_current_texture().unwrap();
                let surface_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let render_args = RenderArgs {
                    ext_dt: 0.0,
                    draw_size: window.inner_size().into(),
                    window_size: window
                        .inner_size()
                        .to_logical::<f64>(window.scale_factor())
                        .into(),
                };

                let command_buffer = wgpu2d.draw(
                    &device,
                    &surface_config,
                    &surface_view,
                    render_args.viewport(),
                    |_c, g| {
                        graphics::clear([0.0, 0.0, 0.0, 1.0], g);
                    },
                );

                queue.submit(std::iter::once(command_buffer));
                surface_texture.present();
            }
            _ => (),
        }
    });
}
