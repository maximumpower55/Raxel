use std::{borrow::Cow, fs};

pub enum ResourceLoader {
    TEXTURE,
    SHADER,
}

pub struct ResourceId(pub Option<String>, pub String);

impl ResourceId {
    pub fn load<'a>(&self, loader: ResourceLoader) -> LoadedResource<'a> {
        let formatted = &format!(
            "resources/{}{}",
            self.0
                .as_ref()
                .map(|value| value.clone() + "/")
                .unwrap_or(String::new()),
            self.1
        );
        let path = &std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(formatted);
        match loader {
            ResourceLoader::SHADER => {
                let data = fs::read_to_string(path)
                    .expect(&format!("No shader at {}!", path.to_str().unwrap()));
                LoadedResource::SHADER(wgpu::ShaderSource::Wgsl(Cow::Owned(data)))
            }
            ResourceLoader::TEXTURE => {
                let data =
                    image::open(path).expect(&format!("No texture at {}!", path.to_str().unwrap()));
                LoadedResource::TEXTURE(data)
            }
        }
    }
}
pub enum LoadedResource<'a> {
    TEXTURE(image::DynamicImage),
    SHADER(wgpu::ShaderSource<'a>),
}
