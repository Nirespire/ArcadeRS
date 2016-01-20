use ::phi::Phi;
use ::phi::data::Rectangle;
use ::sdl2::pixels::Color;

//? The velocity shared by all bullets, in pixels per second.
const BULLET_SPEED: f64 = 240.0;

//? The size of the rectangle which will represent the bullet.
const BULLET_W: f64 = 8.0;
const BULLET_H: f64 = 4.0;

pub trait Bullet {
    // Copy the pointer not the value it points to
    fn update(self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>>;

    // Render to screen
    fn render(&self, phi: &mut Phi);

    // Return bounding box
    fn rect(&self) -> Rectangle;
}

#[derive(Clone, Copy)]
struct RectBullet {
    rect: Rectangle,
}

impl Bullet for RectBullet {
    // Update bullet. If it has left the screen, None else Some(update_bullet)
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        let (w, _) = phi.output_size();
        self.rect.x += BULLET_SPEED * dt;

        // If bullet left screen, delete it
        if self.rect.x > w {
            None
        }
        else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}

struct SineBullet {
    pos_x: f64,
    origin_y: f64,
    amplitude: f64,
    angular_vel: f64,
    total_time: f64,
}

impl Bullet for SineBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        self.total_time += dt;

        self.pos_x += BULLET_SPEED * dt;

        let (w, _) = phi.output_size();

        if self.rect().x > w {
            None
        }
        else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        let dy = self.amplitude * f64::sin(self.angular_vel * self.total_time);
        Rectangle{
            x: self.pos_x,
            y: self.origin_y + dy,
            w: BULLET_W,
            h: BULLET_H,
        }
    }
}

struct DivergentBullet {
    pos_x: f64,
    origin_y: f64,
    a: f64,
    b: f64,
    total_time: f64,
}

impl Bullet for DivergentBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        self.total_time += dt;
        self.pos_x += BULLET_SPEED * dt;

        // If the bullet has left the screen, then delete it.
        let (w, h) = phi.output_size();
        let rect = self.rect();

        if rect.x > w || rect.x < 0.0 ||
           rect.y > h || rect.y < 0.0 {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        // We will render this kind of bullet in yellow.
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        let dy = self.a *
                    ((self.total_time / self.b).powi(3) -
                     (self.total_time / self.b).powi(2));

        Rectangle {
            x: self.pos_x,
            y: self.origin_y + dy,
            w: BULLET_W,
            h: BULLET_H,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CannonType {
    RectBullet,
    SineBullet { amplitude: f64, angular_vel: f64},
    DivergentBullet { a:f64, b: f64},
}

pub fn spawn_bullets(
    cannon: CannonType,
    cannons_x: f64,
    cannon1_y: f64,
    cannon2_y: f64) -> Vec<Box<Bullet>> {

    match cannon {
        CannonType::RectBullet =>
            vec![
                Box::new(RectBullet {
                    rect: Rectangle {
                        x: cannons_x,
                        y: cannon1_y,
                        w: BULLET_W,
                        h: BULLET_H,
                    }
                }),
                Box::new(RectBullet {
                    rect: Rectangle {
                        x: cannons_x,
                        y: cannon2_y,
                        w: BULLET_W,
                        h: BULLET_H,
                    }
                }),
            ],

        CannonType::SineBullet{ amplitude, angular_vel} =>
            vec![
                Box::new(SineBullet {
                    pos_x: cannons_x,
                    origin_y: cannon1_y,
                    amplitude: amplitude,
                    angular_vel: angular_vel,
                    total_time: 0.0,
                }),
                Box::new(SineBullet {
                    pos_x: cannons_x,
                    origin_y: cannon2_y,
                    amplitude: amplitude,
                    angular_vel: angular_vel,
                    total_time: 0.0,
                }),
            ],
        CannonType::DivergentBullet { a, b } =>
            vec![
                // If a,b > 0, eventually goes upwards
                Box::new(DivergentBullet {
                    pos_x: cannons_x,
                    origin_y: cannon1_y,
                    a: -a,
                    b: b,
                    total_time: 0.0,
                }),
                // If a,b > 0, eventually goes downwards
                Box::new(DivergentBullet {
                    pos_x: cannons_x,
                    origin_y: cannon2_y,
                    a: a,
                    b: b,
                    total_time: 0.0,
                }),
            ]
    }
}
