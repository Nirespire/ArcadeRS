use ::phi::{Phi, View, ViewAction};
use ::phi::data::Rectangle;
use ::sdl2::pixels::Color;
use ::phi::gfx::{CopySprite, Sprite};
use ::views::shared::BgSet;

// Constants

// pixels traveled by the player every second when moving
const PLAYER_SPEED: f64 = 180.0;

const SHIP_W: f64 = 43.0;
const SHIP_H: f64 = 39.0;

const DEBUG: bool = false;

// Data types

// Various states the ship could be in
#[derive(Clone, Copy)]
enum ShipFrame {
    UpNorm = 0,
    UpFast = 1,
    UpSlow = 2,
    MidNorm = 3,
    MidFast = 4,
    MidSlow = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8,
}

struct Ship {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: ShipFrame,
}


// View definitions

pub struct ShipView{
    player: Ship,
    bg: BgSet,
}

impl ShipView {

    #[allow(dead_code)]
    pub fn new(phi: &mut Phi) -> ShipView {
        let bg = BgSet::new(&mut phi.renderer);
        ShipView::with_backgrounds(phi, bg)
    }

    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet) -> ShipView {
        // Load texture from filesystem
        let spritesheet = Sprite::load(&mut phi.renderer, "assets/spaceship.png").unwrap();

        // Can init vec up front if we know number of elements
        let mut sprites = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle{
                    w: SHIP_W,
                    h: SHIP_H,
                    x: SHIP_W * x as f64,
                    y: SHIP_H * y as f64,
                }).unwrap());
            }
        }

        ShipView {
            player: Ship {
                rect: Rectangle {
                    x: 64.0,
                    y: 64.0,
                    w: SHIP_W,
                    h: SHIP_H,
                },
                sprites: sprites,
                current: ShipFrame::MidNorm,
            },
            bg: bg,
        }
    }
}

impl View for ShipView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit {
            return ViewAction::Quit;
        }

        if phi.events.now.key_escape == Some(true) {
            return ViewAction::ChangeView(Box::new(
                ::views::main_menu::MainMenuView::with_backgrounds(
                    phi, self.bg.clone())));
        }

        // Move the Ship

        let diagonal =
            (phi.events.key_up ^ phi.events.key_down) &&
            (phi.events.key_left ^ phi.events.key_right);

        let moved =
            if diagonal { 1.0 / 2.0f64.sqrt() }
            else { 1.0 } * PLAYER_SPEED * elapsed;

        let dx = match(phi.events.key_left, phi.events.key_right) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        let dy = match(phi.events.key_up, phi.events.key_down) {
            (true, true) | (false, false) => 0.0,
            (true, false) => - moved,
            (false, true) => moved,
        };

        self.player.rect.x += dx;
        self.player.rect.y += dy;

        // Boundaries of the playable area
        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: phi.output_size().0 * 0.70, // don't let it go to the far right wall
            h: phi.output_size().1,
        };

        // If player is larger than the screen, abort
        self.player.rect = self.player.rect.move_inside(movable_region).unwrap();

        // Select correct ship sprite to show
        self.player.current =
            if dx == 0.0 && dy < 0.0       { ShipFrame::UpNorm }
            else if dx > 0.0 && dy < 0.0   { ShipFrame::UpFast }
            else if dx < 0.0 && dy < 0.0   { ShipFrame::UpSlow }
            else if dx == 0.0 && dy == 0.0 { ShipFrame::MidNorm }
            else if dx > 0.0 && dy == 0.0  { ShipFrame::MidFast }
            else if dx < 0.0 && dy == 0.0  { ShipFrame::MidSlow }
            else if dx == 0.0 && dy > 0.0  { ShipFrame::DownNorm }
            else if dx > 0.0 && dy > 0.0   { ShipFrame::DownFast }
            else if dx < 0.0 && dy > 0.0   { ShipFrame::DownSlow }
            else { unreachable!() };

        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0,0,0));
        phi.renderer.clear();

        // Render the backgrounds
        self.bg.back.render(&mut phi.renderer, elapsed);
        self.bg.middle.render(&mut phi.renderer, elapsed);

        // Render scene
        // Debug bounding box for ship
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200,200,50));
            phi.renderer.fill_rect(self.player.rect.to_sdl().unwrap());
        }

        // Render ship sprite
        phi.renderer.copy_sprite(
            &self.player.sprites[self.player.current as usize],
            self.player.rect,
        );

        // Render the foreground
        self.bg.front.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}
