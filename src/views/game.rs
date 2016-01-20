use ::phi::{Phi, View, ViewAction};
use ::phi::data::{MaybeAlive, Rectangle};
use ::sdl2::pixels::Color;
use ::phi::gfx::{CopySprite, Sprite, AnimatedSprite, AnimatedSpriteDescr};
use ::views::shared::BgSet;
use ::views::bullets::*;

// Constants

const DEBUG: bool = false;

// pixels traveled by the player every second when moving
const PLAYER_SPEED: f64 = 180.0;
const PLAYER_PATH: &'static str = "assets/spaceship.png";
const PLAYER_TOTAL: usize = 9;
const PLAYER_W: f64 = 43.0;
const PLAYER_H: f64 = 39.0;

// Asteroid constants
const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

// Explosion constants
const EXPLOSION_PATH: &'static str = "assets/explosion.png";
const EXPLOSIONS_WIDE: usize = 5;
const EXPLOSIONS_HIGH: usize = 4;
const EXPLOSIONS_TOTAL: usize = 17;
const EXPLOSION_SIDE: f64 = 96.0;
const EXPLOSION_FPS: f64 = 16.0;
const EXPLOSION_DURATION: f64 = 1.0 / EXPLOSION_FPS * EXPLOSIONS_TOTAL as f64;


// Data types

// Various states the ship could be in
#[derive(Clone, Copy)]
enum PlayerFrame {
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

struct Player {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: PlayerFrame,
    cannon: CannonType,
}

impl Player {

    pub fn new(phi: &mut Phi) -> Player {
        let spritesheet = Sprite::load(&mut phi.renderer, PLAYER_PATH).unwrap();
        let mut sprites = Vec::with_capacity(PLAYER_TOTAL);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle {
                    w: PLAYER_W,
                    h: PLAYER_H,
                    x: PLAYER_W * x as f64,
                    y: PLAYER_H * y as f64,
                }).unwrap());
            }
        }

        Player {
            rect: Rectangle {
                x: 64.0,
                y: (phi.output_size().1 - PLAYER_H) / 2.0,
                w: PLAYER_W,
                h: PLAYER_H,
            },
            sprites: sprites,
            current: PlayerFrame::MidNorm,
            cannon: CannonType::RectBullet,
        }
    }

    // Event handling
    pub fn update(&mut self, phi: &mut Phi, elapsed: f64) {
        // Change player cannons

        if phi.events.now.key_1 == Some(true) {
            self.cannon = CannonType::RectBullet;
        }

        if phi.events.now.key_2 == Some(true) {
            self.cannon = CannonType::SineBullet {
                amplitude: 10.0,
                angular_vel: 15.0,
            };
        }

        if phi.events.now.key_3 == Some(true) {
            self.cannon = CannonType::DivergentBullet {
                a: 100.0,
                b: 1.2,
            };
        }

        // Move the Player

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

        self.rect.x += dx;
        self.rect.y += dy;

        // Boundaries of the playable area
        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: phi.output_size().0 * 0.70, // don't let it go to the far right wall
            h: phi.output_size().1,
        };

        // If player is larger than the screen, abort
        self.rect = self.rect.move_inside(movable_region).unwrap();

        // Select correct ship sprite to show
        self.current =
            if dx == 0.0 && dy < 0.0       { PlayerFrame::UpNorm }
            else if dx > 0.0 && dy < 0.0   { PlayerFrame::UpFast }
            else if dx < 0.0 && dy < 0.0   { PlayerFrame::UpSlow }
            else if dx == 0.0 && dy == 0.0 { PlayerFrame::MidNorm }
            else if dx > 0.0 && dy == 0.0  { PlayerFrame::MidFast }
            else if dx < 0.0 && dy == 0.0  { PlayerFrame::MidSlow }
            else if dx == 0.0 && dy > 0.0  { PlayerFrame::DownNorm }
            else if dx > 0.0 && dy > 0.0   { PlayerFrame::DownFast }
            else if dx < 0.0 && dy > 0.0   { PlayerFrame::DownSlow }
            else { unreachable!() };
    }

    pub fn render(&self, phi: &mut Phi) {
        // Debug bounding box for ship
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200,200,50));
            phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
        }

        // Render ship sprite
        phi.renderer.copy_sprite(
            &self.sprites[self.current as usize],
            self.rect,
        );
    }

    pub fn spawn_bullets(&self) -> Vec<Box<Bullet>> {
        let cannons_x = self.rect.x + 30.0;
        let cannon1_y = self.rect.y + 6.0;
        let cannon2_y = self.rect.y + PLAYER_H - 10.0;

        spawn_bullets(self.cannon, cannons_x, cannon1_y, cannon2_y)
    }
}

