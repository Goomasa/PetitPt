#[derive(Clone, Copy)]
pub enum Bxdf {
    Lambertian,
    Specular,
    Dielectric { ior: f64 },
    Light,
}
