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
    pub mediums: Vec<&'a Object<'a>>,
    pub background: Texture<'a>,
    pub lights: Vec<&'a Object<'a>>,
    pub bvh_tree: BvhTree,
}

impl<'a> Scene<'a> {
    pub fn new(mut objs: Vec<&'a Object>, mediums: Vec<&'a Object>, back: Texture<'a>) -> Self {
        objs.shrink_to_fit();
        let lights = objs
            .clone()
            .into_iter()
            .filter(|obj| obj.get_bxdf().is_light())
            .collect();

        objs.sort_by(|o1, o2| o1.get_obj_id().cmp(&o2.get_obj_id()));
        let bvh_tree = construct_bvh(&objs);

        Scene {
            objects: objs,
            mediums,
            background: back,
            lights,
            bvh_tree,
        }
    }

    pub fn intersect_obj(&self, ray: &Ray, record: &mut HitRecord, node: &BvhNode) -> bool {
        let (l, r) = node.children;
        if node.bbox.hit(ray) {
            if l == -1 {
                for i in node.elements.iter() {
                    let _ = self.objects[*i].hit(ray, record);
                }
            } else {
                let _ = self.intersect_obj(ray, record, &self.bvh_tree[l as usize]);
                let _ = self.intersect_obj(ray, record, &self.bvh_tree[r as usize]);
            }
        }
        record.obj_id != -1
    }

    fn intersect_medium(&self, ray: &Ray, record: &mut HitRecord) -> bool {
        for med in self.mediums.iter() {
            med.hit(ray, record);
        }
        record.obj_id != -1
    }

    pub fn intersect(&self, ray: &Ray, record: &mut HitRecord, node: &BvhNode) -> bool {
        let b1 = self.intersect_obj(ray, record, node);
        let b2 = self.intersect_medium(ray, record);
        // evaluate both of them
        b1 || b2
    }

    fn calc_transmittance(&self, ray: &Ray, init_e: f64, max_dist: f64) -> f64 {
        // return (sigma_extinct, distant)
        let mut mlist = vec![(init_e, 0.)];
        for med in self.mediums.iter() {
            let mut record = HitRecord::init_with_dist(max_dist);
            if med.hit(ray, &mut record) {
                let sigma_e = med.get_bxdf().get_sigma_ex();
                if mlist.last().unwrap().0 == sigma_e {
                    mlist.push((0., record.distance));
                } else {
                    mlist.push((sigma_e, record.distance));
                }
            }
        }
        mlist.push((0., max_dist));
        mlist.sort_by(|(_, d1), (_, d2)| d1.total_cmp(d2));

        let mut transmittance = 1.;
        for i in 0..mlist.len() - 1 {
            transmittance *= (-mlist[i].0 * (mlist[i + 1].1 - mlist[i].1)).exp();
        }

        transmittance
    }

    pub fn nee(&self, org: Point3, rand: &mut XorRand, sigma_e: f64) -> (NeeResult, f64) {
        let mut nee_result = NeeResult::new();
        let mut size = self.lights.len() as u32;

        if let Texture::ImageTex { .. } = self.background {
            size += 1;
        }

        if size == 0 {
            return (nee_result, 1.);
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
                if self.intersect_obj(&Ray { org, dir }, &mut HitRecord::new(), &self.bvh_tree[0]) {
                    return (nee_result, 1.);
                }

                nee_result.color = color;
                nee_result.pdf = pdf / size as f64;
                nee_result.dir = dir;
                return (nee_result, 1.);
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
        let ray = Ray { org, dir };
        let _ = self.intersect_obj(&ray, &mut record, &self.bvh_tree[0]);
        if record.obj_id != obj.get_obj_id() {
            return (nee_result, 1.);
        }

        let transmittance = self.calc_transmittance(&ray, sigma_e, dist);

        nee_result.dir = dir;
        nee_result.color = record.color;
        nee_result.pdf = pdf / size as f64;

        (nee_result, transmittance)
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
