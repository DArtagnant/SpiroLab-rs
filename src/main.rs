use eframe::egui;
use log::{debug, error};
use std::f32::consts::PI;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([750.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SpiroLab R edition",
        options,
        Box::new(|cc| Ok(Box::<SpiroApp>::default())),
    );
    Ok(())
}

#[derive(Debug)]
struct SpiroApp {
    large_radius: f32,
    small_radius: f32,
    large_frequency: u32,
    small_frequency: u32,
    interpolate_distance_max: f32,
}

impl Default for SpiroApp {
    fn default() -> Self {
        Self {
            large_radius: 200.0,
            small_radius: 100.0,
            large_frequency: 20,
            small_frequency: 50,
            interpolate_distance_max: 30.0,
        }
    }
}

impl eframe::App for SpiroApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SpiroLab R edition");
            let available_height = ui.available_height();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_height(available_height);
                    ui.set_width(200.0);
                    ui.heading("Valeurs :");
                    ui.add(
                        egui::Slider::new(&mut self.large_radius, 10.0..=500.0)
                            .text("large radius"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.small_radius, 10.0..=500.0)
                            .text("small radius"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.large_frequency, 10..=1000)
                            .text("large frequency"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.small_frequency, 10..=1000)
                            .text("small frequency"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.interpolate_distance_max, 1.0..=100.0)
                            .text("resolution"),
                    );
                });
                ui.vertical(|ui| {
                    ui.set_min_height(available_height);
                    let available_size = ui.available_size();
                    let painter_rect = egui::Rect::from_min_size(ui.min_rect().min, available_size);

                    let painter = ui.painter_at(painter_rect);
                    Spiro::from_frequency(
                        [0.0, 0.0],
                        self.large_radius,
                        self.small_radius,
                        self.large_frequency,
                        self.small_frequency,
                        self.interpolate_distance_max,
                    )
                    .draw(
                        &painter,
                        [available_size[0] / 2.0 + 200.0, available_size[1] / 2.0],
                    );
                });
            });
        });
    }
}

// impl Default for SpiroApp {
//     fn default() -> Self {
//         Self {
//             spiros: vec![Spiro::from_frequency(
//                 [0.0, 0.0],
//                 200.0,
//                 100.0,
//                 20,
//                 30,
//                 20.0,
//             )],
//         }
//     }
// }

#[derive(Debug, Clone, Copy)]
struct SpiroPoint {
    point: [f32; 2],
    circle_angle: f32,
    point_angle: f32,
}

impl SpiroPoint {
    fn calc_point(
        center: [f32; 2],
        large_radius: f32,
        small_radius: f32,
        circle_angle: f32,
        point_angle: f32,
    ) -> Self {
        let small_center_x = center[0] + large_radius * circle_angle.cos();
        let small_center_y = center[1] + large_radius * circle_angle.sin();
        Self {
            point: [
                small_center_x + small_radius * point_angle.cos(),
                small_center_y + small_radius * point_angle.sin(),
            ],
            circle_angle,
            point_angle,
        }
    }

    fn zeros() -> Self {
        Self {
            point: [0.0, 0.0],
            circle_angle: 0.0,
            point_angle: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Spiro {
    center: [f32; 2],
    large_radius: f32,
    small_radius: f32,
    large_angular_velocity: f32,
    small_angular_velocity: f32,
    nb_points: u32,
    interpolate_distance_max: f32,
}

fn distance(point1: [f32; 2], point2: [f32; 2]) -> f32 {
    ((point1[0] - point2[0]).powi(2) + (point1[1] - point2[1]).powi(2)).sqrt()
}

fn average_angle(a: f32, b: f32) -> f32 {
    if (a + PI) % (2.0 * PI) == b {
        ((a + b) / 2.0) % (2.0 * PI)
    } else {
        (a.sin() + b.sin()).atan2(a.cos() + b.cos())
    }
}

impl Spiro {
    fn draw(&self, painter: &egui::Painter, offset: [f32; 2]) {
        let mut point1: Option<SpiroPoint> = None;
        let mut point2: SpiroPoint = SpiroPoint::zeros();
        let mut dbg_nb_points = 0u32;
        for _ in 0..=self.nb_points {
            point1 = Some(point2);
            point2 = SpiroPoint::calc_point(
                self.center,
                self.large_radius,
                self.small_radius,
                (point2.circle_angle + self.large_angular_velocity) % (2.0 * PI),
                (point2.point_angle + self.small_angular_velocity) % (2.0 * PI),
            );

            if let Some(point1) = point1 {
                let mut to_be_constructed =
                    Vec::<SpiroPoint>::with_capacity(2 * self.nb_points as usize);
                to_be_constructed.push(point2);
                to_be_constructed.push(point1);
                let mut operations: u32 = 0;
                while to_be_constructed.len() > 1 && operations < 10_000 {
                    operations += 1;
                    let tbc_point1 = to_be_constructed.pop().unwrap();
                    let tbc_point2 = to_be_constructed.last().unwrap();
                    if distance(tbc_point1.point, tbc_point2.point) < self.interpolate_distance_max
                    {
                        dbg_nb_points += 1;
                        painter.line(
                            vec![
                                egui::pos2(
                                    tbc_point1.point[0] + offset[0],
                                    tbc_point1.point[1] + offset[1],
                                ),
                                egui::pos2(
                                    tbc_point2.point[0] + offset[0],
                                    tbc_point2.point[1] + offset[1],
                                ),
                            ],
                            egui::Stroke::new(3.0, egui::Color32::GREEN),
                        );
                    } else {
                        let middle_point = SpiroPoint::calc_point(
                            self.center,
                            self.large_radius,
                            self.small_radius,
                            average_angle(tbc_point1.circle_angle, tbc_point2.circle_angle),
                            average_angle(tbc_point1.point_angle, tbc_point2.point_angle),
                        );
                        to_be_constructed.push(middle_point);
                        to_be_constructed.push(tbc_point1);
                    }
                }
            }
        }
        println!("Spiro à {dbg_nb_points} points dessiné (interpolation comprise)");
    }

    fn from_frequency(
        center: [f32; 2],
        large_radius: f32,
        small_radius: f32,
        large_frequency: u32,
        small_frequency: u32,
        interpolate_distance_max: f32,
    ) -> Self {
        Self {
            center,
            large_radius,
            small_radius,
            large_angular_velocity: (2.0 * PI) / large_frequency as f32,
            small_angular_velocity: (2.0 * PI) / small_frequency as f32,
            interpolate_distance_max,
            nb_points: num::integer::lcm(large_frequency, small_frequency) + 1,
        }
    }
}
