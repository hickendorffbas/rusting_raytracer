use std::vec;

use image::{DynamicImage, Rgba, GenericImage};

mod math;
use math::{V3, VectorMath, clamp, max, min};


//Settings:
//TODO: I think we need a scene struct and output settings struct or something like that to organise this better.
const IMG_WIDTH_PX:u32 = 2500;
const IMG_HEIGHT_PX:u32 = 2500;
const FOCAL_LENGTH:f64 = 10.0;
const CAMERA_POSITION:Point = Point { x: 0.0, y: 0.0, z: -FOCAL_LENGTH };
const VIEW_PORT_WIDTH:f64 = 4.0; 
const FADE_DISTANCE_START:f64 = 1000000.0;
const FADE_DISTANCE_END:f64 = 2000000.0;
const SPECULAR_REFLECTION_CONSTANT:f64 = 0.5; //TODO: should (also) be per material, not (only) global
const DIFFUSE_REFLECTION_CONSTANT:f64 = 0.1; //TODO: should (also) be per material, not (only) global
const AMBIENT_REFLECTION_CONSTANT:f64 = 0.1; //TODO: should (also) be per material, not (only) global
const MATERIAL_SHININESS_CONSTANT:f64 = 1.5; //TODO: should (also) be per material, not (only) global
const COLOR_MODE:ColorMode = ColorMode::Light;


#[allow(dead_code)]
enum ColorMode {
    StaticColor,
    Normals,
    Light
}


const VIEW_PORT_HEIGHT:f64 = (IMG_HEIGHT_PX as f64 / IMG_WIDTH_PX as f64) * VIEW_PORT_WIDTH;
const VIEW_PORT_TOP_LEFT:Point = Point { x: -(VIEW_PORT_WIDTH / 2.0), y: -(VIEW_PORT_HEIGHT / 2.0), z: 0.0};

const PIX_SIZE_X:f64 = VIEW_PORT_WIDTH / IMG_WIDTH_PX as f64;
const PIX_SIZE_Y:f64 = VIEW_PORT_HEIGHT / IMG_HEIGHT_PX as f64;
const PIX_X_Y_RATIO_IS_SANE:bool = PIX_SIZE_X - PIX_SIZE_Y < 0.001 && PIX_SIZE_X - PIX_SIZE_Y > -0.001;

#[allow(dead_code)] const fn check_viewport_is_sane() {
    //This function is not actually dead code, but its compile-time only
    if !PIX_X_Y_RATIO_IS_SANE {
        panic!("viewport scaling is not correct!");
    }
}
const _: () = check_viewport_is_sane();


#[allow(dead_code)] const COLOR_BLACK:Color = Color {r: 0.0, g: 0.0, b: 0.0};
#[allow(dead_code)] const COLOR_RED:Color = Color {r: 255.0, g: 0.0, b: 0.0};
#[allow(dead_code)] const COLOR_GREEN:Color = Color {r: 0.0, g: 255.0, b: 0.0};
#[allow(dead_code)] const COLOR_BLUE:Color = Color {r: 0.0, g: 0.0, b: 255.0};
#[allow(dead_code)] const COLOR_PURPLE:Color = Color {r: 255.0, g: 0.0, b: 255.0};
#[allow(dead_code)] const COLOR_YELLOW:Color = Color {r: 255.0, g: 255.0, b: 0.0};
#[allow(dead_code)] const COLOR_GRAY:Color = Color {r: 120.0, g: 120.0, b: 120.0};
#[allow(dead_code)] const COLOR_BROWN:Color = Color {r: 139.0, g: 69.0, b: 19.0};
#[allow(dead_code)] const COLOR_WHITE:Color = Color {r: 255.0, g: 255.0, b: 255.0};


trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}

#[derive(Clone, Debug)]
struct Color {
    r: f64,
    g: f64,
    b: f64,
}

trait ColorMath {
    fn add(&self, other: &Color) -> Color;
    fn subtract(&self, other: &Color) -> Color;
    fn multiply(&self, amount: f64) -> Color;
    fn relative_element_wise_multiply(&self, other: &Color) -> Color; //TODO: how is this operator properly called?
    fn lerp(&self, other: &Color, ratio: f64) -> Color;
}

