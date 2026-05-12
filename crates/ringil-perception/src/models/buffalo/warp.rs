use image::Rgba32FImage;
use nalgebra::{
    ArrayStorage, Matrix1x2, Matrix2, Matrix2x1, Matrix3, Matrix3x1,
};

// Maps points from source to destination using the calculated transformation
// matrix.
pub fn warp_into(
    input: &Rgba32FImage,
    matrix: Matrix3<f32>,
    output: &mut Rgba32FImage,
) {
    let inverse = matrix
        .try_inverse()
        .expect("Transformation matrix must be invertible");

    let (in_width, in_height) = (input.width() as i32, input.height() as i32);
    let (out_width, out_height) = (output.width(), output.height());

    for out_x in 0..out_width {
        for out_y in 0..out_height {
            let point = Matrix3x1::new(out_x as f32, out_y as f32, 1.0);
            let in_pixel = inverse * point;

            let in_x = in_pixel.x as i32;
            let in_y = in_pixel.y as i32;

            // Nearest-neighbor sampling. Out-of-bounds pixels remain default
            // (transparent/black).
            if in_x >= 0 && in_x < in_width && in_y >= 0 && in_y < in_height {
                output[(out_x, out_y)] =
                    *input.get_pixel(in_x as u32, in_y as u32);
            }
        }
    }
}

// Computes the optimal affine transformation (rotation, scaling, translation)
// between two point sets.
pub fn umeyama<const R: usize>(
    src: &[(f32, f32); R],
    dst: &[(f32, f32); R],
) -> Matrix3<f32> {
    let r_f32 = R as f32;

    let src_mean = (
        src.iter().map(|v| v.0).sum::<f32>() / r_f32,
        src.iter().map(|v| v.1).sum::<f32>() / r_f32,
    );
    let dst_mean = (
        dst.iter().map(|v| v.0).sum::<f32>() / r_f32,
        dst.iter().map(|v| v.1).sum::<f32>() / r_f32,
    );

    let src_demean = nalgebra::Matrix::from_array_storage(ArrayStorage(
        src.map(|v| [v.0 - src_mean.0, v.1 - src_mean.1]),
    ));
    let dst_demean = nalgebra::Matrix::from_array_storage(ArrayStorage(
        dst.map(|v| [v.0 - dst_mean.0, v.1 - dst_mean.1]),
    ));

    // Covariance matrix
    let a = (dst_demean * src_demean.transpose()) / r_f32;
    let svd = a.svd(true, true);

    let mut d = [1.0, 1.0];
    if a.determinant() < 0.0 {
        d[1] = -1.0;
    }

    let mut t = Matrix2::identity();
    let u = svd.u.unwrap();
    let v_t = svd.v_t.unwrap();

    let rank = a.rank(1e-5);
    assert!(rank > 0, "Collinear points, cannot compute transformation.");

    // Handle degenerate cases for rotation matrix estimation
    if rank == 1 {
        if u.determinant() * v_t.determinant() > 0.0 {
            u.mul_to(&v_t, &mut t);
        } else {
            let s = d[1];
            d[1] = -1.0;
            let dg = Matrix2::new(d[0], 0.0, 0.0, d[1]);
            (u * dg).mul_to(&v_t, &mut t);
            d[1] = s;
        }
    } else {
        let dg = Matrix2::new(d[0], 0.0, 0.0, d[1]);
        (u * dg).mul_to(&v_t, &mut t);
    }

    // Scale estimation
    let var0 = src_demean.row(0).variance();
    let var1 = src_demean.row(1).variance();
    let scale =
        (Matrix1x2::new(d[0], d[1]) * svd.singular_values)[0] / (var0 + var1);

    // Translation estimation
    let t_scale = t * scale;
    let t_mean = t_scale * Matrix2x1::new(src_mean.0, src_mean.1);
    let trans = Matrix2x1::new(dst_mean.0, dst_mean.1) - t_mean;

    Matrix3::new(
        t_scale.m11,
        t_scale.m12,
        trans[0],
        t_scale.m21,
        t_scale.m22,
        trans[1],
        0.0,
        0.0,
        1.0,
    )
}
