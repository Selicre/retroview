use crate::ram::GenesisRam;

use crate::ReadBE;

use super::{Drawing, Driver, Tileset};

use super::tiles::*;

use glium::glutin;

use glium::texture::{RawImage2d, Texture2d, SrgbTexture2d};
use glium::VertexBuffer;

const ROM: &[u8] = include_bytes!("../sonic3k.bin");

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


/*pub fn render(this: &mut Option<S3K>, driver: &mut Driver<'_>, frame: GenesisRam) {
    let first = this.is_none();

    let this = this.get_or_insert_with(|| {
        S3K {
            prev_state: frame.clone(),
            tex: create_texture(driver, &frame.v),
            pal: create_palette(driver, &frame.m68k[0xFC00..0xFC80], &frame.m68k[0xF080..0xF100]),

            plane_a: create_plane(driver, &frame.m68k, false),
            plane_b: create_plane(driver, &frame.m68k, true),
            x_offset: 0.0,
            y_offset: 0.0,
        }
    });
    if !first {
        this.tex = create_texture(driver, &frame.v);
        this.pal = create_palette(driver, &frame.m68k[0xFC00..0xFC80], &frame.m68k[0xF080..0xF100]);
        if &this.prev_state.m68k[0..0xA800] != &frame.m68k[0..0xA800] {
            //let r = this.prev_state.m68k[0..0xA800].iter().zip(frame.m68k[0..0xA800].iter()).position(|(a,b)| a != b).unwrap();
            //println!("{:04X}: {} => {}", r, this.prev_state.m68k[r], frame.m68k[r]);
            this.plane_a = create_plane(driver, &frame.m68k, false);
            this.plane_b = create_plane(driver, &frame.m68k, true);
        }

    }
    this.update_offset(driver, &frame.m68k);
    // TODO: update shit
    this.render_tiles(driver, &frame.m68k);
    this.render_sprites(driver, &frame.m68k, &frame.v);

    this.prev_state = frame;
    //this.test(driver);
}*/

pub struct S3K {
    state: GenesisRam,

    tex: Texture2d,
    ui_tex: Texture2d,

    pal: SrgbTexture2d,
    ui_pal: SrgbTexture2d,

    //plane_shape: Buffer<Vertex>,
    //plane_tiles: Buffer<TilemapCell>,

    plane_a: [Buffer<SpriteTile>; 2],
    plane_b: [Buffer<SpriteTile>; 2],

    sprites: Buffer<SpriteTile>,

    x_offset: f32,
    y_offset: f32,

    frame: usize,

    lerp_next: bool,

    screen_x: f32,
    screen_y: f32,

}

