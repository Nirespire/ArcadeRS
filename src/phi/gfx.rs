use ::phi::data::Rectangle;
use ::std::cell::RefCell;
use ::std::path::Path;
use ::std::rc::Rc;
use ::sdl2::render::{Renderer, Texture};
use ::sdl2_image::LoadTexture;


// Common interface for rendering component to region
pub trait Renderable {
    fn render(&self, renderer: &mut Renderer, dest: Rectangle);
}

// Automatically implement a clone trait
#[derive(Clone)]
pub struct Sprite {
    tex: Rc<RefCell<Texture>>,
    src: Rectangle,
}

impl Sprite {
    // Create new sprite by wrapping a texture
    pub fn new(texture:Texture) -> Sprite {
        let tex_query = texture.query();

        Sprite {
            tex: Rc::new(RefCell::new(texture)),
            src: Rectangle {
                w: tex_query.width as f64,
                h: tex_query.height as f64,
                x: 0.0,
                y: 0.0,
            }
        }
    }

    pub fn load(renderer: &Renderer, path: &str) -> Option<Sprite> {
        renderer.load_texture(Path::new(path)).ok().map(Sprite::new)
    }

    // Returns a sprite representing sub-region of current
    // Some if valid, None if invalid
    pub fn region(&self, rect: Rectangle) -> Option<Sprite> {
        let new_src = Rectangle {
            x: rect.x + self.src.x,
            y: rect.y + self.src.y,
            ..rect // same as w: rect.w, h: rect.h
        };

        // Verify subregion is inside current
        if self.src.contains(new_src) {
            Some(Sprite {
                tex: self.tex.clone(),
                src: new_src,
            })
        }
        else{
            None
        }
    }

    // Returns dimensions of region
    pub fn size(&self) -> (f64, f64) {
        (self.src.w, self.src.h)
    }

}

impl Renderable for Sprite {
    fn render(&self, renderer: &mut Renderer, dest: Rectangle) {
        renderer.copy(&mut self.tex.borrow_mut(), self.src.to_sdl(), dest.to_sdl())
    }
}

#[derive(Clone)]
pub struct AnimatedSprite {
    // The frames that will be rendered, in order
    sprites: Rc<Vec<Sprite>>,

    // Second between each frame
    frame_delay: f64,

    // Total time a sprite has been alive
    current_time: f64,
}

impl AnimatedSprite {
    pub fn new(sprites: Vec<Sprite>, frame_delay: f64) -> AnimatedSprite {
        AnimatedSprite {
            sprites: Rc::new(sprites),
            frame_delay: frame_delay,
            current_time: 0.0,
        }
    }

    // Create new animated sprite which goes to next frame fps times per second
    pub fn with_fps(sprites: Vec<Sprite>, fps: f64) -> AnimatedSprite {
        if fps == 0.0 {
            panic!("Passed 0 to AnimatedSprite::with_fps");
        }

        AnimatedSprite::new(sprites, 1.0 / fps)
    }

    // Number of frames of the animation
    pub fn frames(&self) -> usize {
        self.sprites.len()
    }

    // If negative, rewind animation
    pub fn set_frame_delay(&mut self, frame_delay: f64) {
        self.frame_delay = frame_delay;
    }

    // Set number of frames an animation goes through per second
    pub fn set_fps(&mut self, fps: f64) {
        if fps == 0.0 {
            panic!("Passed 0 to AnimatedSprite::set_fps");
        }
        self.set_frame_delay(1.0 / fps);
    }

    pub fn add_time(&mut self, dt: f64) {
        self.current_time += dt;

        // If going back in time, lets us select last frame when current_frame goes negative
        if self.current_time < 0.0 {
            self.current_time = (self.frames() -1) as f64 * self.frame_delay;
        }
    }
}

impl Renderable for AnimatedSprite {
    // Renders the current frame of the sprite
    fn render(&self, renderer: &mut Renderer, dest: Rectangle) {
        let current_frame = (self.current_time / self.frame_delay) as usize % self.frames();

        let sprite = &self.sprites[current_frame];
        sprite.render(renderer, dest);
    }
}

// Trait to render a sprite within an area
pub trait CopySprite<T> {
    fn copy_sprite(&mut self, sprite: &T, dest: Rectangle);
}

impl <'window, T: Renderable> CopySprite<T> for Renderer<'window> {
    fn copy_sprite(&mut self, renderable: &T, dest: Rectangle){
        renderable.render(self, dest);
    }
}
