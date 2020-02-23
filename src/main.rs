use rayon::{iter::IndexedParallelIterator, prelude::*};

// type Mat3x3 = ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64));

#[allow(non_snake_case)]
fn int2d_par(dx: f64, dy: f64, data: &[&[f64]]) -> f64 {
    data.par_windows(3)
        .map(|rows| {
            rows[0]
                .par_windows(3)
                .map(|vals| (vals[0], vals[1], vals[2]))
                .zip(
                    rows[1]
                        .par_windows(3)
                        .map(|vals| (vals[0], vals[1], vals[2])),
                )
                .zip(
                    rows[2]
                        .par_windows(3)
                        .map(|vals| (vals[0], vals[1], vals[2])),
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
                        (Dx_1 + Dx_2) * 0.5,
                        (Dy_1 + Dy_2) * 0.5,
                        (Dx_2 - Dx_1) / dx,
                        (Dy_2 - Dy_1) / dy,
                        DxDy,
                        DyDx,
                    )
                })
                .map(|(fxy, Dx, Dy, DxDx, DyDy, DxDy, DyDx)| {
                    (fxy + Dx * dx * 0.5
                        + Dy * dy * 0.5
                        + DxDx * dx * dx / 6.
                        + DyDy * dy * dy / 6.
                        + DxDy * dx * dy / 16.
                        + DyDx * dx * dy / 16.)
                        * dx
                        * dy
                })
                .sum::<f64>()
        })
        .sum()
}

#[allow(non_snake_case)]
fn int2d_line<'a>(
    dx: f64,
    dy: f64,
    data: &'a [&[f64]],
) -> impl IndexedParallelIterator<Item = f64> + 'a {
    data.par_windows(3).map(move |rows| {
        rows[0]
            .par_windows(3)
            .map(|vals| (vals[0], vals[1], vals[2]))
            .zip(
                rows[1]
                    .par_windows(3)
                    .map(|vals| (vals[0], vals[1], vals[2])),
            )
            .zip(
                rows[2]
                    .par_windows(3)
                    .map(|vals| (vals[0], vals[1], vals[2])),
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
                    (Dx_1 + Dx_2) * 0.5,
                    (Dy_1 + Dy_2) * 0.5,
                    (Dx_2 - Dx_1) / dx,
                    (Dy_2 - Dy_1) / dy,
                    DxDy,
                    DyDx,
                )
            })
            .map(|(fxy, Dx, Dy, DxDx, DyDy, DxDy, DyDx)| {
                (fxy + Dx * dx * 0.5
                    + Dy * dy * 0.5
                    + DxDx * dx * dx / 6.
                    + DyDy * dy * dy / 6.
                    + DxDy * dx * dy / 16.
                    + DyDx * dx * dy / 16.)
                    * dx
                    * dy
            })
            .sum::<f64>()
    })
}

use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader},
    iter::once,
    str::FromStr,
};

fn main() -> Result<(), &'static str> {
    let args: Vec<_> = args().skip(1).collect();
    if args.get(0).map(|s| -> &str { &*s }) == Some("-h") {
        println!("Usage:");
        println!("  -h              => print this help");
        println!("  line [-D <x,y>] <filename> => integrate each line yielding result");
        println!("  seq [-D <x,y>] <filename>  => integrate yielding value on each step");
        println!("  uno [-D <x,y>] <filename>  => integrate in parallel yielding only result");
        return Ok(());
    }
    let mut dx = 1f64;
    let mut dy = 1f64;
    let filename: &str;
    if let Some("-D") = args.get(1).map(|s| -> &str { &*s }) {
        let mut deltas = args.get(2)
            .expect("argument after -D must be <f64,f64>")
            .split(',')
            .map(f64::from_str);
        dx = deltas.next().expect("dx").expect("parse dx into f64");
        dy = deltas.next().expect("dy").expect("parse dy into f64");
        filename = args.get(3).expect("fourth argument must be <filename>");
    } else {
        filename = args.get(1).expect("second argument must be <filename> of '-D'");
    }
    let data = BufReader::with_capacity(1024 * 1024, File::open(filename).unwrap())
        .lines()
        .map(|s| {
            let mut res = s
                .unwrap()
                .split_whitespace()
                .map(|s| s.parse::<f64>().unwrap())
                .collect::<Vec<_>>();
            // skip "z" in the first column with value in the second
            res[0] = res[1];
            res
        })
        .collect::<Vec<_>>();
    let data: Vec<_> = once(data[0].clone()).chain(data.into_iter()).collect();
    let data_ref = &data.iter().map(|v| -> &[f64] { &*v }).collect::<Vec<_>>();
    match args.get(0).map(|s| -> &str { &*s }) {
        Some(arg @ "line") | Some(arg @ "seq") => {
            let solution: Vec<f64> = int2d_line(dx, dy, data_ref).collect();
            match arg {
                "line" => solution.into_iter().for_each(|s| println!("{:e}", s)),
                "seq" => {
                    let mut acc = 0.0;
                    solution.into_iter().for_each(|s| {
                        acc += s;
                        println!("{:e}", acc);
                    })
                }
                _ => unreachable!(),
            }
        }
        Some("uno") => {
            let solution = int2d_par(dx, dy, data_ref);
            println!("{:e}", solution);
        }
        _ => {
            return Err("first argument must be (-h|line|seq|uno)");
        }
    }
    Ok(())
}