impl S3K {
    pub fn new(driver: &mut Driver, frame: usize, state: GenesisRam) -> Self {
        let ui = GenesisRam::read_gsx(std::fs::File::open("../levelsel.gs0").unwrap()).unwrap();
        let (x_offset, y_offset) = get_pivot(&state.m68k);
        S3K {
            tex: create_texture(driver, &state.v),
            ui_tex: create_texture(driver, &ui.v),
            pal: create_palette(driver, &state.m68k[0xFC00..0xFC80], &state.m68k[0xF080..0xF100]),
            ui_pal: create_palette(driver, &ui.m68k[0xFC00..0xFC80], &ui.m68k[0xF080..0xF100]),

            plane_a: create_plane(driver, &state.m68k, false),
            plane_b: create_plane(driver, &state.m68k, true),
            sprites: {
                let mut v = vec![];
                create_all_sprites(&state.m68k, &state.v, &mut v);
                Buffer::dynamic(driver, v)
            },
            state,
            x_offset,
            y_offset,

            frame,

            lerp_next: true,

            screen_x: -1920.0/2.0,
            screen_y: -1080.0/2.0,
        }
    }
    pub fn update(&mut self, driver: &mut Driver, frame: usize, state: GenesisRam) {
        self.tex = create_texture(driver, &state.v);
        self.pal = create_palette(driver, &state.m68k[0xFC00..0xFC80], &state.m68k[0xF080..0xF100]);
        if &self.state.m68k[0..0xA800] != &state.m68k[0..0xA800] {
            let r = self.state.m68k[0..0xA800].iter().zip(state.m68k[0..0xA800].iter()).position(|(a,b)| a != b).unwrap();
            eprintln!("{:04X}: {} => {}", r, self.state.m68k[r], state.m68k[r]);
            self.plane_a = create_plane(driver, &state.m68k, false);
            self.plane_b = create_plane(driver, &state.m68k, true);
        }

        self.update_offset(&state.m68k);
        let sprites = &mut self.sprites.local;
        sprites.clear();
        create_all_sprites(&state.m68k, &state.v, sprites);
        self.sprites.update(driver);

        self.frame = frame;
        self.state = state;
    }
    pub fn render_chunks(&mut self, driver: &mut Driver, offset: f32) {
        let mut f = vec![];
        let mut f2 = vec![];
        for i in 0..0x8 {
            for j in 0..0x20 {
                add_chunk(&self.state.m68k, [&mut f, &mut f2], i, j, (j*8+i) as u8);
            }
        }
        f.extend(f2.into_iter());
        //eprintln!("{:#?}", f);
        let buf = Buffer::new(driver, f);
        let drawing = Drawing::merged(&buf.buf, (&self.tex, &self.pal))
            .offset(0.0, -offset);
        driver.draw(drawing);
    }
    pub fn render_mappings(&mut self, driver: &mut Driver, offset: f32) {
        let mut f = vec![];
        let mut f2 = vec![];
        for i in 0..0x8 {
            for j in 0..0x20 {
                add_chunk(&self.state.m68k, [&mut f, &mut f2], i, j, (j*8+i) as u8);
            }
        }
        f.extend(f2.into_iter());
        //eprintln!("{:#?}", f);
        let buf = Buffer::new(driver, f);
        let drawing = Drawing::merged(&buf.buf, (&self.tex, &self.pal))
            .offset(0.0, -offset);
        driver.draw(drawing);
    }
    pub fn draw(&mut self, driver: &mut Driver) {
        self.render_tiles(driver);
        self.render_sprites(driver);
        self.render_debug(driver);
    }
    pub fn move_screen(&mut self, x: f32, y: f32) {
        self.screen_x += x;
        self.screen_y += y;
    }
    fn x_offset(&self) -> f32 { (self.x_offset + self.screen_x).round() }
    fn y_offset(&self) -> f32 { (self.y_offset + self.screen_y).round() }
    fn waterline(&self) -> f32 { 
        let ram = &self.state.m68k;
        let has_water = ram[0xF730] != 0;
        if has_water {
            ram.read_u16(0xF648) as f32 - self.y_offset()
        } else {
            10000.0
        }
    }
    fn update_offset(&mut self, ram: &[u8]) {
        let old = &self.state.m68k;
        let loop_height = ram.read_u16(0xEEAA) as f32;
        //self.x_offset = ram.read_u16(0xEE80) as f32;
        //self.y_offset = (ram.read_u16(0xEE84) & loop_height) as f32;

        let is_wrapping = ram.read_i16(0xEE18) < 0;
        let (pivot_x, pivot_y) = get_pivot(ram);

        let cam_delta = (
            ram.read_u16(0xEE80) as i32 - old.read_u16(0xEE80) as i32,
            ram.read_u16(0xEE84) as i32 - old.read_u16(0xEE84) as i32
        );

        if cam_delta.0.abs() + cam_delta.1.abs() > 128 {
            // Silent teleport
            self.x_offset += cam_delta.0 as f32;
            self.y_offset += cam_delta.1 as f32;
            return;
        }

        if pivot_x == 0.0 && pivot_y == 0.0 {
            self.lerp_next = false;
            return;
        }
        if !self.lerp_next {
            self.lerp_next = true;
            self.x_offset = pivot_x;
            self.y_offset = pivot_y;
        }

        self.x_offset += (pivot_x - self.x_offset) / 10.0;
        if is_wrapping && (pivot_y - self.y_offset).abs() > loop_height / 2.0 {
            let pivot_y = if pivot_y > self.y_offset {
                pivot_y - loop_height
            } else {
                pivot_y + loop_height
            };
            self.y_offset += (pivot_y - self.y_offset) / 10.0;
            self.y_offset = self.y_offset.rem_euclid(loop_height);
        } else {
            self.y_offset += (pivot_y - self.y_offset) / 10.0;
        }
    }
    fn render_debug(&mut self, driver: &mut Driver) {
        let ram = &self.state.m68k;
        let text = format!("CAM {:04X}:{:04X}", self.x_offset() as i16, self.y_offset() as i16);
        let zone = ram[0xFE10];
        let act = ram[0xFE11];
        let serc = ram.read_u16(0xEEC0);
        let terc = ram.read_u16(0xEEC2);
        let lprop = ram.read_u16(0xEEC6);
        let text2 = format!("ZONE {:02X}:{:02X} SERC {} TERC {} LPROP {:04X}", zone, act, serc / 4, terc / 4, lprop);

        let rings = ram.read_u16(0xFE20);
        let mins = ram.read_u16(0xFE22);
        let secs = ram[0xFE24];
        let frames = ram[0xFE25];


        let (r_world, r_world_total) = {
            let addr: usize = 0x1A99A;      // ring mappings
            let frame = ram[0xFEB3] as usize;
            let rings_left = ram.read_u32(0xEE42) as usize;
            let rings_start = (0..=rings_left).rev().step_by(4).find(|k| {
                //println!("testing {:06X}", k);
                ROM.read_u32(*k) == 0
            }).unwrap();
            let mut ring_resp = 0xE700;
            let mut r_world = 0;
            let mut r_world_total = 0;
            for i in (rings_start+4..).step_by(4) {
                ring_resp += 2;
                // ring consumed; skip
                let x = ROM.read_u16(i);
                if x == 0xFFFF { break; }  // we are done
                r_world_total += 1;
                if ram.read_u16(ring_resp) != 0x0000 {
                    r_world += 1;
                }
            }
            (r_world, r_world_total)
        };

        let (mut r_monitor, mut r_monitor_total) = (0,0);
        let mut r_giant = 0;
        let mut r_giant_total = 0;

        let sprites = ROM.read_u32(0x1E3D98 + (zone as usize * 2 + act as usize) * 4) as usize;
        let obj_data = &ROM[sprites..];
        let obj_resp_table = &ram[0xEB00..0xEDFF];

        let in_ost = ram[0xB000..0xCFCC].chunks(0x4A).map(|c| c.read_u16(0x48) as usize).filter(|c| *c != 0).collect::<Vec<_>>();

        let w_text = obj_data.chunks(6).take_while(|i| i.read_u16(0) != 0xFFFF).zip(obj_resp_table.iter()).enumerate().flat_map(|(idx,(i,r))| {
            let x = i.read_u16(0) as u32;
            let y = i.read_u16(2) as u32;
            let ranged = y & 0x8000 != 0;
            let vflip = y & 0x4000 != 0;
            let hflip = y & 0x2000 != 0;
            let y = y & 0xFFF;
            let data = i.read_u16(4);
            let resp_flag = r & 0x80 != 0;
            let resp_data = r & 0x7F;

            if data == 0x0103 {
                r_monitor_total += 1;
                if resp_data == 0x01 { r_monitor += 1; }
            }
            if data & 0xFF00 == 0x8500 {
                r_giant_total += 1;
                if 1<<(data & 0x00FF) & ram.read_u32(0xFF92) != 0 { r_giant += 1; }
            }

            let loaded = resp_flag && in_ost.contains(&(idx+0xEB00));
            let destroyed = resp_flag && !loaded;

            let mut line1 = format!("{:04X}", data);
            if resp_data != 0 { line1.push_str(&format!(":{:02X}", resp_data)) }
            let line2 = [ranged, vflip, hflip, loaded, destroyed].iter()
                .zip("RVHLD".chars())
                .filter_map(|(a,b)| if *a { Some(b) } else { None })
                .collect::<String>();
            vec![(x, y, line1), (x, y+8, line2)].into_iter()
        }).collect::<Vec<_>>();
        //self.draw_world_text(driver, &w_text, false);


        let text3 = format!("FRAME {}", self.frame);
        let text4 = format!("TIME  {:02}:{:02}:{:02}", mins, secs, frames);
        let text5 = format!("RINGS {} W {}:{} M {}:{} G {}:{}", rings, r_world, r_world_total, r_monitor, r_monitor_total, r_giant, r_giant_total);

        self.draw_ui_text(driver, &[(8,8,&text),(8,16,&text2),(8,24,&text3),(8,32,&text4),(8,40,&text5)], false);
    }
    fn display_plane_b(&self) -> bool {
        let ram = &self.state.m68k;
        let zone = ram[0xFE10];
        let act = ram[0xFE11];
        let serc = ram.read_u16(0xEEC0);
        let terc = ram.read_u16(0xEEC2);
        let lprop = ram.read_u16(0xEEC6);

        (zone == 0x00 && act == 0x00 && terc >= 0x0C) ||
        (zone == 0x01 && act == 0x01 && terc == 0x04) ||
        (zone == 0x05 && act == 0x00 && terc == 0x04) ||
        (zone == 0x04 && act == 0x01) ||
        (zone == 0x09 && act == 0x00 && terc == 0x04) ||
        (zone == 0x16 && act == 0x00 && terc != 0x00) ||
        (zone == 0x0A)
    }
    fn render_tiles(&mut self, driver: &mut Driver<'_>) {
        let ram = &self.state.m68k;
        let loop_width = ram.read_u16(0x8000) as f32 * 128.0;
        let loop_height = ram.read_u16(0xEEAA) as f32;

        let waterline = self.waterline(); //ram.read_u16(0xF648) as f32 - self.y_offset();


        let cam_x = ram.read_u16(0xEE80) as f32;
        let cam_y = ram.read_u16(0xEE84) as f32;
        let b_cam_x = ram.read_u16(0xEE8C) as f32;
        let b_cam_y = ram.read_u16(0xEE90) as f32;

        // Hide the BG entirely for now
        /*let drawing = Drawing::merged(&self.plane_b[1].buf, (&self.tex, &self.pal))
            .loop_height(loop_height)
            .waterline(waterline)
            .offset(-self.x_offset() + cam_x - b_cam_x, -self.y_offset() + cam_y - b_cam_y);
        driver.draw(drawing);*/
        let drawing = Drawing::merged(&self.plane_a[1].buf, (&self.tex, &self.pal))
            .loop_width(loop_width)
            .loop_height(loop_height)
            .waterline(waterline)
            .offset(-self.x_offset(), -self.y_offset());
        driver.draw(drawing);
        if self.display_plane_b() {
            let drawing = Drawing::merged(&self.plane_b[0].buf, (&self.tex, &self.pal))
                .loop_height(loop_height)
                .waterline(waterline)
                .offset(-self.x_offset() + cam_x - b_cam_x, -self.y_offset() + cam_y - b_cam_y);
            driver.draw(drawing);
        }
        let drawing = Drawing::merged(&self.plane_a[0].buf, (&self.tex, &self.pal))
            .loop_width(loop_width)
            .loop_height(loop_height)
            .waterline(waterline)
            .offset(-self.x_offset(), -self.y_offset());
        driver.draw(drawing);
    }
    fn render_sprites(&mut self, driver: &mut Driver) {
        let ram = &self.state.m68k;
        let loop_height = ram.read_u16(0xEEAA) as f32;
        //let waterline = ram.read_u16(0xF648) as f32 - self.y_offset();
        let waterline = self.waterline(); //ram.read_u16(0xF648) as f32 - self.y_offset();
        let drawing = Drawing::merged(&self.sprites.buf, (&self.tex, &self.pal))
            .loop_height(loop_height)
            .waterline(waterline)
            .offset(-self.x_offset(), -self.y_offset());
        driver.draw(drawing);
    }
    pub fn draw_world_text(&self, driver: &mut Driver, text: &[(u32, u32, impl AsRef<str>)], selected: bool) {
        let ram = &self.state.m68k;
        let loop_height = ram.read_u16(0xEEAA) as f32;

        let tile = |c| "0123456789⭐©:.ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().position(|d| d == c).map(|c| c as u32 + 16).unwrap_or(0);

        let iter = text.iter()
            .flat_map(|(x,y,text)| text.as_ref().chars().enumerate().map(move |(i,ch)| (*x + i as u32 * 8, *y, ch)))
            .map(|(x,y,c)| SpriteTile { position: [x as f32, y as f32], tile_id: tile(c), flip: 0, pal: if selected { 3 } else { 0 } });
        let buf = Buffer::new(driver, iter.collect());
        let drawing = Drawing::merged(&buf.buf, (&self.ui_tex, &self.ui_pal))
            .loop_height(loop_height)
            .offset(-self.x_offset(), -self.y_offset());
        driver.draw(drawing);
    }
    pub fn draw_ui_text(&self, driver: &mut Driver, text: &[(u32, u32, &str)], selected: bool) {
        let tile = |c| "0123456789⭐©:.ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().position(|d| d == c).map(|c| c as u32 + 16).unwrap_or(0);

        let iter = text.iter()
            .flat_map(|(x,y,text)| text.chars().enumerate().map(move |(i,ch)| (*x + i as u32 * 8, *y, ch)))
            .map(|(x,y,c)| SpriteTile { position: [x as f32, y as f32], tile_id: tile(c), flip: 0, pal: if selected { 3 } else { 0 } });
        let buf = Buffer::new(driver, iter.collect());
        let drawing = Drawing::merged(&buf.buf, (&self.ui_tex, &self.ui_pal))
            .scale(3.0);
        driver.draw(drawing);
    }
    pub fn render_vram(&mut self, driver: &mut Driver<'_>, pal: u32) {
        let buf = Buffer::new(driver, (0..0x7FF).map(|c| SpriteTile { position: [(c % 32) as f32 * 8.0, (c / 32) as f32 * 8.0], tile_id: c, flip: 0, pal }).collect());
        let drawing = Drawing::merged(&buf.buf, (&self.tex, &self.pal));
        driver.draw(drawing);
    }
}

