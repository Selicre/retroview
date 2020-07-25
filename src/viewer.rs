use crate::gl::tiles::*;
use crate::gl::{Drawing, Driver, Tileset};

use glium::glutin;

use glium::texture::{RawImage2d, Texture2d, SrgbTexture2d};
use glium::VertexBuffer;


pub struct Buffer<T: Copy> {
    pub local: Vec<T>,
    pub buf: VertexBuffer<T>,
}

impl<T: Copy + glium::Vertex> Buffer<T> {
    fn new(driver: &mut Driver, src: Vec<T>) -> Self {
        Buffer {
            buf: VertexBuffer::new(driver.display, &src).unwrap(),
            local: src
        }
    }
    fn dynamic(driver: &mut Driver, src: Vec<T>) -> Self {
        Buffer {
            buf: VertexBuffer::dynamic(driver.display, &src).unwrap(),
            local: src
        }
    }
    fn update(&mut self, driver: &mut Driver) {
        //if self.local.len() != self.buf.len() {
            self.buf = VertexBuffer::dynamic(driver.display, &self.local).unwrap();
        //} else {
        //    self.buf.write(&self.local);
        //}
    }
}

pub struct Viewer {
    offset: [f32; 2],

    pal_selected: u32,
    scale: f32,

    gfx: Texture2d,
    pal: SrgbTexture2d,

    ui_gfx: Texture2d,
    ui_pal: SrgbTexture2d,

    vram_layout: Buffer<SpriteTile>,
    sprite_data: Buffer<SpriteTile>,
}

impl Viewer {
    pub fn new(gfx: &snesgfx::from::Tileset, pal: &snesgfx::from::Palette, obj_file: &[u8], driver: &mut Driver) -> Self {
        let gfx = image::DynamicImage::ImageLuma8(gfx.as_image()).to_rgba();
        let dim = gfx.dimensions();
        println!("{:?}", dim);
        let gfx = RawImage2d::from_raw_rgba(gfx.into_raw(), dim);


        let pal = pal.as_image();
        let dim = pal.dimensions();
        let pal = RawImage2d::from_raw_rgba(pal.into_raw(), dim);

        let vram_layout = Buffer::new(driver, (0..0x800).map(|c| SpriteTile {
            position: [(c % 16) as f32 * 8.0, (c / 16) as f32 * 8.0],
            tile_id: c,
            flip: 0,
            pal: 0
        }).collect());

        let sprite_data = parse_obj_file(obj_file);

        Self {
            offset: [0.0; 2],

            pal_selected: 0,
            scale: 2.0,

            gfx: Texture2d::new(driver.display, gfx).unwrap(),
            pal: SrgbTexture2d::new(driver.display, pal).unwrap(),

            ui_gfx: Texture2d::empty(driver.display, 256, 256).unwrap(),
            ui_pal: SrgbTexture2d::empty(driver.display, 16, 16).unwrap(),

            vram_layout,

            sprite_data: Buffer::dynamic(driver, sprite_data)
        }
    }
    pub fn move_screen(&mut self, x: f32, y: f32) {
        self.offset[0] -= x;
        self.offset[1] -= y;
    }
    pub fn move_pal(&mut self, driver: &mut Driver, pal: i32) {
        self.pal_selected += pal as u32;
        self.pal_selected %= 16;
        for i in self.vram_layout.local.iter_mut() {
            i.pal = self.pal_selected;
        }
        self.vram_layout.update(driver);
    }
    pub fn draw(&mut self, driver: &mut Driver) {
        let drawing = Drawing::merged(&self.vram_layout.buf, (&self.gfx, &self.pal))
            .scale(self.scale)
            .offset((self.offset[0] + 512.0) / self.scale, self.offset[1] / self.scale);
        driver.draw(drawing);
        let drawing = Drawing::merged(&self.sprite_data.buf, (&self.gfx, &self.pal))
            .scale(self.scale)
            .offset(self.offset[0] / self.scale, self.offset[1] / self.scale);
        driver.draw(drawing);
    }
}
fn parse_obj_file(input: &[u8]) -> Vec<SpriteTile> {
    input.chunks_exact(6).rev().flat_map(|c| if let &[a,b,y,x,props,tile] = c {
        let tile_id = tile as u32 + ((props as u32 & 0x01) << 8) + 0x200;
        let pal = ((props as u32 >> 1) & 0x07) + 8;
        let flip = props as u32 >> 6;
        if a & 0x80 != 0 {
            println!("{:02X}{:02X} ab, {:02X}:{:02X} off, {:03X} tile", a,b, x,y, tile_id);
            if a & 0x01 != 0 {
                let hflip = flip & 0x01 != 0;
                let vflip = flip & 0x02 != 0;

                let h1 = if hflip { 8.0 } else { 0.0 };
                let h2 = if hflip { 0.0 } else { 8.0 };
                let v1 = if vflip { 8.0 } else { 0.0 };
                let v2 = if vflip { 0.0 } else { 8.0 };
                vec![
                    SpriteTile {
                        position: [x as i8 as f32 + h1, y as i8 as f32 + v1],
                        tile_id, pal,
                        flip
                    },
                    SpriteTile {
                        position: [x as i8 as f32 + h2, y as i8 as f32 + v1],
                        tile_id: tile_id + 1, pal,
                        flip
                    },
                    SpriteTile {
                        position: [x as i8 as f32 + h1, y as i8 as f32 + v2],
                        tile_id: tile_id + 16, pal,
                        flip
                    },
                    SpriteTile {
                        position: [x as i8 as f32 + h2, y as i8 as f32 + v2],
                        tile_id: tile_id + 17, pal,
                        flip
                    },

                ]
            } else {
                vec![SpriteTile {
                    position: [x as i8 as f32, y as i8 as f32],
                    tile_id, pal,
                    flip
                }]
            }
        } else {
            vec![]
        }
    } else { vec![] }).collect()
}