impl ColorMath for Color {
    fn add(&self, other: &Color) -> Color {
        return Color { r: self.r + other.r, g: self.g + other.g, b: self.b + other.b };
    }

    fn subtract(&self, other: &Color) -> Color {
        return Color { r: self.r - other.r, g: self.g - other.g, b: self.b - other.b };
    }

    fn multiply(&self, amount: f64) -> Color {
        return Color { r: self.r * amount, g: self.g * amount, b: self.b * amount };
    }

    fn relative_element_wise_multiply(&self, other: &Color) -> Color {
        //TODO: this is not correct, since we don't assume light is bounded by 255 anymore (it just gets rescaled to that before saving to a bmp)
        let new_r = (self.r / 255.0) * other.r;
        let new_g = (self.g / 255.0) * other.g;
        let new_b = (self.b / 255.0) * other.b;
        return Color { r: new_r, g: new_g, b: new_b };
    }

    fn lerp(&self, other: &Color, ratio: f64) -> Color {
        return other.subtract(&self).multiply(ratio).add(&self);
    }
}

type Point = V3;
type Direction = V3;

struct Ray {
    origin: Point,
    direction: Direction
}

struct Sphere {
    center: Point,
    radius: f64,
    color: Color,
}

struct Triangle {
    p1: Point,
    p2: Point,
    p3: Point,
    color: Color,
}

struct Light {
    position: Point,
    diffuse_component: Color,
    specular_component: Color,
}

struct Hit {
    point: Point,
    material_color: Color,
    distance: f64,
    surface_normal: Direction,
}

enum Object {
    SphereObject(Sphere),
    TriangleObject(Triangle),
    LightObject(Light),
}


impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let origin_to_center = ray.origin.subtract(&self.center);

        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * origin_to_center.dot(&ray.direction);
        let c = origin_to_center.dot(&origin_to_center) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let solution1 = (-b + discriminant.sqrt()) / 2.0 * a;
        let solution2 = (-b - discriminant.sqrt()) / 2.0 * a;
        let closest_solution = min(solution1, solution2, f64::MAX);
        let intersection = ray.origin.add(&ray.direction.multiply(closest_solution));
        let distance = intersection.subtract(&ray.origin).length();

        let normal = intersection.subtract(&self.center).normalize();

        return Some( Hit { point: intersection, material_color: self.color.clone(), distance: distance, surface_normal: normal } );
    }
}

fn points_are_on_same_side_of_ray(point_to_test_1: &Point, point_to_test_2: &Point,
                                  line_start_point: &Point, line_end_point: &Point) -> bool {

    let boundary_vec = line_start_point.subtract(&line_end_point);
    let point_1_vec = boundary_vec.cross(&point_to_test_1.subtract(&line_end_point));
    let point_2_vec = boundary_vec.cross(&point_to_test_2.subtract(&line_end_point));
    let c = point_1_vec.dot(&point_2_vec);

    return c > 0.0;
}

impl Intersectable for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {

        //Compute the normal, by cross-product of any 2 vectors on the plane (just the first 2 sides)
        let side1 = self.p2.subtract(&self.p1);
        let side2 = self.p3.subtract(&self.p1);
        let normal = side1.cross(&side2);

        //Find the plane of the triangle (Ax + By + Cz + K == 0)
        let plane_k = normal.multiply(-1.0).dot(&self.p1);

        //Compute the point along the ray where it intersects the plane:
        let a = normal.x;
        let b = normal.y;
        let c = normal.z;
        let distance_along_ray = - ((a*ray.origin.x + b*ray.origin.y + c*ray.origin.z + plane_k) / 
                                    (a*ray.direction.x + b*ray.direction.y + c*ray.direction.z));

        if distance_along_ray < 0.0 {
            //This is behind the view port
            return None;
        }

        //Get the point where the ray intersects the plane:
        let intersection = ray.origin.add(&ray.direction.multiply(distance_along_ray));

        //Check if it hits the triangle, first a fast check if it is even in the bounding box of the triangle:
        if intersection.x < min(self.p1.x, self.p2.x, self.p3.x) { return None; }
        if intersection.x > max(self.p1.x, self.p2.x, self.p3.x) { return None; }

