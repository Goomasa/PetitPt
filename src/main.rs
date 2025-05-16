use camera::{HexLensModel, LensModel, PinholeModel};
use material::Bxdf;
use math::Vec3;
use object::{Axis, Object};
use polygon::read_ply;
use random::FreshId;
use render::render;
use scene::Scene;

mod aabb;
mod bvh;
mod camera;
mod material;
mod math;
mod object;
mod polygon;
mod radiance;
mod random;
mod ray;
mod render;
mod scene;
mod texture;

fn example1() {
    let freshid = &mut FreshId::new();

    let plane = Object::set_rect(
        Axis::Y,
        Vec3(-100., 0., -100.),
        Vec3(100., 0., 100.),
        Bxdf::Lambertian,
        Vec3(0.3, 0.3, 0.3),
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

    let objects = vec![&plane, &sphere1, &sphere2, &sphere3, &sphere4, &sphere5];

    let camera = HexLensModel::new(
        800,
        450,
        Vec3(0., 0., -1.).normalize(),
        Vec3(0., 3., 86.),
        40.,
        2.,
        40.,
        40.,
        150.,
        8,
        8,
    );

    let scene = Scene::new(objects, Vec3::new(0.9));

    let _ = render(&camera, &scene);
}

fn example2() {
    let freshid = &mut FreshId::new();

    let plane = Object::set_rect(
        Axis::Y,
        Vec3(-100., 0., -100.),
        Vec3(100., 0., 100.),
        Bxdf::Lambertian,
        Vec3(0.3, 0.3, 0.3),
        freshid,
    );

    let rect = Object::set_rect(
        Axis::Z,
        Vec3(-15., 0., -15.),
        Vec3(15., 15., -15.),
        Bxdf::Light,
        Vec3(20., 20., 20.),
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
        800,
        450,
        40.,
        Vec3(0., -0.1, -1.).normalize(),
        16.,
        4,
        4,
    );

    let scene = Scene::new(objects, Vec3::new(0.));

    let _ = render(&camera, &scene);
}

pub fn example3() {
    let freshid = &mut FreshId::new();

    let rect0 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 0., 0.),
        Vec3(25., 0., -50.),
        Bxdf::Lambertian,
        Vec3(1., 1., 1.),
        freshid,
    );
    let rect1 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(1., 1., 1.),
        freshid,
    );
    let rect2 = Object::set_rect(
        Axis::X,
        Vec3(-25., 0., 0.),
        Vec3(-25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(1., 0.1, 0.1),
        freshid,
    );
    let rect3 = Object::set_rect(
        Axis::X,
        Vec3(25., 0., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(0.1, 1., 0.1),
        freshid,
    );
    let rect4 = Object::set_rect(
        Axis::Z,
        Vec3(-25., 0., -50.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(0.1, 0.1, 1.),
        freshid,
    );
    let rect5 = Object::set_rect(
        Axis::Y,
        Vec3(-5., 49.99, -20.),
        Vec3(5., 49.99, -30.),
        Bxdf::Light,
        Vec3(25., 25., 25.),
        freshid,
    );

    let sphere = Object::set_sphere(
        Vec3(0., 10., -25.),
        10.,
        Bxdf::MicroBtdf { a: 0.5, ior: 1.5 },
        Vec3::new(1.),
        freshid,
    );

    let objects = vec![&rect0, &rect1, &rect2, &rect3, &rect4, &rect5, &sphere];
    let camera = PinholeModel::new(
        Vec3(0., 25., 55.),
        800,
        600,
        40.,
        Vec3(0., 0., -1.).normalize(),
        30.,
        4,
        4,
    );

    let scene = Scene::new(objects, Vec3::new(0.));

    let _ = render(&camera, &scene);
}

pub fn cornel_box() {
    let freshid = &mut FreshId::new();

    let rect0 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 0., 0.),
        Vec3(25., 0., -50.),
        Bxdf::Lambertian,
        Vec3(1., 1., 1.),
        freshid,
    );
    let rect1 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(1., 1., 1.),
        freshid,
    );
    let rect2 = Object::set_rect(
        Axis::X,
        Vec3(-25., 0., 0.),
        Vec3(-25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(1., 0.1, 0.1),
        freshid,
    );
    let rect3 = Object::set_rect(
        Axis::X,
        Vec3(25., 0., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(0.1, 1., 0.1),
        freshid,
    );
    let rect4 = Object::set_rect(
        Axis::Z,
        Vec3(-25., 0., -50.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(1., 1., 1.),
        freshid,
    );
    let rect5 = Object::set_rect(
        Axis::Y,
        Vec3(-5., 49.99, -20.),
        Vec3(5., 49.99, -30.),
        Bxdf::Light,
        Vec3(25., 25., 25.),
        freshid,
    );

    //using stanford-bunny
    //Stanford Computer Graphics Laboratory
    //http://graphics.stanford.edu/data/3Dscanrep/
    let polygon = read_ply(
        "assets/bun_zipper_res4.ply",
        Vec3(0.1, 0.1, 1.0),
        Bxdf::Lambertian,
        200.,
        Vec3(5., -7.5, -20.),
        freshid,
    );

    let mut objects = vec![&rect0, &rect1, &rect2, &rect3, &rect4, &rect5];
    for obj in polygon.iter() {
        objects.push(obj);
    }

    let camera = PinholeModel::new(
        Vec3(0., 25., 55.),
        800,
        600,
        40.,
        Vec3(0., 0., -1.).normalize(),
        30.,
        4,
        4,
    );

    let scene = Scene::new(objects, Vec3::new(0.));

    let _ = render(&camera, &scene);
}

fn main() {
    let start = std::time::Instant::now();
    //example1();
    //example2();
    example3();
    //cornel_box();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
