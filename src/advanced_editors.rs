use crate::utils::parse_coordinates;
use gtk::{
    Align, Box, Button, DrawingArea, Entry, EventControllerMotion, GestureClick, Orientation,
    prelude::*,
};
use std::{cell::RefCell, rc::Rc};

const SIZE: f64 = 300.0;

#[derive(Clone, Copy)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

pub struct BezierCurve {
    points: [Point; 4],
    dragging_point_index: Option<usize>,
}

impl BezierCurve {
    fn new() -> Self {
        Self {
            points: [
                Point::new(0.0, SIZE + (SIZE / 3.0)),
                Point::new(SIZE / 3.0, SIZE),
                Point::new(SIZE / 3.0 * 2.0, SIZE / 3.0 * 2.0),
                Point::new(SIZE, SIZE / 3.0),
            ],
            dragging_point_index: None,
        }
    }

    fn new_from_points(points: [Point; 2]) -> Self {
        Self {
            points: [
                Point::new(0.0, SIZE + (SIZE / 3.0)),
                points[0],
                points[1],
                Point::new(SIZE, SIZE / 3.0),
            ],
            dragging_point_index: None,
        }
    }

    fn point_at(&self, index: usize) -> Point {
        self.points[index]
    }

    /// Set point at index to point (if it's C0 or C1)
    fn set_point(&mut self, index: usize, point: Point) {
        if index == 1 || index == 2 {
            let new_x = point.x.clamp(0.0, SIZE);
            let new_y = point.y.clamp(0.0, SIZE / 3.0 * 5.0);
            self.points[index] = Point::new(new_x, new_y);
        }
    }

    fn is_point_near(&self, x: f64, y: f64, index: usize, tolerance: f64) -> bool {
        if index != 1 && index != 2 {
            return false;
        }
        let p = self.point_at(index);
        let dx = p.x - x;
        let dy = p.y - y;
        dx * dx + dy * dy <= tolerance * tolerance
    }

    fn set_points_from_coordinates(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.points[1].x = x0;
        self.points[1].y = y0;
        self.points[2].x = x1;
        self.points[2].y = y1;
    }

    fn find_point_index_at(&self, x: f64, y: f64, tolerance: f64) -> Option<usize> {
        (1..3).find(|&i| self.is_point_near(x, y, i, tolerance))
    }
}