fn get_pivot(ram: &[u8]) -> (f32, f32) {
    let loop_height = ram.read_u16(0xEEAA) as f32;

    let is_wrapping = ram.read_i16(0xEE18) < 0;

    let sonic_x = ram.read_i16(0xB010) as f32;
    let mut tails_x = ram.read_i16(0xB010+0x4A) as f32;

    let sonic_y = ram.read_i16(0xB014) as f32;
    let mut tails_y = ram.read_i16(0xB014+0x4A) as f32;

    // Is tails respawning? TODO: try better
    if tails_x == 0.0 || tails_x > 0x7000 as f32 { tails_x = sonic_x; tails_y = sonic_y; }

    // TODO: get viewport
    if tails_x - sonic_x >= 1840.0 {
        tails_x = sonic_x + 1840.0;
    }
    if tails_x - sonic_x <= -1840.0 {
        tails_x = sonic_x - 1840.0;
    }
    // TODO: fix wrapping
    if !is_wrapping {
        if tails_y - sonic_y >= 1000.0 {
            tails_y = sonic_y + 1000.0;
        }
        if tails_y - sonic_y <= -1000.0 {
            tails_y = sonic_y - 1000.0;
        }
    }

    let midpoint_x = (sonic_x + tails_x) / 2.0;
    let mut midpoint_y = (sonic_y + tails_y) / 2.0;

    //eprintln!("Sonic: {} {}", 
    //eprintln!("Pivot: {} {}", pivot_x, pivot_y);

    // Get closer midpoint
    if is_wrapping {
        if (sonic_y - tails_y).abs() > loop_height / 2.0 {
            midpoint_y += loop_height / 2.0;
        }
    }
    (midpoint_x, midpoint_y)
}

