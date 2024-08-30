use crate::vec3::*;
use crate::perlin::*;
use crate::rng::Random;

extern crate stb_image;

use stb_image::image::*;

pub trait Texture : Send + Sync {
    fn colour(&self, u : f32, v : f32, p : &Vec3) -> Vec3;
}

pub struct ConstantTexture {
    colour : Vec3
}

impl ConstantTexture {
    pub fn new_with_colour(colour : Vec3) -> Box<ConstantTexture> {
        Box::new(ConstantTexture {colour})
    }
}

impl Texture for ConstantTexture {
    fn colour(&self, _u : f32, _v : f32, _p : &Vec3) -> Vec3 {
        self.colour
    }
}

pub struct CheckerTexture {
    odd : Box<dyn Texture>,
    even : Box<dyn Texture>
}

impl CheckerTexture {
    pub fn new_with_textures(odd : Box<dyn Texture>, even : Box<dyn Texture>) -> Box<CheckerTexture> {
        Box::new(CheckerTexture { odd, even })
    }
}

impl Texture for CheckerTexture {
    fn colour(&self, u : f32, v : f32, p : &Vec3) -> Vec3 {
        let sines = (10.0 * p.x()).sin() * (10.0 * p.y()).sin() * (10.0 * p.z()).sin();
        if sines < 0.0 { self.odd.colour(u,v,p) } else { self.even.colour(u,v,p) }
    }
}

pub struct NoiseTexture {
    noise : Perlin,
    scale : f32
}

impl NoiseTexture {
    pub fn build(rnd : &mut Random, scale : f32) -> Box<NoiseTexture> {
        Box::new(NoiseTexture { noise : Perlin::build(rnd), scale})
    }
}

impl Texture for NoiseTexture {
    fn colour(&self, _u : f32, _v : f32, p : &Vec3) -> Vec3 {
        &Vec3::from(1.0, 1.0, 1.0) * (0.5*(1.0 + (self.scale*p.z() + 10.0*self.noise.turb(&(p*self.scale), 7)).sin()))
    }
}

pub struct ImageTexture {
    data : Vec<u8>,
    width : usize,
    height : usize
}

impl ImageTexture {
    pub fn load(path : String) -> ImageTexture {
        let img = stb_image::image::load(&path);
        match img {
            LoadResult::Error(msg) => {
                let err = format!("Could not load texture {}, {}", &path, msg);
                panic!("{}", err); 
            },
            LoadResult::ImageU8(img) => {
                ImageTexture {width: img.width, height : img.height, data : img.data}
            },
            LoadResult::ImageF32(_) => {
                let err = format!("Could not load texture {}, {}", &path, "unsupported format");
                panic!("{}", err); 
            }
        }
    }
}
impl Texture for ImageTexture {
    fn colour(&self, u : f32, v : f32, _p : &Vec3) -> Vec3 {
        let i = (u * self.width as f32) as i32;
        let j = ((1.0 - v) * self.height as f32 - 0.001) as i32;
        let i = if i < 0 { 0 } else {if i > self.width as i32 - 1 { self.width - 1 } else {i as usize}};
        let j = if j < 0 { 0 } else {if j > self.height as i32 - 1 { self.height - 1 } else {j as usize}};

        let base_address = 3*(i + self.width*j);
        &Vec3::from(f32::from(self.data[base_address]),f32::from(self.data[base_address + 1]), f32::from(self.data[base_address + 2])) / 255.0
    }
}