pub fn create_curve_editor(value_entry: &Entry) -> (Box, Button) {
    let bezier = match parse_coordinates(&value_entry.text()) {
        (_, Ok((c0_x, c0_y, c1_x, c1_y))) => Rc::new(RefCell::new(BezierCurve::new_from_points([
            Point::new(c0_x * SIZE, 100.0 + SIZE * (1.0 - c0_y)),
            Point::new(c1_x * SIZE, 100.0 + SIZE * (1.0 - c1_y)),
        ]))),
        (_, Err(_)) => Rc::new(RefCell::new(BezierCurve::new())),
    };

    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .visible(false)
        .build();

    let toggle_button = Button::with_label("Show Editor");

    let vbox_clone = vbox.clone();
    let button_clone = toggle_button.clone();
    toggle_button.connect_clicked(move |_| {
        let is_visible = vbox_clone.is_visible();
        vbox_clone.set_visible(!is_visible);
        button_clone.set_label(if is_visible {
            "Show Editor"
        } else {
            "Hide Editor"
        });
    });

    let drawing_area = DrawingArea::builder()
        .hexpand(false)
        .vexpand(false)
        .halign(Align::Center)
        .valign(Align::Start)
        .build();

    drawing_area.set_size_request(SIZE as i32, (SIZE / 3.0 * 5.0) as i32);

    let bezier_clone = bezier.clone();
    let da_clone = drawing_area.clone();
    value_entry.connect_changed(move |entry| {
        if let (_, Ok((x0, y0, x1, y1))) = parse_coordinates(&entry.text()) {
            bezier_clone.borrow_mut().set_points_from_coordinates(
                x0 * SIZE,
                100.0 + SIZE * (1.0 - y0),
                x1 * SIZE,
                100.0 + SIZE * (1.0 - y1),
            );
        }
        da_clone.queue_draw();
    });

    let bezier_clone = bezier.clone();
    drawing_area.set_draw_func(move |widget, cr, _width, _height| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();

        let scale_factor = widget.scale_factor() as f64;
        cr.scale(scale_factor, scale_factor);

        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(2.0 / scale_factor);
        cr.move_to(0.0, SIZE / 3.0);
        cr.line_to(SIZE, SIZE / 3.0);
        cr.move_to(0.0, SIZE / 3.0 * 4.0);
        cr.line_to(SIZE, SIZE / 3.0 * 4.0);
        cr.stroke().unwrap();

        cr.set_source_rgba(0.8, 0.8, 0.8, 0.6);
        cr.set_line_width(1.0 / scale_factor);

        let grid_step = SIZE / 8.0;

        for i in 0..=8 {
            let x = i as f64 * grid_step;
            cr.move_to(x, 0.0);
            cr.line_to(x, 5.0 * SIZE / 3.0);
        }

        for i in -2..=10 {
            let y = SIZE / 3.0 + i as f64 * grid_step;
            cr.move_to(0.0, y);
            cr.line_to(SIZE, y);
        }

        cr.stroke().unwrap();

        let bez = bezier_clone.borrow();

        cr.set_source_rgba(0.7, 0.7, 0.7, 0.5);
        cr.move_to(bez.point_at(0).x, bez.point_at(0).y);
        cr.line_to(bez.point_at(1).x, bez.point_at(1).y);
        cr.move_to(bez.point_at(2).x, bez.point_at(2).y);
        cr.line_to(bez.point_at(3).x, bez.point_at(3).y);
        cr.stroke().unwrap();

        cr.set_source_rgb(0.15, 0.15, 0.99);
        cr.move_to(bez.point_at(0).x, bez.point_at(0).y);
        cr.curve_to(
            bez.point_at(1).x,
            bez.point_at(1).y,
            bez.point_at(2).x,
            bez.point_at(2).y,
            bez.point_at(3).x,
            bez.point_at(3).y,
        );
        cr.set_line_width(2.0 / scale_factor);
        cr.stroke().unwrap();

        for (i, p) in bez.points.iter().enumerate() {
            match i {
                0 | 3 => cr.set_source_rgb(0.0, 1.0, 0.0),
                1 | 2 => cr.set_source_rgb(1.0, 0.0, 0.0),
                _ => {}
            }
            cr.arc(
                p.x,
                p.y,
                6.0 / scale_factor,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            cr.fill().unwrap();
        }
    });

    let click_gesture = GestureClick::builder().button(0).build();

    let da_clone = drawing_area.clone();
    let bezier_clone = bezier.clone();
    click_gesture.connect_pressed(move |_gesture, _n_press, x, y| {
        let scale_factor = da_clone.scale_factor() as f64;
        let x_scaled = x / scale_factor;
        let y_scaled = y / scale_factor;

        let mut bez = bezier_clone.borrow_mut();
        if let Some(index) = bez.find_point_index_at(x_scaled, y_scaled, 10.0) {
            bez.dragging_point_index = Some(index);
        }
    });

    let bezier_clone = bezier.clone();
    click_gesture.connect_released(move |_gesture, _n_press, _x, _y| {
        let mut bez = bezier_clone.borrow_mut();
        bez.dragging_point_index = None;
    });

    drawing_area.add_controller(click_gesture);

    let update_value_entry_from_coords =
        |entry: &Entry, c0_x: f64, c0_y: f64, c1_x: f64, c1_y: f64| {
            let (name, _) = parse_coordinates(entry.text().as_str());
            let new_value = format!(
                "{}, {:.3}, {:.3}, {:.3}, {:.3}",
                name,
                c0_x / SIZE,
                1.0 - ((c0_y - 100.0) / SIZE),
                c1_x / SIZE,
                1.0 - ((c1_y - 100.0) / SIZE)
            );
            entry.set_text(&new_value);
        };

    let motion_controller = EventControllerMotion::new();
    let bezier_clone = bezier.clone();
    let da_clone = drawing_area.clone();
    let value_entry_clone = value_entry.clone();
    motion_controller.connect_motion(move |_, x, y| {
        let (c0_x, c0_y, c1_x, c1_y) = {
            let bez = bezier_clone.borrow();
            (
                bez.points[1].x,
                bez.points[1].y,
                bez.points[2].x,
                bez.points[2].y,
            )
        };

        update_value_entry_from_coords(&value_entry_clone, c0_x, c0_y, c1_x, c1_y);

        let scale_factor = da_clone.scale_factor() as f64;
        let x_scaled = x / scale_factor;
        let y_scaled = y / scale_factor;

        let mut bez = bezier_clone.borrow_mut();
        if let Some(index) = bez.dragging_point_index
            && (index == 1 || index == 2)
        {
            let new_x = x_scaled.clamp(0.0, SIZE);
            let new_y = y_scaled.clamp(0.0, SIZE / 3.0 * 5.0);
            bez.set_point(index, Point::new(new_x, new_y));
            da_clone.queue_draw();
        }
    });

    drawing_area.add_controller(motion_controller);

    vbox.append(&drawing_area);

    (vbox, toggle_button)
}
