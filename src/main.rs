
use std::io::{self, prelude::*};
use std::fs::{self, File};
use std::collections::HashMap;
use std::hash::Hash;

pub mod utils;
pub mod gl;
pub mod viewer;


fn main() {
    let (width, height) = (1920, 1080);

    let mut cursor = (0.0f64,0.0f64);

    let mut dragging = None::<(f64,f64)>;
    let mut playing = false;

    let mut viewer = None;

    let mut args = std::env::args().skip(1);
    let gfx_name = args.next().expect("need gfx name");
    let gfx_name = std::path::PathBuf::from(gfx_name);

    let pal_name = args.next().and_then(|c| {
        Some(c.replace("@", gfx_name.parent()?.to_str()?))
    }).or_else(|| {
        Some(format!("{}.COL", gfx_name.file_stem()?.to_str()?))
    }).expect("need pal name");

    let obj_name = args.next();


    let gfx = snesgfx::from::Tileset::from_bitplane(4, &std::fs::read(&gfx_name).unwrap());
    let pal = snesgfx::from::Palette::from_slice(&std::fs::read(&pal_name).unwrap());

    let mut obj = obj_name.map(|c| {
        c.replace("@", gfx_name.parent().unwrap().to_str().unwrap())
    }).map(|c| std::fs::read(&c).unwrap()).unwrap_or(vec![]);

    obj.truncate(0x2000);


    gl::run(move |driver, ev| {
        let viewer = viewer.get_or_insert_with(|| viewer::Viewer::new(&gfx, &pal, &obj, driver));
        for i in ev.drain(..) {
            use glium::glutin;
            use glutin::event::{self, VirtualKeyCode, WindowEvent::*};
            match i {
                KeyboardInput { input: event::KeyboardInput {
                    state,
                    virtual_keycode: Some(c),
                    ..
                }, .. } => {
                    let pressed = state == event::ElementState::Pressed;
                    if pressed { match c {
                        VirtualKeyCode::Space => playing ^= true,
                        VirtualKeyCode::Q => viewer.move_pal(driver, 1),
                        VirtualKeyCode::A => viewer.move_pal(driver, -1),
                        _ => {}
                    } }
                },
                CursorMoved { position, .. } => {
                    cursor = position.into();
                    if let Some(d) = dragging.as_mut() {
                        let x = d.0 - cursor.0;
                        let y = d.1 - cursor.1;
                        viewer.move_screen(x.round() as f32, y.round() as f32);
                        *d = cursor;
                    }
                },
                MouseInput { state, button: event::MouseButton::Left, .. } => {
                    if state == event::ElementState::Pressed {
                        dragging = Some(cursor);
                    } else {
                        dragging = None;
                    }
                },
                _ => {}
            }
        }
        viewer.draw(driver);
    });
}