fn create_all_sprites(ram: &[u8], vram: &[u8], sprites: &mut Vec<SpriteTile>) {
    let cam_x = ram.read_u16(0xEE80);
    let cam_y = ram.read_u16(0xEE84);

    create_rings(ram, sprites);
    create_objects(ram, sprites);
    create_screen_sprites(vram, cam_x as f32, cam_y as f32, sprites);
}

fn create_objects(ram: &[u8], out: &mut Vec<SpriteTile>) {
    let mut out2 = vec![];
    let object_refs = &ram[0xAC00..0xB000];
    for level in object_refs.chunks(0x80) {
        let len = level.read_u16(0) as usize;
        println!("length: {}", len);
        for i in (2..len+2).step_by(2) {
            let addr = level.read_u16(i) as usize;
            //let obj = &self.img_prev.ram.m68k[addr..addr + 0x4A];
            let obj = &ram[addr..addr+0x4A];
            println!("RENDERED: {:04X}", addr);
            add_object(ram, obj, &mut out2);
        }
    }
    out2.iter_mut().for_each(|c| c.pal += 8);
    out.extend(out2.into_iter());
}

fn add_object(ram: &[u8], obj: &[u8], out: &mut Vec<SpriteTile>) {
    let id = obj.read_u32(0x00) as usize; //((obj[0x01] as usize) << 16) | ((obj[0x02] as usize) << 8) | obj[0x03] as usize;
    let vmask = obj.read_u16(0x0A); //((obj[0x0A] as u16) << 8) | obj[0x0B] as u16;
    let addr = obj.read_u32(0x0C) as usize; //((obj[0x0D] as usize) << 16) | ((obj[0x0E] as usize) << 8) | obj[0x0F] as usize;
    let frame = obj[0x22] as usize;
    let x = obj.read_i16(0x10) as i32;//((obj[0x10] as i32) << 8) | obj[0x11] as i32;
    let y = obj.read_i16(0x14) as i32;//((obj[0x14] as i32) << 8) | obj[0x15] as i32;
    let width = obj[0x06];
    let height = obj[0x07];
    // h/v mirror
    let hm = obj[0x04] & 0x01 != 0;
    let vm = obj[0x04] & 0x02 != 0;
    let is_static = obj[0x04] & 0x20 != 0;
    let is_compound = obj[0x04] & 0x40 != 0;
    let is_visible = obj[0x04] & 0x80 != 0;

    println!("Object {:06X} at {:04X}:{:04X} [{}{}] {}{}", id, x, y,
        if hm { "H" } else { "h" },
        if vm { "V" } else { "v" },
        if is_static { " (static)" } else { "" },
        if is_compound { " (compound)" } else { "" });
    // do not draw if already drawn
    // note: this is largely useless
    let is_offscreen = {
        let x_scr = x - ram.read_u16(0xEE80) as i32;
        let y_scr = y - ram.read_u16(0xEE84) as i32;
        //x_scr+(width as i32) < 0 || x_scr > 320 || y_scr+(height as i32) < 0 || y_scr > 224
        true
    };
    if !is_offscreen && !is_compound { println!("Already on screen or hidden"); return; }

    let x = x as f32;
    let y = y as f32;

    if is_static {
        add_object_map(addr, 1, vmask, x, y, hm, vm, out);
    } else if is_compound {
        let csprites_amt = ((obj[0x16] as usize) << 8) | obj[0x17] as usize;
        if csprites_amt*6+0x18 > obj.len() { return; }
        let cspr_data = &obj[0x18..0x18+csprites_amt*6];
        for i in cspr_data.chunks(6) {
            let x = i.read_u16(0) as f32;
            let y = i.read_u16(2) as f32;
            let frame = i.read_u16(4) as usize;
            let offset = addr+(((ROM[addr+frame*2] as usize) << 8) | ROM[addr+frame*2+1] as usize);
            let len = ((ROM[offset] as usize) << 8) | ROM[offset+1] as usize;
            println!("mappings {:06X}, frame {} => {:06X}, length {}", addr, frame, offset, len);
            add_object_map(offset+2, len, vmask, x, y, hm, vm, out);
        }
    } else {
        if addr+frame*2 > 0x400000 { return; }
        let offset = addr+(((ROM[addr+frame*2] as usize) << 8) | ROM[addr+frame*2+1] as usize);
        let len = ((ROM[offset] as usize) << 8) | ROM[offset+1] as usize;
        println!("mappings {:06X}, frame {} => {:06X}, length {}", addr, frame, offset, len);
        add_object_map(offset+2, len, vmask, x, y, hm, vm, out);
    }
}

