// Tells compiler to import macros from events file
// Cannot be done in namespace since this is a preprocessor action
#[macro_use]
mod events;
pub mod data;

use ::sdl2::render::Renderer;
use ::sdl2::pixels::Color;

// Call macro function like normal code
struct_events!{
    keyboard: {
        key_escape: Escape,
        key_space: Space,
        key_down: Down,
        key_up: Up,
        key_right: Right,
        key_left: Left
    },
    else: {
        quit: Quit { .. }
    }
}

// Wrapper to easily write
//  view.render(&mut context, elapsed)
// instead of
//  view.render(&mut renderer, &mut events, elapsed)
pub struct Phi<'window> {
    pub events: Events,
    pub renderer: Renderer<'window>,
}

impl<'window> Phi<'window> {
    pub fn output_size(&self) -> (f64, f64) {
        let (w,h) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
    }
}

// ViewAction is a way for for currently executed/rendered view
// to tell the main game loop what should be executed before the next frame
pub enum ViewAction {
    None,
    Quit,
    ChangeView(Box<View>),
}

// Way to implement shared methods between views
pub trait View {
    // Called on every frame
    // Deals with logic and rendering of current view
    // "elapsed" is in seconds
    fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}


// Closures!
// What used to be in main, now main game loop is modularized out
pub fn spawn<F>(title: &str, init: F)
where F: Fn(&mut Phi) -> Box<View> {
    // Initialize SDL2
    let sdl_context = ::sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();

    // Create the window
    let window = video.window(title, 800, 600)
        .position_centered().opengl().resizable()
        .build().unwrap();

    // Create the context
    let mut context = Phi {
        events: Events::new(sdl_context.event_pump().unwrap()),
        renderer: window.renderer()
            .accelerated()
            .build().unwrap(),
    };

    // Create the default view
    let mut current_view = init(&mut context);

    // Frame timing
    let interval = 1_000 / 60;
    let mut before = timer.ticks();
    let mut last_second = timer.ticks();
    let mut fps = 0u16;

    // Main game loop
    loop {

        // Update frame timing
        let now = timer.ticks();
        let dt = now - before;
        let elapsed = dt as f64 / 1_000.0;

        // If elapsed time is too short, wait and try again
        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        fps += 1;

        if now - last_second > 1_000 {
            println!("FPS: {}", fps);
            last_second = now;
            fps = 0;
        }

        // Logic and rendering
        context.events.pump(&mut context.renderer);

        match current_view.render(&mut context, elapsed) {
            ViewAction::None => context.renderer.present(),
            ViewAction::Quit => break,
            ViewAction::ChangeView(new_view) => current_view = new_view,
        }
    }
}