struct Asteroid {
    sprite: AnimatedSprite,
    rect: Rectangle,
    vel: f64,
}

impl Asteroid {

    fn factory(phi: &mut Phi) -> AsteroidFactory {
        // Read asteroid sprite sheet and construct animated sprite of it
        AsteroidFactory{
            sprite: AnimatedSprite::with_fps(
                AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
                    image_path: ASTEROID_PATH,
                    total_frames: ASTEROIDS_TOTAL,
                    frames_high: ASTEROIDS_HIGH,
                    frames_wide: ASTEROIDS_WIDE,
                    frame_w: ASTEROID_SIDE,
                    frame_h: ASTEROID_SIDE,
                }),
                1.0),
        }
    }

    fn new(phi: &mut Phi) -> Asteroid {
        let mut asteroid = Asteroid {
            sprite: Asteroid::get_sprite(phi, 15.0),
            rect: Rectangle {
                w: ASTEROID_SIDE,
                h: ASTEROID_SIDE,
                x: 128.0,
                y: 128.0,
            },
            vel: 0.0,
        };

        asteroid.reset(phi);
        asteroid
    }

    fn reset(&mut self, phi: &mut Phi){
        let (w,h) = phi.output_size();

        // Set the fps between 10 and 30
        // random f64 returns value between 0 and 1
        self.sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        self.rect = Rectangle {
            w: ASTEROID_SIDE,
            h: ASTEROID_SIDE,
            x: w,
            y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
        };

        // vel between 50.0 and 150.0
        self.vel = ::rand::random::<f64>().abs() * 100.0 + 50.0;
    }

    fn get_sprite(phi: &mut Phi, fps: f64) -> AnimatedSprite {
        let asteroid_spritesheet = Sprite::load(&mut phi.renderer, ASTEROID_PATH).unwrap();
        let mut asteroid_sprites = Vec::with_capacity(ASTEROIDS_TOTAL);

        for yth in 0..ASTEROIDS_HIGH {
            for xth in 0..ASTEROIDS_WIDE {
                if ASTEROIDS_WIDE * yth + xth >= ASTEROIDS_TOTAL {
                    break;
                }

                asteroid_sprites.push(
                    asteroid_spritesheet.region(Rectangle {
                        w: ASTEROID_SIDE,
                        h: ASTEROID_SIDE,
                        x: ASTEROID_SIDE * xth as f64,
                        y: ASTEROID_SIDE * yth as f64,
                    }).unwrap());
            }
        }

        AnimatedSprite::with_fps(asteroid_sprites, fps)
    }

    fn update(mut self, dt: f64) -> Option<Asteroid> {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);

        if self.rect.x <= -ASTEROID_SIDE {
            None
        }
        else{
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {

        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
        }

        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}

struct AsteroidFactory {
    sprite: AnimatedSprite,
}

impl AsteroidFactory {
    fn random(&self, phi: &mut Phi) -> Asteroid {
        let (w,h) = phi.output_size();

        let mut sprite = self.sprite.clone();
        sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        Asteroid {
            sprite: sprite,
            rect: Rectangle {
                w: ASTEROID_SIDE,
                h: ASTEROID_SIDE,
                x: w,
                y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
            },
            vel: ::rand::random::<f64>().abs() * 100.0 + 50.0,
        }
    }
}

struct Explosion {
    sprite: AnimatedSprite,
    rect: Rectangle,
    alive_since: f64,
}

impl Explosion {
    fn factory(phi: &mut Phi) -> ExplosionFactory {
        ExplosionFactory{
            sprite: AnimatedSprite::with_fps(
                AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
                    image_path: EXPLOSION_PATH,
                    total_frames: EXPLOSIONS_TOTAL,
                    frames_high: EXPLOSIONS_HIGH,
                    frames_wide: EXPLOSIONS_WIDE,
                    frame_w: EXPLOSION_SIDE,
                    frame_h: EXPLOSION_SIDE,
                }),
                EXPLOSION_FPS),
        }
    }

    fn update(mut self, dt: f64) -> Option<Explosion> {
        self.alive_since += dt;
        self.sprite.add_time(dt);

        if self.alive_since >= EXPLOSION_DURATION {
            None
        }
        else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }
}

