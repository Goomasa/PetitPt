use camera::{LensModel, PinholeModel};
use material::Bxdf;
use math::Vec3;
use object::{
    Axis,
    Object::{self, Plane, Sphere},
};
use random::FreshId;
use render::render;
use scene::Scene;

mod camera;
mod material;
mod math;
mod object;
mod radiance;
mod random;
mod ray;
mod render;
mod scene;

fn example1() {
    let plane: Object = Plane {
        axis: Axis::Y,
        pos: 0.,
        bxdf: Bxdf::Lambertian,
        color: Vec3(0.4, 0.4, 0.4),
        id: 0,
    };

    let sphere1: Object = Sphere {
        center: Vec3(0., 8., -5.),
        radius: 7.,
        color: Vec3(1., 1., 1.),
        bxdf: Bxdf::Light,
        id: 1,
    };

    let sphere2: Object = Sphere {
        center: Vec3(10., 4., 2.),
        radius: 4.,
        color: Vec3(0.7, 0., 0.7),
        bxdf: Bxdf::Specular,
        id: 2,
    };

    let sphere3: Object = Sphere {
        center: Vec3(0., 2., 6.),
        radius: 2.,
        color: Vec3(0.7, 0.7, 0.),
        bxdf: Bxdf::Lambertian,
        id: 3,
    };

    let sphere4: Object = Sphere {
        center: Vec3(-10., 3., 5.),
        radius: 3.,
        color: Vec3(0., 0.7, 0.7),
        bxdf: Bxdf::Lambertian,
        id: 4,
    };

    let sphere5: Object = Sphere {
        center: Vec3(-4., 1., 12.),
        radius: 1.,
        color: Vec3(0.99, 0.99, 0.99),
        bxdf: Bxdf::Dielectric { ior: 1.5 },
        id: 5,
    };

    let objects = vec![&plane, &sphere1, &sphere2, &sphere3, &sphere4, &sphere5];
    let camera = LensModel::new(
        0.75,
        400,
        Vec3(0., 0., -1.).normalize(),
        Vec3(0., 3., 60.),
        40.,
        2.,
        30.,
        23.5,
        60.,
        4,
        4,
    );

    let scene = Scene::new(objects, Vec3::new(0.));

    let _ = render(&camera, &scene);
}

fn example2() {
    let freshid = &mut FreshId::new();

    let plane = Object::set_plane(Axis::Y, 0., Bxdf::Lambertian, Vec3(0.4, 0.4, 0.4), freshid);

    let rect = Object::set_rect(
        Axis::Z,
        Vec3(-15., 0., -15.),
        Vec3(15., 15., -15.),
        Bxdf::Light,
        Vec3(10., 10., 10.),
        freshid,
    );

    let sphere1 = Object::set_sphere(
        Vec3(0., 7., -5.),
        7.,
        Bxdf::Lambertian,
        Vec3(0.9, 0.3, 0.),
        freshid,
    );

    let sphere2 = Object::set_sphere(
        Vec3(10., 4., 2.),
        4.,
        Bxdf::Specular,
        Vec3(0.7, 0., 0.7),
        freshid,
    );

    let sphere3 = Object::set_sphere(
        Vec3(1., 2., 7.),
        2.,
        Bxdf::Lambertian,
        Vec3(0.7, 0.7, 0.),
        freshid,
    );

    let sphere4 = Object::set_sphere(
        Vec3(-10., 3., 5.),
        3.,
        Bxdf::Lambertian,
        Vec3(0., 0.7, 0.7),
        freshid,
    );

    let sphere5 = Object::set_sphere(
        Vec3(-4., 1., 12.),
        1.,
        Bxdf::Dielectric { ior: 1.5 },
        Vec3(0.9, 0.9, 0.9),
        freshid,
    );

    let objects = vec![
        &plane, &rect, &sphere1, &sphere2, &sphere3, &sphere4, &sphere5,
    ];
    let camera = PinholeModel::new(
        Vec3(0., 5., 20.),
        0.75,
        600,
        40.,
        Vec3(0., -0.1, -1.).normalize(),
        16.,
        4,
        4,
    );

    let scene = Scene::new(objects, Vec3::new(0.));

    let _ = render(&camera, &scene);
}

fn main() {
    let start = std::time::Instant::now();
    //sample_scene1();
    example2();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