fn create_screen_sprites(vram: &[u8], world_x: f32, world_y: f32, out: &mut Vec<SpriteTile>) {
    let mut next = 0;
    let mut i = 0;
    let mut list = (0..).scan(0, |state, _| { *state = get_next_sprite(*state, vram)?; Some(*state) }).collect::<Vec<_>>();
    list.insert(0,0);

    // Ghetto HUD removal

    let hudtiles = vec![
        0xA6CA, 0xA6E2, 0xA6EA, // score
        0xA6DA, 0xA6F2,         // time
        0xA6D2, 0xA6FA,         // rings
        0x87D4, 0xA7D8          // lives
    ];
    let mut hud_done = false;
    let list = list.into_iter().filter(|idx| {
        let sprite = &vram[0xF800+*idx*8..0xF808+*idx*8];
        let y = sprite.read_u16(0) & 0x03FF;
        let x = sprite.read_u16(6) & 0x01FF;
        let tile = sprite.read_u16(4);
        hud_done |= !hudtiles.contains(&tile);
        y != 0 && x != 0 && hud_done
    }).collect::<Vec<_>>();

    for idx in list.iter().rev() {
        let sprite = &vram[0xF800+idx*8..0xF808+idx*8];
        let y = sprite.read_u16(0) & 0x03FF;
        let x = sprite.read_u16(6) & 0x01FF;
        add_sprite(&sprite[2..6], world_x - 128.0 + x as f32, world_y - 128.0 + y as f32, out);
    }
}

