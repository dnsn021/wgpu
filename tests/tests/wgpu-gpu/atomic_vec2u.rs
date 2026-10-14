use wgpu::util::DeviceExt;
use wgpu_test::{apply, gpu_test, GpuTestConfiguration, GpuTestInitializer, TestParameters};

pub fn all_tests(tests: &mut Vec<GpuTestInitializer>) {
    tests.push(ATOMIC_VEC2U_MIN_MAX);
}

#[apply(gpu_test!)]
static ATOMIC_VEC2U_MIN_MAX: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(
        TestParameters::default()
            .limits(wgpu::Limits::default())
            .features(wgpu::Features::ATOMIC_VEC2U_MIN_MAX),
    )
    .run_async(|ctx| async move {
        assert!(!ctx.device.features().contains(wgpu::Features::SHADER_INT64));
        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("atomic_vec2u.wgsl"));
        let initial: Vec<u32> = [u64::MAX, 0, u64::MAX, 0]
            .into_iter()
            .flat_map(|value| {
                [
                    0x1357,
                    0xaaaa,
                    value as u32,
                    (value >> 32) as u32,
                    0x2468,
                    0xbbbb,
                ]
            })
            .collect();
        let expected: Vec<u32> = [
            511u64,
            511u64 << 32,
            0x7fff_ffff_ffff_ffff,
            0x8000_0000_0000_0000,
        ]
        .into_iter()
        .flat_map(|value| {
            [
                0x1357,
                0xaaaa,
                value as u32,
                (value >> 32) as u32,
                0x2468,
                0xbbbb,
            ]
        })
        .collect();
        let expected_values: Vec<u32> = expected
            .chunks_exact(6)
            .flat_map(|entry| [entry[2], entry[3]])
            .collect();
        let output = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("atomic vec2u output"),
                contents: bytemuck::cast_slice(&initial),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            });
        let values = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("atomic vec2u decoded values"),
            size: (expected_values.len() * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("atomic vec2u readback"),
            size: output.size() + values.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bgl = ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: output.as_entire_binding(),
            }],
        });
        let compute = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("compute"),
                compilation_options: Default::default(),
                cache: None,
            });
        let reader = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("atomic vec2u read-only consumer"),
                layout: None,
                module: &shader,
                entry_point: Some("read_values"),
                compilation_options: Default::default(),
                cache: None,
            });
        let read_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &reader.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: values.as_entire_binding(),
                },
            ],
        });
        let render = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            });
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 32,
                height: 16,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        for fragment in [false, true] {
            for _ in 0..4 {
                ctx.queue
                    .write_buffer(&output, 0, bytemuck::cast_slice(&initial));
                let mut encoder = ctx.device.create_command_encoder(&Default::default());
                if fragment {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Discard,
                            },
                        })],
                        ..Default::default()
                    });
                    pass.set_pipeline(&render);
                    pass.set_bind_group(0, &bind_group, &[]);
                    pass.draw(0..3, 0..1);
                } else {
                    let mut pass = encoder.begin_compute_pass(&Default::default());
                    pass.set_pipeline(&compute);
                    pass.set_bind_group(0, &bind_group, &[]);
                    pass.dispatch_workgroups(8, 1, 1);
                }
                {
                    let mut pass = encoder.begin_compute_pass(&Default::default());
                    pass.set_pipeline(&reader);
                    pass.set_bind_group(0, &read_group, &[]);
                    pass.dispatch_workgroups(1, 1, 1);
                }
                encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, output.size());
                encoder.copy_buffer_to_buffer(&values, 0, &readback, output.size(), values.size());
                ctx.queue.submit([encoder.finish()]);
                readback
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, Result::unwrap);
                ctx.async_poll(wgpu::PollType::wait_indefinitely())
                    .await
                    .unwrap();
                {
                    let data = readback.slice(..).get_mapped_range().unwrap();
                    assert_eq!(
                        bytemuck::cast_slice::<u8, u32>(&data[..output.size() as usize]),
                        expected,
                        "fragment={fragment}"
                    );
                    assert_eq!(
                        bytemuck::cast_slice::<u8, u32>(&data[output.size() as usize..]),
                        expected_values,
                        "read-only consumer after fragment={fragment}"
                    );
                }
                readback.unmap();
            }
        }
    });
