#[derive(Copy,Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}
glium::implement_vertex!(Vertex, position);

#[derive(Copy,Clone,Debug)]
pub struct TilemapCell {
    pub tile_id: u32,
    pub flip: u32,  // bitfield: yx
    pub pal: u32
}
glium::implement_vertex!(TilemapCell, tile_id, flip, pal);

impl TilemapCell {
    pub fn merge(self, position: [f32;2]) -> SpriteTile {
        SpriteTile {
            position,
            tile_id: self.tile_id,
            flip: self.flip,
            pal: self.pal
        }
    }
}

#[derive(Copy,Clone,Debug)]
pub struct SpriteTile {
    pub position: [f32; 2],
    pub tile_id: u32,
    pub flip: u32,
    pub pal: u32
}
glium::implement_vertex!(SpriteTile, position, tile_id, flip, pal);

impl SpriteTile {
    pub fn split(self) -> (Vertex, TilemapCell) {
        (Vertex { position: self.position }, TilemapCell {
            tile_id: self.tile_id,
            flip: self.flip,
            pal: self.pal
        })
    }
}