fn get_next_sprite(idx: usize, vram: &[u8]) -> Option<usize> {
    let sprite = &vram[0xF800+idx*8..0xF808+idx*8];
    let link = sprite[3] & 0x7F;
    if link == 0 { None } else { Some(link as usize) }
}

fn create_rings(ram: &[u8], tiles: &mut Vec<SpriteTile>) {
    let addr: usize = 0x1A99A;      // ring mappings
    let frame = ram[0xFEB3] as usize;
    let rings_left = ram.read_u32(0xEE42) as usize;
    let rings_start = (0..=rings_left).rev().step_by(4).find(|k| {
        //println!("testing {:06X}", k);
        ROM.read_u32(*k) == 0
    }).unwrap();
    let mut ring_resp = 0xE700;
    for i in (rings_start+4..).step_by(4) {
        ring_resp += 2;
        // ring consumed; skip
        if ram.read_u16(ring_resp) != 0x0000 { continue; }
        let x = ROM.read_u16(i);
        let y = ROM.read_u16(i+2);
        if x == 0xFFFF { break; }  // we are done
        let offset = addr+(ROM.read_u16(addr+frame*2) as usize);
        let len = ROM.read_u16(offset) as usize;

        /*for t in 0..4 {
            let x = x as f32;
            let y = y as f32;
            let cx = (t & 1) as f32 * 8.0;
            let cy = ((t & 2) >> 1) as f32 * 8.0;

            tiles.push(SpriteTile {
                position: [x+cx, y+cy],
                tile_id: i as u32 * 0x20,
                flip: 0,
                pal: 0
            });
        }*/
        //println!("Ring: {:04X}, {:04X}; {:04X}:{:03X}", offset, len, x, y);
        add_object_map(offset+2, len, 0xA6BC, x as f32, y as f32, false, false, tiles);
    }
}

