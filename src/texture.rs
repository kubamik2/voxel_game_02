use std::{io::Read, sync::Arc};

#[derive(Clone)]
pub struct Texture {
    texture: Arc<wgpu::Texture>,
    view: Arc<wgpu::TextureView>,
    sampler: Arc<wgpu::Sampler>
}


impl Texture {
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: &str) -> anyhow::Result<Self> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, image, Some(label)))
    }

    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, image: image::DynamicImage, label: Option<&str>) -> Self {
        let rgba = image.to_rgba8();

        use image::GenericImageView;
        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            sample_count: 1
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler: Arc::new(sampler),
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(device: &wgpu::Device, queue: &wgpu::Queue, path: P) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(path.as_ref())?;
        let file_name = path.as_ref().file_name().map(|f| f.to_str()).flatten().unwrap_or("texture");
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;

        Self::from_bytes(device, queue, &bytes, file_name)
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            sample_count: 1,
            view_formats: &[]
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(label),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture: Arc::new(texture),
            view: Arc::new(view),
            sampler: Arc::new(sampler),
        }
    }

    pub fn bind_group_layout_entries(&self, texture_binding: u32, sampler_binding: u32) -> [wgpu::BindGroupLayoutEntry; 2] {
        [
            wgpu::BindGroupLayoutEntry {
                binding: texture_binding,
                count: None,
                ty: wgpu::BindingType::Texture {
                    multisampled: self.texture.sample_count() > 1,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true }, // TODO might be something else across different textures
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            },
            wgpu::BindGroupLayoutEntry {
                binding: sampler_binding,
                count: None,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // TODO same
                visibility: wgpu::ShaderStages::FRAGMENT,
            },
        ]
    }

    pub fn bind_group_entries(&self, texture_binding: u32, sampler_binding: u32) -> [wgpu::BindGroupEntry; 2] {
        [
            wgpu::BindGroupEntry {
                binding: texture_binding,
                resource: wgpu::BindingResource::TextureView(&self.view),
            },
            wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            }
        ]
    }
}

static TEXTURE_ATLAS_BIND_GROUP_LAYOUT: std::sync::OnceLock<wgpu::BindGroupLayout> = std::sync::OnceLock::new();

pub struct TextureAtlas {
    texture: Texture,
    bind_group: wgpu::BindGroup,
}

impl TextureAtlas {
    pub fn new<P: AsRef<std::path::Path>>(file_path: P, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut file = std::fs::File::open(file_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let texture = Texture::from_bytes(device, queue, &bytes, "texture_atlas_texture").unwrap();
        let bind_group = Self::create_bind_group(device, &texture);
        Self { texture, bind_group }
    }

    pub fn get_or_init_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
        TEXTURE_ATLAS_BIND_GROUP_LAYOUT.get_or_init(|| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_atlas_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None
                    }
                ]
            })
        })
    }
    
    fn create_bind_group(device: &wgpu::Device, texture: &Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_atlas_bind_group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler)
                }
            ],
            layout: Self::get_or_init_bind_group_layout(device)
        })
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}