        if intersection.y < min(self.p1.y, self.p2.y, self.p3.y) { return None; }
        if intersection.y > max(self.p1.y, self.p2.y, self.p3.y) { return None; }

        if intersection.z < min(self.p1.z, self.p2.z, self.p3.z) { return None; }
        if intersection.z > max(self.p1.z, self.p2.z, self.p3.z) { return None; }

        //Now that we know it is in the bounding box, do the actual intersection check:
        if !points_are_on_same_side_of_ray(&intersection, &self.p1, &self.p2, &self.p3) { return None }
        if !points_are_on_same_side_of_ray(&intersection, &self.p2, &self.p3, &self.p1) { return None }
        if !points_are_on_same_side_of_ray(&intersection, &self.p3, &self.p1, &self.p2) { return None }

        return Some(Hit {
            material_color: self.color.clone(),
            distance: intersection.subtract(&ray.origin).length(),
            point: intersection,
            surface_normal: normal.normalize(),
        });
    }
}

fn ray_through_points(start: Point, end: Point) -> Ray {
    return Ray { direction: end.subtract(&start).normalize(), origin: start }
}

fn get_color_for_hitpoint(hit: Hit, scene: &Vec<Object>) -> Color {

    let computed_color = match COLOR_MODE {
        ColorMode::StaticColor => {
            COLOR_RED
        },
        ColorMode::Normals => {
            Color {r: (hit.surface_normal.x + 1.0) * 127.5,
                   g: (hit.surface_normal.y + 1.0) * 127.5,
                   b: (hit.surface_normal.z + 1.0) * 127.5}
        },
        ColorMode::Light => {
            let mut resulting_light = Color {r: 0.0, g: 0.0, b: 0.0};
            let mut all_light_sources_summed:Color = Color { r: 0.0, g: 0.0, b: 0.0 };

            for obj in scene.iter() {

                match obj {
                    Object::LightObject(light_object) => {

                        //TODO: not sure if I need both specular and diffuse here?
                        all_light_sources_summed = all_light_sources_summed.add(&light_object.diffuse_component);
                        all_light_sources_summed = all_light_sources_summed.add(&light_object.specular_component);

                        let vec_to_light_source = light_object.position.subtract(&hit.point).normalize();
                        let l_dot_n = vec_to_light_source.dot(&hit.surface_normal);

                        let diffuse_light_part = light_object.diffuse_component.multiply(DIFFUSE_REFLECTION_CONSTANT * l_dot_n);
                        let color_before_lighting = &hit.material_color;

                        let diffuse_part = diffuse_light_part.relative_element_wise_multiply(&color_before_lighting);

                        let v_to_camera = CAMERA_POSITION.subtract(&hit.point).normalize();
                        let reflected_ray_direction = hit.surface_normal.multiply(l_dot_n * 2.0).subtract(&vec_to_light_source).normalize();
                        let r_dot_v = reflected_ray_direction.dot(&v_to_camera);

                        resulting_light = resulting_light.add(&diffuse_part);

                        if r_dot_v > 0.0 {
                            let specular_part = light_object.specular_component.multiply((SPECULAR_REFLECTION_CONSTANT * r_dot_v).powf(MATERIAL_SHININESS_CONSTANT));
                            resulting_light = resulting_light.add(&specular_part);
                        }

                        //TODO: do I somehow need to scale the light with the number of lights? (not generally, but maybe some overal light scaling to make tuning easier)
                            //and I need to be able to set (or automatically determine) the sensitivity of the camera (i.e. how we map back to 0-255 for colors)

                    },
                    _ => {}
                }

            }

            let ambient_part = hit.material_color.multiply(AMBIENT_REFLECTION_CONSTANT);
            resulting_light = resulting_light.add(&ambient_part);

            resulting_light
        }
    };

    let result_color = if hit.distance > FADE_DISTANCE_START {
        let ratio = clamp((hit.distance - FADE_DISTANCE_START) / (FADE_DISTANCE_END - FADE_DISTANCE_START), 0.0, 1.0);
        computed_color.lerp(&COLOR_BLACK, ratio)
    } else {
        computed_color
    };

    return result_color;
}


