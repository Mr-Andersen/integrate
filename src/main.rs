use rayon::prelude::*;

// type Mat3x3 = ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64));


#[allow(non_snake_case)]
fn int2d_par(dx: f64, dy: f64, data: &[&[f64]]) -> f64 {
    data.par_windows(3)
        .map(|rows| {
            rows[0].par_windows(3).map(|vals| (vals[0], vals[1], vals[2]))
                .zip(
                    rows[1].par_windows(3).map(|vals| (vals[0], vals[1], vals[2]))
                )
                .zip(
                    rows[2].par_windows(3).map(|vals| (vals[0], vals[1], vals[2])),
                )
                .map(|((a, b), c)| (a, b, c))
                .map(|((_f00, f01, _f02), (f10, f11, f12), (_f20, f21, f22))| {
                    let Dx_1 = (f11 - f10) / dx;
                    let Dx_2 = (f12 - f11) / dx;
                    let Dy_1 = (f11 - f01) / dy;
                    let Dy_2 = (f21 - f11) / dy;
                    // todo: mean
                    let DyDx = (f22 - f21 - f12 + f11) / dx / dy;
                    let DxDy = DyDx;
                    (
                        f11,
                        (Dx_1 + Dx_2) * 0.5, (Dy_1 + Dy_2) * 0.5,
                        (Dx_2 - Dx_1) / dx, (Dy_2 - Dy_1) / dy,
                        DxDy, DyDx
                    )
                })
                .map(|(fxy, Dx, Dy, DxDx, DyDy, DxDy, DyDx)| {
                    (
                        fxy + Dx * dx * 0.5 + Dy * dy * 0.5
                            + DxDx * dx * dx / 6. + DyDy * dy * dy / 6.
                            + DxDy * dx * dy / 16. + DyDx * dx * dy / 16.
                    ) * dx * dy
                })
                .sum::<f64>()
        }).sum()
}

use std::{
    env::args,
    fs::File,
    io::{BufReader, BufRead}
};

// максимум -- 1e7
fn main() {
    let data: Vec<Vec<f64>>
        = BufReader::with_capacity(
            1024 * 1024,
            File::open(args().skip(1).next().unwrap()).unwrap()
        )
        .lines()
        .map(|s| {
            s.unwrap()
                .split_whitespace()
                .map(|s| s.parse::<f64>().unwrap())
                .collect()
        })
        .collect();
    let solution
        = int2d_par(
            1.0, 1.0,
            &data.iter()
                .map(|v| -> &[f64] { &*v })
                .collect::<Vec<_>>()
        );
    println!("{:e}", solution);
}
