use crate::{
    bvh::{construct_bvh, BvhNode, BvhTree},
    math::Point3,
    object::{
        sample_rect, sample_rect_pdf, sample_sphere, sample_sphere_pdf, sample_tri_pdf,
        sample_triangle, Object,
    },
    random::XorRand,
    ray::{HitRecord, NeeResult, Ray},
    texture::Texture,
};

pub struct Scene<'a> {
    pub objects: Vec<&'a Object<'a>>,
    pub background: Texture<'a>,
    pub lights: Vec<&'a Object<'a>>,
    pub bvh_tree: BvhTree,
}

impl<'a> Scene<'a> {
    pub fn new(mut objs: Vec<&'a Object>, back: Texture<'a>) -> Self {
        let lights = objs
            .clone()
            .into_iter()
            .filter(|obj| obj.get_bxdf().is_light())
            .collect();

        objs.sort_by(|o1, o2| o1.get_obj_id().cmp(&o2.get_obj_id()));
        let bvh_tree = construct_bvh(&objs);

        Scene {
            objects: objs,
            background: back,
            lights,
            bvh_tree,
        }
    }

    pub fn intersect(&self, ray: &Ray, record: &mut HitRecord, node: &BvhNode) -> bool {
        let (l, r) = node.children;
        if node.bbox.hit(ray, record.distance) {
            if l == -1 {
                for i in node.elements.iter() {
                    let _ = self.objects[*i].hit(ray, record);
                }
            } else {
                let _ = self.intersect(ray, record, &self.bvh_tree[l as usize]);
                let _ = self.intersect(ray, record, &self.bvh_tree[r as usize]);
            }
        }
        record.obj_id != -1
    }

    pub fn nee(&self, org: Point3, rand: &mut XorRand) -> NeeResult {
        let mut nee_result = NeeResult::new();
        let mut size = self.lights.len() as u32;

        if let Texture::ImageTex { .. } = self.background {
            size += 1;
        }

        if size == 0 {
            return nee_result;
        }

        let idx = rand.nexti() % size;

        if let Texture::ImageTex {
            cdf,
            cdf_row,
            px_w,
            px_h,
            ..
        } = &self.background
        {
            if idx == size - 1 {
                let (color, dir, pdf) = self
                    .background
                    .sample_hdr(cdf, &cdf_row, *px_w, *px_h, rand);
                if self.intersect(&Ray { org, dir }, &mut HitRecord::new(), &self.bvh_tree[0]) {
                    return nee_result;
                }

                nee_result.color = color;
                nee_result.pdf = pdf / size as f64;
                nee_result.dir = dir;
                return nee_result;
            }
        }

        let obj = self.lights[idx as usize];

        let (pdf, dir, dist) = match obj {
            Object::Sphere { center, radius, .. } => sample_sphere(org, center, *radius, rand),
            Object::Rectangle {
                axis, min_p, max_p, ..
            } => sample_rect(org, axis, max_p, min_p, rand),
            Object::Triangle {
                p, pq, pr, normal, ..
            } => sample_triangle(org, p, pq, pr, normal, obj.get_area(), rand),
        };

        let mut record = HitRecord::init_with_dist(dist + 0.1);
        let _ = self.intersect(&Ray { org, dir }, &mut record, &self.bvh_tree[0]);
        if record.obj_id != obj.get_obj_id() {
            return nee_result;
        }

        nee_result.dir = dir;
        nee_result.color = record.color;
        nee_result.pdf = pdf / size as f64;

        nee_result
    }

    pub fn sample_obj_pdf(&self, org: Point3, record: &HitRecord) -> f64 {
        let obj = self.objects[record.obj_id as usize];
        match obj {
            Object::Sphere { center, radius, .. } => sample_sphere_pdf(org, center, *radius),
            Object::Rectangle { .. } => sample_rect_pdf(org, record.pos, obj, record.normal),
            Object::Triangle { .. } => sample_tri_pdf(org, record.pos, obj, record.normal),
        }
    }
}