pub fn add_object_map(offset: usize, len: usize, vmask: u16, x: f32, y: f32, h: bool, v: bool, out: &mut Vec<SpriteTile>) {
    // get mappings address
    //let offset = ((ROM[addr+frame*2] as usize) << 8) | ROM[addr+frame*2+1] as usize;
    //let len = ((ROM[addr+offset] as usize) << 8) | ROM[addr+offset+1] as usize;
    //println!("mappings {:06X}, frame {} => {:06X}, length {}", addr, frame, addr+offset, len);
    //println!("mapping offset {:06X}", offset);
    if len > 0x100 { return; }
    if offset+len*6 > 0x400000 { return; }
    let data = &ROM[offset..offset+len*6];
    for i in data.chunks(6).rev() {
    
        let size = i[1];
        let width = (((size & 0x0C) >> 2) + 1)*8;
        let height = ((size & 0x03) + 1)*8;
        
        let mut yoff = i[0] as i8;
        if v {
            yoff = -(height as i8) - yoff;
        }
        let mut xoff = ((i[4] as u16) << 8 | i[5] as u16) as i16;
        if h {
            xoff = -(width as i16) - xoff;
        }
        let vram = (i[2] as u16) << 8 | i[3] as u16;
        let mut tile = vram.wrapping_add(vmask);
        tile ^= (h as u16) << 11;
        tile ^= (v as u16) << 12;
        
        //println!("tile {:04X} + {:04X} size {:X}", vram, vmask, size);
        let spr: [u8;4] = [size, 0, (tile>>8) as u8, tile as u8];
        add_sprite(&spr, x + xoff as f32, y + yoff as f32, out);
        //self.vdp.render_sprite(&spr, x+xoff as i32, y+yoff as i32, canvas);
    }
}

pub fn add_sprite(sprite: &[u8], x: f32, y: f32, out: &mut Vec<SpriteTile>) {
    let tile = ((sprite[2] as u16) << 8) | sprite[3] as u16;
    let prio = sprite[2] & 0x80 != 0;
    let vflip = sprite[2] & 0x10 != 0;
    let hflip = sprite[2] & 0x08 != 0;
    let width = ((sprite[0] & 0x0C) >> 2) + 1;
    let height = (sprite[0] & 0x03) + 1;
    //println!("sprite: {}x{}, prio={} tile {:04X} at ({:03X}, {:03X})", width, height, prio, tile & 0x3FF, x, y);
    for i in 0..(width as usize * height as usize) {
        //let (_, img) = self.render_tile(tile + i as u16);
        let xoff = i as i32 / height as i32;
        let sx = x + ((if hflip { width as i32 - xoff - 1 } else { xoff }) * 8) as f32;
        let yoff = i as i32 % height as i32;
        let sy = y + ((if vflip { height as i32 - yoff - 1 } else { yoff }) * 8) as f32;
        //utils::overlay(&mut out[prio as usize], &img, sx, sy);

        let mut tile = make_tile(tile + i as u16).merge([sx, sy]);
        //tile.flip ^= ((vflip as u32) << 1) + hflip as u32;
        out.push(tile);
    }
}

fn create_texture(driver: &mut Driver, vram: &[u8]) -> Texture2d {
    use crate::ram::GenesisVdp;
    let vdp = GenesisVdp::new(vram.to_vec());
    let tex = vdp.render_vram();
    let dim = tex.dimensions();
    let tex = RawImage2d::from_raw_rgba(tex.into_raw(), dim);
    Texture2d::new(driver.display, tex).unwrap()
}

