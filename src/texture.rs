use gl::types::GLvoid;
use image::DynamicImage;

pub struct Texture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub nr_channels: u32,
}

#[allow(dead_code)]
impl Texture {
    pub fn new(path: &str) -> Self {
        let texture_image = image::open(path).unwrap().flipv();
        let width = texture_image.width();
        let height = texture_image.height();
        let nr_channels = texture_image.color().channel_count();
        // TODO support more than 3 channels
        if nr_channels != 3 {
            panic!("Texture format not supported.");
        }
        let data = match texture_image {
            DynamicImage::ImageRgb8(texture_image) => texture_image.into_raw(),
            _ => panic!("Image format not supported"),
        };

        // set texture alignment to 1 byte because we are using 3 color channels (3 bytes) and not 4 (RGBA)
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }

        let mut texture = 0;
        unsafe {
            // generate texture id
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            // set texture wrapping
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            // set texture filtering
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            // set texture data
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                width as i32,
                height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            );
            // generates mipmaps
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Self {
            id: texture,
            width,
            height,
            nr_channels: nr_channels as u32,
        }
    }

    pub fn bind(&self, texture_unit: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}
