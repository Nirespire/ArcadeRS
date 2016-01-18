use ::sdl2::render::Renderer;
use :: phi::data::Rectangle;
use ::phi::gfx::{CopySprite, Sprite};

#[derive(Clone)]
pub struct Background {
    pub pos: f64,
    // The amount of pixels moved left every second
    pub vel: f64,
    pub sprite: Sprite,
}

impl Background {
    pub fn render(&mut self, renderer: &mut Renderer, elapsed: f64) {
        // Determine position based on time and image dimension, not screen size
        let size = self.sprite.size();
        self.pos += self.vel * elapsed;
        if self.pos > size.0 {
            self.pos -= size.0;
        }

        // Determine scale ratio of window to sprite
        let (win_w, win_h) = renderer.output_size().unwrap();
        let scale = win_h as f64 / size.1;

        // Render as many copies of background as necessary to fill screen
        let mut physical_left = -self.pos * scale;

        while physical_left < win_w as f64 {
            // While left of image is still inside window
            renderer.copy_sprite(&self.sprite, Rectangle {
                x: physical_left,
                y: 0.0,
                w: size.0 * scale,
                h: win_h as f64,
            });

            physical_left += size.0 * scale;
        }
    }
}

#[derive(Clone)]
pub struct BgSet {
    pub back: Background,
    pub middle: Background,
    pub front: Background,
}

impl BgSet {
    pub fn new(renderer: &mut Renderer) -> BgSet {
        BgSet{
            back: Background {
                pos: 0.0,
                vel: 20.0,
                sprite: Sprite::load(renderer, "assets/starBG.png").unwrap(),
            },
            middle: Background {
                pos: 0.0,
                vel: 40.0,
                sprite: Sprite::load(renderer, "assets/starMG.png").unwrap(),
            },
            front: Background {
                pos: 0.0,
                vel: 80.0,
                sprite: Sprite::load(renderer, "assets/starFG.png").unwrap(),
            },
        }
    }
}
