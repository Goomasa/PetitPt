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
mod filter;
mod material;
mod math;
mod object;
mod pathtracing;
mod polygon;
mod random;
mod ray;
mod render;
mod scene;
mod texture;

pub fn example1() {
    let obj_id = &mut FreshId::new();
    let medium_id = &mut FreshId::new();
    let (data, px_w, px_h) = load_hdr("assets/kloofendal_48d_partly_cloudy_puresky_1k.hdr");
    let cdf = make_cdf_hdr(&data, px_w, px_h);

    let rect = Object::set_rect(
        Axis::Y,
        Vec3(-30., 0., 0.),
        Vec3(30., 0., 60.),
        Bxdf::Lambertian,
        Texture::set_checker(15, Vec3::new(1.), Vec3::new(0.1)),
        obj_id,
    );

    let sphere0 = Object::set_sphere(
        Vec3(-12., 5., 30.),
        4.,
        Bxdf::set_medium(0.1, 0.1, 2),
        Texture::set_solid(Vec3::new(1.)),
        medium_id,
    );

    let sphere1 = Object::set_sphere(
        Vec3(-6., 5., 30.),
        4.,
        Bxdf::set_dielectric(1.5, 0),
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let sphere2 = Object::set_sphere(
        Vec3(6., 5., 30.),
        4.,
        Bxdf::set_spec_co(Vec3(0.188, 0.543, 1.332), Vec3(3.403, 2.231, 1.869)),
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let sphere3 = Object::set_sphere(
        Vec3(18., 5., 30.),
        4.,
        Bxdf::set_microbrdf_co(
            0.3,
            0.05,
            Vec3(0.188, 0.543, 1.332),
            Vec3(3.403, 2.231, 1.869),
        ),
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let objects = vec![&rect, &sphere1, &sphere2, &sphere3];
    let mediums = vec![&sphere0];
    let camera = PinholeModel::new(
        Vec3(0., 10., 70.),
        800,
        450,
        300.,
        Vec3(0., 0., -1.).normalize(),
        230.,
        4,
        4,
    );

    let scene = Scene::new(
        objects,
        mediums,
        Texture::set_image(&data, &cdf, px_w, px_h),
    );

    let _ = render(&camera, &scene);
}

pub fn cornel_box() {
    let obj_id = &mut FreshId::new();
    let medium_id = &mut FreshId::new();

    let rect0 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 0., 0.),
        Vec3(25., 0., -50.),
        Bxdf::Lambertian,
        Texture::set_checker(10, Vec3::new(0.1), Vec3::new(1.)),
        obj_id,
    );
    let rect1 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );
    let rect2 = Object::set_rect(
        Axis::X,
        Vec3(-25., 0., 0.),
        Vec3(-25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(1., 0.1, 0.1)),
        obj_id,
    );
    let rect3 = Object::set_rect(
        Axis::X,
        Vec3(25., 0., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0.1, 1.0, 0.1)),
        obj_id,
    );
    let rect4 = Object::set_rect(
        Axis::Z,
        Vec3(-25., 0., -50.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );
    let rect5 = Object::set_rect(
        Axis::Y,
        Vec3(-5., 49.99, -20.),
        Vec3(5., 49.99, -30.),
        Bxdf::Light,
        Texture::set_solid(Vec3(50., 45., 45.)),
        obj_id,
    );

    //Au: set_spec_co(Vec3(0.188, 0.543, 1.332), Vec3(3.403, 2.231, 1.869))
    //Cu: set_spec_co(Vec3(0.275, 1.116, 1.247), Vec3(3.3726, 2.5956, 2.456))

    let sphere = Object::set_sphere(
        Vec3(15., 7., -13.),
        7.,
        Bxdf::set_microbrdf_co(
            0.3,
            0.05,
            Vec3(0.275, 1.116, 1.247),
            Vec3(3.3726, 2.5956, 2.456),
        ),
        Texture::set_solid(Vec3(0.5, 0.5, 1.)),
        obj_id,
    );

    let medium = Object::set_rect(
        Axis::Z,
        Vec3(-25., 0., 0.1),
        Vec3(25., 50., 0.1),
        Bxdf::set_medium(0., 0.03, 2),
        Texture::set_solid(Vec3::new(0.9)),
        medium_id,
    );

    //using stanford-bunny
    //Stanford Computer Graphics Laboratory
    //http://graphics.stanford.edu/data/3Dscanrep/
    let polygon = read_ply(
        "assets/bun_zipper_res4.ply",
        Vec3(0.5, 0.5, 1.0),
        Bxdf::set_dielectric(1.5, 0),
        200.,
        Vec3(-3., -7.5, -25.),
        obj_id,
    );

    let mut objects = vec![&rect0, &rect1, &rect2, &rect3, &rect4, &rect5, &sphere];
    for obj in polygon.iter() {
        //objects.push(obj);
    }

    let mediums = vec![&medium];
    /*
    let camera = PinholeModel::new(
        Vec3(0., 25., 55.),
        800,
        600,
        40.,
        Vec3(0., 0., -1.).normalize(),
        30.,
        2,
        2,
    );
    */
    let camera = LensModel::new(
        600,
        600,
        Vec3(0., 0., -1.).normalize(),
        Vec3(0., 25., 120.),
        30.,
        2.,
        42.,
        96.,
        100.,
        12,
        12,
    );
    let scene = Scene::new(objects, mediums, Texture::set_solid(Vec3::new(0.)));

    let _ = render(&camera, &scene);
}

fn main() {
    let start = std::time::Instant::now();
    //example1();
    cornel_box();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
