#[test]
fn atomic_vec2u_requires_device_feature() {
    let source = include_str!("../../wgpu-gpu/atomic_vec2u.wgsl");
    let (device, _) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
    wgpu_test::fail(
        &device,
        || {
            let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        },
        Some("parsing error"),
    );
    wgpu_test::fail(
        &device,
        || {
            let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl("enable atomic_vec2u_min_max;".into()),
            });
        },
        Some("parsing error"),
    );
    let (device, _) = wgpu::Device::noop(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::ATOMIC_VEC2U_MIN_MAX,
        ..Default::default()
    });
    let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    assert!(!device.features().contains(wgpu::Features::SHADER_INT64));
}
