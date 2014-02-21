use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2, Vec4};
use cgmath::aabb::{Aabb, Aabb2};
use calx::rectutil::RectUtil;
use area;
use area::{Location, Area, is_solid};
use fov::Fov;
use glutil::app::{App, SPRITE_INDEX_START};

pub static FLOOR : uint = SPRITE_INDEX_START;
pub static CUBE : uint = SPRITE_INDEX_START + 1;
pub static XWALL : uint = SPRITE_INDEX_START + 2;
pub static YWALL : uint = SPRITE_INDEX_START + 3;
pub static XYWALL : uint = SPRITE_INDEX_START + 4;
pub static OWALL : uint = SPRITE_INDEX_START + 5;
pub static AVATAR : uint = SPRITE_INDEX_START + 6;
pub static WATER : uint = SPRITE_INDEX_START + 7;
pub static CURSOR_BOTTOM : uint = SPRITE_INDEX_START + 8;
pub static CURSOR_TOP : uint = SPRITE_INDEX_START + 9;

pub struct AreaView<'s> {
    seen: ~Fov,
    remembered: ~Fov,
    pos: Point2<int>,
}

impl<'s> AreaView<'s> {
    pub fn new() -> AreaView<'s> {
        AreaView {
            seen: ~Fov::new(),
            remembered: ~Fov::new(),
            pos: Point2::new(0, 0),
        }
    }

    pub fn draw(&self, area: &Area, app: &mut App) {
        app.set_color(&Vec4::new(0.1f32, 0.3f32, 0.6f32, 1f32));
        // XXX: Horrible prototype code, figure out cleaning.

        let origin = Vec2::new(320.0f32, 180.0f32);

        // Mouse cursoring
        let mouse = app.get_mouse();
        let cursor_chart_pos = screen_to_chart(&mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));

        let mut rect = Aabb2::new(
            screen_to_chart(&Point2::new(0f32, 0f32).add_v(&origin.neg())),
            screen_to_chart(&Point2::new(640f32, 392f32).add_v(&origin.neg())));
        rect = rect.grow(&screen_to_chart(&Point2::new(640f32, 0f32).add_v(&origin.neg())));
        rect = rect.grow(&screen_to_chart(&Point2::new(0f32, 392f32).add_v(&origin.neg())));

        // Draw floors
        for pt in rect.points() {
            let p = Location(pt);
            let offset = chart_to_screen(&pt).add_v(&origin);
            if area.get(&p) == area::Water {
                app.set_color(&Vec4::new(0.0f32, 0.5f32, 1.0f32, 1f32));
                app.draw_sprite(WATER, &offset);
            } else {
                app.set_color(&Vec4::new(0.7f32, 0.7f32, 0.8f32, 1f32));
                app.draw_sprite(FLOOR, &offset);
            }
        }

        // Draw cursor back under the protruding geometry.
        app.set_color(&Vec4::new(1.0f32, 0.4f32, 0.4f32, 1f32));
        app.draw_sprite(CURSOR_BOTTOM, &chart_to_screen(&cursor_chart_pos).add_v(&origin));

        // Draw walls
        for pt in rect.points() {
            let p = Location(pt);
            let offset = chart_to_screen(&pt).add_v(&origin);
            app.set_color(&Vec4::new(0.6f32, 0.5f32, 0.1f32, 1f32));
            if area.get(&p) == area::Wall {
                let left = is_solid(area.get(&(p + Vec2::new(-1, 0))));
                let rear = is_solid(area.get(&(p + Vec2::new(-1, -1))));
                let right = is_solid(area.get(&(p + Vec2::new(0, -1))));

                if left && right && rear {
                    app.draw_sprite(CUBE, &offset);
                    if !is_solid(area.get(&(p + Vec2::new(1, -1)))) ||
                       !is_solid(area.get(&(p + Vec2::new(1, 0)))) {
                        app.draw_sprite(YWALL, &offset);
                    }
                    if !is_solid(area.get(&(p + Vec2::new(-1, 1)))) ||
                       !is_solid(area.get(&(p + Vec2::new(0, 1)))) {
                        app.draw_sprite(XWALL, &offset);
                    }
                    if !is_solid(area.get(&(p + Vec2::new(1, 1)))) {
                        app.draw_sprite(OWALL, &offset);
                    }
                } else if left && right {
                    app.draw_sprite(XYWALL, &offset);
                } else if left {
                    app.draw_sprite(XWALL, &offset);
                } else if right {
                    app.draw_sprite(YWALL, &offset);
                } else {
                    app.draw_sprite(OWALL, &offset);
                };
            }

            if p == Location(Point2::new(0i8, 0i8)) {
                app.set_color(&Vec4::new(0.9f32, 0.9f32, 1.0f32, 1f32));
                app.draw_sprite(AVATAR, &offset);
            }
        }

        app.set_color(&Vec4::new(1.0f32, 0.4f32, 0.4f32, 1f32));
        app.draw_sprite(CURSOR_TOP, &chart_to_screen(&cursor_chart_pos).add_v(&origin));
    }
}

pub fn chart_to_screen(map_pos: &Point2<i8>) -> Point2<f32> {
    Point2::new(
        16.0 * (map_pos.x as f32) - 16.0 * (map_pos.y as f32),
        8.0 * (map_pos.x as f32) + 8.0 * (map_pos.y as f32))
}

pub fn screen_to_chart(screen_pos: &Point2<f32>) -> Point2<i8> {
    let column = (screen_pos.x / 16.0).floor();
    let row = ((screen_pos.y - column * 8.0) / 16.0).floor();
    Point2::new((column + row) as i8, row as i8)
}
