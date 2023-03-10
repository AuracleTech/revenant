use freetype::Library;
use image::DynamicImage;

use crate::{
    serialization::{deserialize_camera, serialize_camera},
    types::{
        AssetCamera, AssetFont, AssetImage2D, AssetManager, AssetMaterial, AssetMaterialSerialized,
        AssetTexture, Character, Filtering, ImageFormat, ImageKind, TextureSize, Wrapping,
    },
};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

#[cfg(not(debug_assertions))]
fn get_assets_path() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get current exe path");
    let mut assets_path = PathBuf::from(exe_path.parent().expect("Failed to get parent directory"));
    assets_path.push("assets");
    assets_path
}

#[cfg(debug_assertions)]
fn get_assets_path() -> PathBuf {
    let mut assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assets_path.push("assets");
    assets_path
}

impl AssetManager {
    pub fn new() -> Self {
        let assets_path = get_assets_path();
        Self {
            image_assets: HashMap::new(),
            image_assets_path: assets_path.join("images"),
            font_assets: HashMap::new(),
            font_assets_path: assets_path.join("fonts"),
            texture_assets: HashMap::new(),
            texture_assets_path: assets_path.join("textures"),
            material_assets: HashMap::new(),
            material_assets_path: assets_path.join("materials"),
            camera_assets: HashMap::new(),
            camera_assets_path: assets_path.join("cameras"),
            assets_path,
        }
    }

    // Image

    pub fn load_image_asset(&mut self, filename: &str) -> AssetImage2D {
        let path = self.image_assets_path.join(filename);
        let ext = path
            .extension()
            .expect("Failed to get file extension.")
            .to_str()
            .expect("Failed to convert file extension to str.");

        let image = match ext {
            "jpg" | "png" => image::open(path).expect("Failed to load image."),
            _ => panic!("Unsupported asset type"),
        };

        AssetImage2D {
            filename: filename.to_string(),
            image,
        }
    }

    // Font

    pub fn load_font_asset(&mut self, filename: &str, size: u32) -> AssetFont {
        let path = self.font_assets_path.join(filename);
        let ext = path
            .extension()
            .expect("Failed to get file extension.")
            .to_str()
            .expect("Failed to convert file extension to str.");

        let library = Library::init().expect("Could not init freetype library");
        let face = match ext {
            "ttf" => library.new_face(path, 0).expect("Could not open font"),
            _ => panic!("Unsupported asset type"),
        };
        let mut chars: HashMap<char, Character> = HashMap::new();

        // TODO make size configurable by width and height
        face.set_pixel_sizes(0, size)
            .expect("Could not set pixel size");

        // TODO make this configurable
        for c in 0..128 {
            chars.insert(c as u8 as char, Character::from_face(&face, c));
        }

        AssetFont {
            filename: filename.to_string(),
            size,
            chars,
        }
    }

    // Texture

    // TODO serialize and deserialize
    pub fn load_texture_asset(
        &mut self,
        filename: &str,
        kind: ImageKind,
        s_wrapping: Wrapping,
        t_wrapping: Wrapping,
        min_filtering: Filtering,
        mag_filtering: Filtering,
        mipmapping: bool,
    ) -> AssetTexture {
        let path = self.texture_assets_path.join(filename);
        let ext = path
            .extension()
            .expect("Failed to get file extension.")
            .to_str()
            .expect("Failed to convert file extension to str.");

        dbg!("Loading texture: '{}'.", filename);

        let image = match ext {
            "jpg" => image::open(path).expect("Failed to load image.").flipv(),
            _ => panic!("Unsupported asset type"),
        };

        if image.width() > i32::MAX as u32 {
            panic!(
                "Texture '{}' width too large dataloss not tolerated.",
                filename
            );
        }
        if image.height() > i32::MAX as u32 {
            panic!(
                "Texture '{}' height too tall dataloss not tolerated.",
                filename
            );
        }

        let size = TextureSize::TwoD {
            width: image.width() as i32,
            height: image.height() as i32,
        };

        // TODO support more than 3 channels
        let format = match image.color() {
            image::ColorType::Rgb8 => ImageFormat::RGB,
            _ => panic!("Texture format not supported."),
        };

        let data = match image {
            DynamicImage::ImageRgb8(texture_image) => texture_image.into_raw(),
            _ => panic!("Image format not supported"),
        };

        AssetTexture::create_texture(
            filename,
            data,
            kind,
            size,
            format,
            s_wrapping,
            t_wrapping,
            min_filtering,
            mag_filtering,
            mipmapping,
        )
    }

    // Material

    // TODO make load_material_asset
    // TODO make make_material_asset
    // TODO make load_material_asset use another function called deserialize_material_asset
    // TODO make make_material_asset use another function called serialize_material_asset
    pub fn serialize_material_asset(&mut self, material: &AssetMaterial) {
        let path = self.material_assets_path.join(&material.filename);
        let ext = path
            .extension()
            .expect("Failed to get file extension.")
            .to_str()
            .expect("Failed to convert file extension to str.");
        let name = material.filename.clone().replace(ext, "material");
        let save_path = self.material_assets_path.join(name);

        let serialized = AssetMaterialSerialized {
            diffuse: material.diffuse.filename.clone(),
            specular: material.specular.filename.clone(),
            specular_strength: material.specular_strength,
            emissive: material.emissive.filename.clone(),
        };

        let mut file = File::create(save_path).expect("Failed to create file.");
        let encoded = bincode::serialize(&serialized).expect("Failed to serialize material.");
        file.write_all(&encoded).expect("Failed to write to file.");
    }

    // TODO returns hard written values what the heckk
    pub fn deserialize_material_asset(&mut self, filename: &str) -> AssetMaterial {
        let path = self.material_assets_path.join(filename);
        let ext = path
            .extension()
            .expect("Failed to get file extension.")
            .to_str()
            .expect("Failed to convert file extension to str.");

        if ext != "material" {
            panic!("Material file extension must be 'material'.");
        }

        let mut file = File::open(path).expect("Failed to open file.");
        let mut encoded = Vec::new();
        file.read_to_end(&mut encoded)
            .expect("Failed to read file.");
        let serialized = bincode::deserialize::<AssetMaterialSerialized>(&encoded)
            .expect("Failed to deserialize material.");

        AssetMaterial {
            filename: filename.to_string(),
            diffuse: self.load_texture_asset(
                &serialized.diffuse,
                ImageKind::Diffuse,
                Wrapping::Repeat,
                Wrapping::Repeat,
                Filtering::Nearest,
                Filtering::Nearest,
                true,
            ),
            specular: self.load_texture_asset(
                &serialized.specular,
                ImageKind::Specular,
                Wrapping::Repeat,
                Wrapping::Repeat,
                Filtering::Nearest,
                Filtering::Nearest,
                true,
            ),
            specular_strength: serialized.specular_strength,
            emissive: self.load_texture_asset(
                &serialized.emissive,
                ImageKind::Emissive,
                Wrapping::Repeat,
                Wrapping::Repeat,
                Filtering::Nearest,
                Filtering::Nearest,
                true,
            ),
        }
    }

    // Camera

    // TODO rename camera to AssetCamera
    pub fn load_camera_asset(&mut self, name: &str) -> AssetCamera {
        deserialize_camera(&self.camera_assets_path.join(name).with_extension("camera"))
    }

    pub fn save_camera_asset(&mut self, camera: AssetCamera) {
        serialize_camera(
            self.camera_assets_path
                .join(&camera.filename)
                .with_extension("camera"),
            camera,
        )
    }
}
