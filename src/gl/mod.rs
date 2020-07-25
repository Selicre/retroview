pub mod shaders;
pub mod tiles;

use self::shaders::tiles as stiles;

use glium::glutin::{self, event_loop::EventLoop};
use glium::VertexBuffer;
use glium::texture;
use glium::uniform;
use glium::Surface;
use glium::backend::Facade;

pub fn run(mut each: impl FnMut(&mut Driver<'_>, &mut Vec<glutin::event::WindowEvent>) + 'static) -> ! {
    let evl = EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::PhysicalSize::new(1280.0, 720.0));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_double_buffer(Some(true))
        .with_srgb(true);
    let display = glium::Display::new(wb, cb, &evl).unwrap();

    let program = glium::Program::from_source(&display, stiles::VERT, stiles::FRAG, Some(stiles::GEOM)).unwrap();

    let mut events = vec![];
    let mut counter = 0;
    let dimensions = display.get_context().get_framebuffer_dimensions();
    evl.run(move |ev, _, cfl| {
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *cfl = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                c => { c.to_static().map(|c| events.push(c)); return },
            },
            /*glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },*/
            glutin::event::Event::MainEventsCleared => (),

            _ => return,
        }
        let mut target = display.draw();

        target.clear_color(0.01, 0.01, 0.01, 0.0);
        let mut driver = Driver {
            program: &program,
            display: &display,
            frame: &mut target,
        };
        each(&mut driver, &mut events);
        target.finish().unwrap();
    });
}

pub struct Driver<'a> {
    pub program: &'a glium::Program,
    pub display: &'a glium::Display,
    pub frame: &'a mut glium::Frame,
}

impl<'a> Driver<'a> {
    pub fn draw(&mut self, input: Drawing) -> Result<(),glium::DrawError> {
        let (sw, sh) = self.display.get_framebuffer_dimensions();
        let (w, h) = (sw as f32 / input.scale, sh as f32 / input.scale);

        let [x,y] = input.offset;

        let matrix = cgmath::ortho(0.0-x, w-x, h-y, 0.0-y, -1.0, 10.0);
        let m: [[f32;4];4] = matrix.into();
        let image_dim = input.tileset.0.dimensions();

        let uniforms = uniform! {
            matrix: m,
            loop_height: input.loop_height,
            loop_width: input.loop_width,
            water_level: sh as f32 - input.waterline,
            tex: input.tileset.0.sampled()
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            tex_size: [image_dim.0 as f32, image_dim.1 as f32],
            palette: input.tileset.1.sampled()
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
        };
        let draw_params = glium::draw_parameters::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };
        match input.buffer {
            BufferKind::Split(pos, data) => self.frame.draw(
                (pos,data),
                &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                &self.program,
                &uniforms,
                &draw_params
            ),
            BufferKind::Merged(map) => self.frame.draw(
                map,
                &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                &self.program,
                &uniforms,
                &draw_params
            ),
        }
    }
}

pub type Tileset<'a> = (&'a texture::Texture2d, &'a texture::SrgbTexture2d);


#[derive(Copy, Clone)]
pub struct Drawing<'a> {
    buffer: BufferKind<'a>,
    offset: [f32;2],
    scale: f32,
    waterline: f32,
    loop_height: f32,
    loop_width: f32,
    tileset: Tileset<'a>
}

impl<'a> Drawing<'a> {
    pub fn split(
        pos: &'a VertexBuffer<tiles::Vertex>,
        data: &'a VertexBuffer<tiles::TilemapCell>,
        tileset: Tileset<'a>,
    ) -> Self {
        Drawing {
            buffer: BufferKind::Split(pos, data),
            offset: [0.0;2],
            scale: 1.0,
            waterline: 10000.0,
            loop_height: 65536.0,
            loop_width: 65536.0,
            tileset
        }
    }
    pub fn merged(
        map: &'a VertexBuffer<tiles::SpriteTile>,
        tileset: Tileset<'a>,
    ) -> Self {
        Drawing {
            buffer: BufferKind::Merged(map),
            offset: [0.0;2],
            scale: 1.0,
            waterline: 10000.0,
            loop_height: 65536.0,
            loop_width: 65536.0,
            tileset
        }
    }
    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset[0] += x;
        self.offset[1] += y;
        self
    }
    pub fn clear_offset(&mut self) {
        self.offset = [0.0;2];
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    pub fn waterline(mut self, waterline: f32) -> Self {
        self.waterline = waterline;
        self
    }
    pub fn loop_height(mut self, height: f32) -> Self {
        self.loop_height = height;
        self
    }
    pub fn loop_width(mut self, width: f32) -> Self {
        self.loop_width = width;
        self
    }
}

#[derive(Copy, Clone)]
enum BufferKind<'a> {
    Split(&'a VertexBuffer<tiles::Vertex>, &'a VertexBuffer<tiles::TilemapCell>),
    Merged(&'a VertexBuffer<tiles::SpriteTile>)
}
