use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

pub async fn init(window: Arc<Window>) -> (Device, SurfaceConfiguration, Queue, Surface<'static>) {
    let window = Arc::new(window);
    let size = window.inner_size();
    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::BUFFER_BINDING_ARRAY,
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    (device, config, queue, surface)
}
