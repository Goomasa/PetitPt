use camera::{HexLensModel, LensModel, PinholeModel};
use material::Bxdf;
use math::Vec3;
use object::{Axis, Object};
use polygon::read_ply;
use random::FreshId;
use render::render;
use scene::Scene;
use texture::{load_hdr, make_cdf_hdr, Texture};

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
        Texture::set_solid(Vec3::new(0.3)),
        freshid,
    );

    let sphere1 = Object::set_sphere(
        Vec3(0., 7., -5.),
        7.,
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0.9, 0.3, 0.)),
        freshid,
    );

    let sphere2 = Object::set_sphere(
        Vec3(10., 4., 2.),
        4.,
        Bxdf::Specular,
        Texture::set_solid(Vec3(0.7, 0., 0.7)),
        freshid,
    );

    let sphere3 = Object::set_sphere(
        Vec3(1., 2., 7.),
        2.,
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0.5, 0.7, 0.)),
        freshid,
    );

    let sphere4 = Object::set_sphere(
        Vec3(-10., 3., 5.),
        3.,
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0., 0.7, 0.7)),
        freshid,
    );

    let sphere5 = Object::set_sphere(
        Vec3(-4., 1., 12.),
        1.,
        Bxdf::Dielectric { ior: 1.5 },
        Texture::set_solid(Vec3::new(0.9)),
        freshid,
    );

    let objects = vec![&plane, &sphere1, &sphere2, &sphere3, &sphere4, &sphere5];

    let camera = HexLensModel::new(
        800,
        450,
        Vec3(0., 0., -1.).normalize(),
        Vec3(0., 5., 86.),
        40.,
        2.,
        40.,
        40.,
        150.,
        8,
        8,
    );

    let scene = Scene::new(objects, Texture::set_solid(Vec3::new(0.7)));

    let _ = render(&camera, &scene);
}

pub fn example2() {
    let freshid = &mut FreshId::new();

    let rect0 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 0., 0.),
        Vec3(25., 0., -50.),
        Bxdf::Lambertian,
        Texture::set_checker(10, Vec3::new(0.1), Vec3::new(1.)),
        freshid,
    );
    let rect1 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(1.)),
        freshid,
    );
    let rect2 = Object::set_rect(
        Axis::X,
        Vec3(-25., 0., 0.),
        Vec3(-25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(1., 0.1, 0.1)),
        freshid,
    );
    let rect3 = Object::set_rect(
        Axis::X,
        Vec3(25., 0., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0.1, 1., 0.1)),
        freshid,
    );
    let rect4 = Object::set_rect(
        Axis::Z,
        Vec3(-25., 0., -50.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0.1, 0.1, 1.)),
        freshid,
    );
    let rect5 = Object::set_rect(
        Axis::Y,
        Vec3(-5., 49.99, -20.),
        Vec3(5., 49.99, -30.),
        Bxdf::Light,
        Texture::set_solid(Vec3::new(25.)),
        freshid,
    );

    let sphere = Object::set_sphere(
        Vec3(0., 10., -25.),
        10.,
        Bxdf::MicroBtdf { a: 0.3, ior: 1.5 },
        Texture::set_solid(Vec3::new(1.)),
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

    let scene = Scene::new(objects, Texture::set_solid(Vec3::new(0.)));

    let _ = render(&camera, &scene);
}

pub fn example3() {
    let freshid = &mut FreshId::new();
    let (data, px_w, px_h) = load_hdr("assets/kloofendal_48d_partly_cloudy_puresky_1k.hdr");
    let cdf = make_cdf_hdr(&data, px_w, px_h);

    let rect = Object::set_rect(
        Axis::Y,
        Vec3(-30., 0., 0.),
        Vec3(30., 0., 60.),
        Bxdf::Lambertian,
        Texture::set_checker(15, Vec3::new(0.1), Vec3::new(0.5)),
        freshid,
    );

    let sphere0 = Object::set_sphere(
        Vec3(-18., 5., 30.),
        4.,
        Bxdf::Specular,
        Texture::set_solid(Vec3::new(1.)),
        freshid,
    );

    let sphere1 = Object::set_sphere(
        Vec3(-6., 5., 30.),
        4.,
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(0.7)),
        freshid,
    );

    let sphere2 = Object::set_sphere(
        Vec3(6., 5., 30.),
        4.,
        Bxdf::MicroBtdf { a: 0.2, ior: 1.5 },
        Texture::set_solid(Vec3::new(1.)),
        freshid,
    );

    let sphere3 = Object::set_sphere(
        Vec3(18., 5., 30.),
        4.,
        Bxdf::MicroBtdf { a: 0.4, ior: 1.5 },
        Texture::set_solid(Vec3::new(1.)),
        freshid,
    );

    let objects = vec![&rect, &sphere0, &sphere1];
    let camera = PinholeModel::new(
        Vec3(0., 10., 70.),
        800,
        450,
        40.,
        Vec3(0., 0., -1.).normalize(),
        30.,
        8,
        8,
    );

    let scene = Scene::new(objects, Texture::set_image(&data, &cdf, px_w, px_h));

    let _ = render(&camera, &scene);
}

pub fn cornel_box() {
    let freshid = &mut FreshId::new();

    let rect0 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 0., 0.),
        Vec3(25., 0., -50.),
        Bxdf::Lambertian,
        Texture::set_checker(10, Vec3::new(0.1), Vec3::new(1.)),
        freshid,
    );
    let rect1 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(1.)),
        freshid,
    );
    let rect2 = Object::set_rect(
        Axis::X,
        Vec3(-25., 0., 0.),
        Vec3(-25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(1., 0.1, 0.1)),
        freshid,
    );
    let rect3 = Object::set_rect(
        Axis::X,
        Vec3(25., 0., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0.1, 1.0, 0.1)),
        freshid,
    );
    let rect4 = Object::set_rect(
        Axis::Z,
        Vec3(-25., 0., -50.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(1.)),
        freshid,
    );
    let rect5 = Object::set_rect(
        Axis::Y,
        Vec3(-5., 49.99, -20.),
        Vec3(5., 49.99, -30.),
        Bxdf::Light,
        Texture::set_solid(Vec3::new(30.)),
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

    let scene = Scene::new(objects, Texture::set_solid(Vec3::new(0.)));

    let _ = render(&camera, &scene);
}

fn main() {
    let start = std::time::Instant::now();
    //example1();
    //example2();
    //example3();
    cornel_box();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