fn create_palette(driver: &mut Driver, dry: &[u8], wet: &[u8]) -> SrgbTexture2d {
    let values: [u8;8] = [0x00, 0x34, 0x57, 0x74, 0x90, 0xAC, 0xCE, 0xFF];
    let tex = image::ImageBuffer::from_fn(16, 16, |x,y| {
        let arr = if y & 7 >= 4 { wet } else { dry };
        let clr = arr.read_u16((x*2 + (y & 3) * 32) as _) as usize;
        let r = values[(clr >> 1) & 0x07];
        let g = values[(clr >> 5) & 0x07];
        let b = values[(clr >> 9) & 0x07];
        let a = match (x,y) {
            (0, _) => 0,
            (_, 0..=7) => 255,
            (_, 7..=std::u32::MAX) => 127
        };
        image::Rgba([r,g,b,a])
    });
    let dim = tex.dimensions();
    let tex = RawImage2d::from_raw_rgba(tex.into_raw(), dim);
    SrgbTexture2d::new(driver.display, tex).unwrap()
}

fn create_plane(driver: &mut Driver, ram: &[u8], plane_b: bool) -> [Buffer<SpriteTile>;2] {
    let off = plane_b as usize * 2;
    let w = ram[0x8001 + off] as usize;
    let h = ram[0x8005 + off] as usize;

    let mut tiles_lo = vec![];
    let mut tiles_hi = vec![];

    let mut cache = vec![None; 256];

    let inst = std::time::Instant::now();
    for y in 0..h {
        let row_ptr = ram.read_u16(0x8008+y*4 + off) as usize;
        for x in 0..w {
            let chunk = ram[row_ptr+x];
            //add_chunk(&ram, [&mut tiles_lo, &mut tiles_hi], x,y,chunk);
            
            let chunk = cache[chunk as usize].get_or_insert_with(|| {
                let mut tiles_lo = vec![];
                let mut tiles_hi = vec![];
                add_chunk(&ram, [&mut tiles_lo, &mut tiles_hi], 0, 0,chunk);
                (tiles_lo, tiles_hi)
            });
            tiles_lo.extend(chunk.0.iter().map(|c| {
                let mut c = *c;
                c.position[0] += x as f32 * 128.0;
                c.position[1] += y as f32 * 128.0;
                c
            }));
            tiles_hi.extend(chunk.1.iter().map(|c| {
                let mut c = *c;
                c.position[0] += x as f32 * 128.0;
                c.position[1] += y as f32 * 128.0;
                c
            }));
        }
    }
    eprintln!("vertices low : {}", tiles_lo.len());
    eprintln!("vertices high: {}", tiles_hi.len());
    eprintln!("time taken: {:.2}ms", inst.elapsed().as_secs_f64() * 1000.);
    [Buffer::new(driver, tiles_hi), Buffer::new(driver, tiles_lo)]
}

fn add_chunk(ram: &[u8], tiles: [&mut Vec<SpriteTile>; 2], world_x: usize, world_y: usize, chunk: u8) {
    //tiles.reserve(256);

    let chunk = chunk as usize;
    let chunk_data = &ram[0x0000+chunk*0x80..0x0080+chunk*0x80];
    for x in 0..8 {
        for y in 0..8 {
            let block = chunk_data.read_u16(x*2+y*16);
            let vflip = block & 0x0800 != 0;
            let hflip = block & 0x0400 != 0;

            let gflip = ((vflip as u32) << 1) + hflip as u32;

            //let top = t & 0x8000 != 0;
            let block_id = block & 0x03FF;
            let block_id = block_id as usize;
            let map = &ram[0x9000+block_id*8..0x9008+block_id*8];

            let pos = |world: usize, local: usize, flip: bool| (world*128 + local*16 + flip as usize * 8) as f32;

            for i in 0..4 {
                let data = map.read_u16(i*2);
                let prio = data & 0x8000 != 0;
                let mut tile = make_tile(data);
                tile.flip ^= gflip;
                let tile = tile.merge([pos(world_x, x, hflip ^ (i & 1 != 0)), pos(world_y, y, vflip ^ (i & 2 != 0))]);
                tiles[prio as usize].push(tile);
            }
        }
    }
}

fn make_tile(data: u16) -> TilemapCell {
    TilemapCell {
        tile_id: (data & 0x7FF) as u32,
        flip: ((data & 0x1800) >> 11) as u32,
        pal: ((data & 0x6000) >> 13) as u32
    }
}
