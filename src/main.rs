use camera::{LensModel, PinholeModel};
use material::Bxdf;
use math::Vec3;
use object::{
    Axis,
    Object::{self, Plane, Sphere},
};
use polygon::read_ply;
use random::FreshId;
use render::render;
use scene::Scene;

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

fn example1() {
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

    let camera = LensModel::new(
        0.75,
        800,
        Vec3(0., 0., -1.).normalize(),
        Vec3(0., 3., 50.),
        40.,
        2.,
        30.,
        23.,
        50.,
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
        6,
        6,
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

    let sphere0 = Object::set_sphere(
        Vec3(5., 5., -10.),
        5.,
        Bxdf::Lambertian,
        Vec3(1., 0.2, 1.),
        freshid,
    );

    let tri0 = Object::set_tri(
        Vec3(-20., 0., -5.),
        Vec3(-15., 20., -10.),
        Vec3(-8., 0., -25.),
        Bxdf::Specular,
        Vec3(0.3, 0.3, 1.),
        freshid,
    );

    let objects = vec![
        &rect0, &rect1, &rect2, &rect3, &rect4, &rect5, &sphere0, &tri0,
    ];
    let camera = PinholeModel::new(
        Vec3(0., 25., 55.),
        0.75,
        800,
        40.,
        Vec3(0., 0., -1.).normalize(),
        30.,
        12,
        12,
    );

    let scene = Scene::new(objects, Vec3::new(0.));

    let _ = render(&camera, &scene);
}

fn bunny() {
    let freshid = &mut FreshId::new();

    let plane = Object::set_plane(Axis::Y, 0., Bxdf::Lambertian, Vec3(0.4, 0.4, 0.4), freshid);

    let rect = Object::set_rect(
        Axis::Z,
        Vec3(-20., 0., -15.),
        Vec3(20., 30., -15.),
        Bxdf::Light,
        Vec3(5., 5., 5.),
        freshid,
    );

    //using stanford-bunny
    //Stanford Computer Graphics Laboratory
    //http://graphics.stanford.edu/data/3Dscanrep/
    let polygon = read_ply(
        "assets/bun_zipper_res4.ply",
        Vec3(0.8, 0.5, 0.8),
        Bxdf::Lambertian,
        100.,
        Vec3(0., -3.5, 0.),
        freshid,
    );

    let mut objects = vec![&rect, &plane];
    for obj in polygon.iter() {
        objects.push(obj);
    }

    let camera = PinholeModel::new(
        Vec3(0., 5., 20.),
        0.75,
        400,
        40.,
        Vec3(0., -0.1, -1.).normalize(),
        16.,
        6,
        6,
    );

    let scene = Scene::new(objects, Vec3::new(0.));

    let _ = render(&camera, &scene);
}

fn main() {
    let start = std::time::Instant::now();
    //example1();
    //example2();
    //cornel_box();
    bunny();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
