use std::io::Read;

pub struct Shader {
    shader_module: wgpu::ShaderModule,
}

impl Shader {
    pub fn from_str(device: &wgpu::Device, source: &str, label: &str) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into())
        });

        Self { shader_module }
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.shader_module
    }

    pub fn from_file<P: AsRef<std::path::Path>>(device: &wgpu::Device, path: P) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(path.as_ref())?;
        let file_name = path.as_ref().file_name().map(|f| f.to_str()).flatten().unwrap_or("shader");
        let mut source = String::new();
        file.read_to_string(&mut source)?;

        Ok(Self::from_str(device, &source, file_name))
    }
}