struct ExplosionFactory {
    sprite: AnimatedSprite,
}

impl ExplosionFactory {
    fn at_center(&self, center: (f64,f64)) -> Explosion {
        let sprite = self.sprite.clone();

        Explosion {
            sprite: sprite,

            // In screen vertically, over right of screen horizontally
            rect: Rectangle::with_size(EXPLOSION_SIDE, EXPLOSION_SIDE)
                .center_at(center),
            alive_since: 0.0,
        }
    }
}

// View definitions

pub struct GameView{
    player: Player,
    // Store bullets behind pointers
    bullets: Vec<Box<Bullet>>,

    asteroids: Vec<Asteroid>,
    asteroid_factory: AsteroidFactory,

    explosions: Vec<Explosion>,
    explosion_factory: ExplosionFactory,

    bg: BgSet,
}

impl GameView {

    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet) -> GameView {
        GameView {
            player: Player::new(phi),
            bullets: vec![],
            asteroids: vec![],
            asteroid_factory: Asteroid::factory(phi),
            explosions: vec![],
            explosion_factory: Explosion::factory(phi),
            bg: bg,
        }
    }
}

impl View for GameView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit {
            return ViewAction::Quit;
        }

        if phi.events.now.key_escape == Some(true) {
            return ViewAction::ChangeView(Box::new(
                ::views::main_menu::MainMenuView::with_backgrounds(
                    phi, self.bg.clone())));
        }

        let old_bullets = ::std::mem::replace(&mut self.bullets, vec![]);

        // Update the player
        self.player.update(phi, elapsed);

        // Update the bullets
        self.bullets = old_bullets.into_iter()
            .filter_map(|bullet| bullet.update(phi, elapsed))
            .collect();

        // Update the asteroids
        self.asteroids =
            ::std::mem::replace(&mut self.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| asteroid.update(elapsed))
            .collect();

        // Update explosions
        self.explosions =
            ::std::mem::replace(&mut self.explosions, vec![])
            .into_iter()
            .filter_map(|explosion| explosion.update(elapsed))
            .collect();

        // Collision detection

        // Track if player is alive
        let mut player_alive = true;

        // Go through bullets and wrap with MaybeAlive to track
        let mut transition_bullets: Vec<_> =
            ::std::mem::replace(&mut self.bullets, vec![])
            .into_iter()
            .map(|bullet| MaybeAlive {alive: true, value: bullet})
            .collect();

        self.asteroids =
            ::std::mem::replace(&mut self.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| {
                // Default, asteroid alive
                let mut asteroid_alive = true;
                for bullet in &mut transition_bullets {
                    if asteroid.rect().overlaps(bullet.value.rect()){
                        bullet.alive = false;
                        asteroid_alive = false;
                    }
                }

                if asteroid.rect().overlaps(self.player.rect) {
                    asteroid_alive = false;
                    player_alive = false;
                }

                if asteroid_alive {
                    Some(asteroid)
                }
                else {
                    // Spawn an explosion at center of asteroid
                    self.explosions.push(
                        self.explosion_factory.at_center(
                            asteroid.rect().center()));
                    None
                }
            })
            .collect();

        // Keep only bullets that are alive
        self.bullets = transition_bullets.into_iter()
            .filter_map(MaybeAlive::as_option)
            .collect();


        // TODO
        if !player_alive {
            println!("The player's ship has been destroyed");
        }

        // Allow the player to shoot after the bullets are updated
        // so they spawn at the tips of the cannons
        if phi.events.now.key_space == Some(true){
            self.bullets.append(&mut self.player.spawn_bullets());
        }

        // Random create a new asteroid about every 100 frames
        if ::rand::random::<usize>() & 100 == 0 {
            self.asteroids.push(self.asteroid_factory.random(phi));
        }

        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0,0,0));
        phi.renderer.clear();

        // Render the backgrounds
        self.bg.back.render(&mut phi.renderer, elapsed);
        self.bg.middle.render(&mut phi.renderer, elapsed);

        // Render the ship
        self.player.render(phi);

        // Render the bullets
        for bullet in &self.bullets {
            bullet.render(phi);
        }

        //Render the asteroids
        for asteroid in &self.asteroids {
            asteroid.render(phi);
        }

        // Render the explosions
        for explosion in &self.explosions {
            explosion.render(phi);
        }

        // Render the foreground
        self.bg.front.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}
