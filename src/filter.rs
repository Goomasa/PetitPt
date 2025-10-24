use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::math::{dot, Color, Vec3};

#[allow(unused)]
fn gaussian(a_sq: f64, sigma: f64) -> f64 {
    (-a_sq / (2. * sigma * sigma)).exp()
}

#[allow(unused)]
fn kernel_size(sigma_s: u32) -> u32 {
    4 * sigma_s + 1
}

#[allow(unused)]
pub fn filter(data: &Vec<Color>, px_w: i32, px_h: i32, sigma_s: u32, sigma_r: f64) -> Vec<Color> {
    let kernel_size = kernel_size(sigma_s);
    let d = ((kernel_size - 1) / 2) as i32;
    let mut filtered_data = vec![Vec3::new(0.); data.len()];

    filtered_data
        .par_chunks_mut(px_w as usize)
        .enumerate()
        .for_each(|(v, row)| {
            for u in 0..px_w as usize {
                let center_id = v * px_w as usize + u;
                let mut weight = [0.; 3];
                let mut sum = [0.; 3];
                for dv in (-d)..=d {
                    for du in (-d)..=d {
                        let new_u = u as i32 + du;
                        let new_v = v as i32 + dv;
                        if new_u < 0 || new_v < 0 || new_u > px_w - 1 || new_v > px_h - 1 {
                            continue;
                        }
                        let dist_sq = (du * du + dv * dv) as f64;
                        let idx = (new_v * px_w + new_u) as usize;
                        let color = [data[idx].0, data[idx].1, data[idx].2];
                        let col_diff = data[center_id as usize] - data[idx];
                        let col_diff = [col_diff.0, col_diff.1, col_diff.2];
                        for i in 0..3 {
                            let w = gaussian(col_diff[i] * col_diff[i], sigma_r)
                                * gaussian(dist_sq, sigma_s as f64);
                            weight[i] = weight[i] + w;
                            sum[i] = sum[i] + color[i] * w;
                        }
                    }
                }

                row[u].0 = sum[0] / weight[0];
                row[u].1 = sum[1] / weight[1];
                row[u].2 = sum[2] / weight[2];
            }
        });

    filtered_data
}

#[allow(unused)]
pub fn guided_filter(
    data: &Vec<Color>,
    normals: &Vec<Color>,
    px_w: i32,
    px_h: i32,
    sigma_s: u32,
    sigma_r: f64,
    sigma_n: f64,
) -> Vec<Color> {
    let kernel_size = kernel_size(sigma_s);
    let d = ((kernel_size - 1) / 2) as i32;
    let mut filtered_data = vec![Vec3::new(0.); data.len()];

    filtered_data
        .par_chunks_mut(px_w as usize)
        .enumerate()
        .for_each(|(v, row)| {
            for u in 0..px_w as usize {
                let center_id = v * px_w as usize + u;
                let mut weight = [0.; 3];
                let mut sum = [0.; 3];
                for dv in (-d)..=d {
                    for du in (-d)..=d {
                        let new_u = u as i32 + du;
                        let new_v = v as i32 + dv;
                        if new_u < 0 || new_v < 0 || new_u > px_w - 1 || new_v > px_h - 1 {
                            continue;
                        }
                        let dist_sq = (du * du + dv * dv) as f64;
                        let idx = (new_v * px_w + new_u) as usize;
                        let color = [data[idx].0, data[idx].1, data[idx].2];
                        let col_diff = data[center_id as usize] - data[idx];
                        let col_diff = [col_diff.0, col_diff.1, col_diff.2];
                        let n_diff = 1. - dot(normals[center_id as usize], normals[idx]);

                        for i in 0..3 {
                            let w = gaussian(col_diff[i] * col_diff[i], sigma_r)
                                * gaussian(dist_sq, sigma_s as f64)
                                * gaussian(n_diff * n_diff, sigma_n);
                            weight[i] = weight[i] + w;
                            sum[i] = sum[i] + color[i] * w;
                        }
                    }
                }

                row[u].0 = sum[0] / weight[0];
                row[u].1 = sum[1] / weight[1];
                row[u].2 = sum[2] / weight[2];
            }
        });

    filtered_data
}