fn send_ray(scene: &Vec<Object>, ray: &Ray) -> Color {
    let mut closest_hit_distance = std::f64::MAX;
    let mut closest_hit:Option<Hit> = None;

    for obj in scene.iter() {
        let opt_hit: Option<Hit> = match obj {
            Object::SphereObject(x) => { x.intersect(&ray) }
            Object::TriangleObject(x) => { x.intersect(&ray) }
            Object::LightObject(_) => { None }
        };

        match opt_hit {
            Some(hit) => {
                if hit.distance < closest_hit_distance {
                    closest_hit_distance = hit.distance;
                    closest_hit = Some(hit);
                }
            },
            _ => {}
        }
    }

    return match closest_hit {
        Some(hit) => get_color_for_hitpoint(hit, &scene),
        _ =>  COLOR_BLACK
    };
}


fn main() {
    let progress_print_interval = if IMG_WIDTH_PX > 1000 { 100 } else { 10 };

    let scene:Vec<Object> = vec![
        Object::SphereObject(Sphere { center: Point { x: 15.0, y: 15.0, z: 150.0 }, radius: 5.0, color: COLOR_GREEN }),
        Object::SphereObject(Sphere { center: Point { x: 15.0, y: 15.0, z: 180.0 }, radius: 5.0, color: COLOR_RED }),
        Object::SphereObject(Sphere { center: Point { x: 15.0, y: 15.0, z: 210.0 }, radius: 5.0, color: COLOR_GREEN }),
        Object::SphereObject(Sphere { center: Point { x: 15.0, y: 15.0, z: 240.0 }, radius: 5.0, color: COLOR_RED }),
        Object::SphereObject(Sphere { center: Point { x: 15.0, y: 15.0, z: 270.0 }, radius: 5.0, color: COLOR_GREEN }),

        Object::TriangleObject(Triangle {p1: Point {x: -10.0, y: -15.0, z: 151.0},
                                         p2: Point {x: -15.0, y: -15.0, z: 150.0},
                                         p3: Point {x: -15.0, y: -10.0, z: 150.0}, color: COLOR_BROWN}),

        Object::TriangleObject(Triangle {p1: Point {x: -10.0, y: 0.0, z: 150.0},
                                         p2: Point {x: -15.0, y: 0.0, z: 250.0},
                                         p3: Point {x: -15.0, y: 5.0, z: 250.0}, color: COLOR_BROWN}),

        Object::TriangleObject(Triangle {p1: Point {x: -10.0, y: 10.0, z: 150.0},
                                         p2: Point {x: -15.0, y: 10.0, z: 151.0},
                                         p3: Point {x: -15.0, y: 15.0, z: 150.0}, color: COLOR_BROWN}),


        Object::LightObject(Light {position: Point { x: -100.0, y: -100.0, z: 0.0 },
                                   diffuse_component: Color {r: 255.0, g: 255.0, b: 255.0},
                                   specular_component: Color {r: 255.0, g: 255.0, b: 255.0}}),
    ];


    let mut img = DynamicImage::new_rgb8(IMG_WIDTH_PX, IMG_HEIGHT_PX);

    for view_port_pixel_x in 0..IMG_WIDTH_PX {
        if view_port_pixel_x % progress_print_interval == 0 {
            println!("scanline: {}", view_port_pixel_x);
        }

        let view_port_coordinate_x = (PIX_SIZE_X * view_port_pixel_x as f64) + VIEW_PORT_TOP_LEFT.x;

        for view_port_pixel_y in 0..IMG_HEIGHT_PX {
            let view_port_coordinate_y = (PIX_SIZE_Y * view_port_pixel_y as f64) + VIEW_PORT_TOP_LEFT.y;

            let view_port_point = Point { x: view_port_coordinate_x.into(),
                                          y: view_port_coordinate_y.into(),
                                          z: CAMERA_POSITION.z + FOCAL_LENGTH };
            let ray = ray_through_points(CAMERA_POSITION, view_port_point);

            let color = send_ray(&scene, &ray);
            let img_color = Rgba([color.r as u8, color.g as u8, color.b as u8, 0]);
            img.put_pixel(view_port_pixel_x, view_port_pixel_y, img_color);
        } 
    }


    //TODO: check which files are present already, and add a suffix
    img.save("image.bmp").unwrap();
}
