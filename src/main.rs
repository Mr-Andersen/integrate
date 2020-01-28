use rayon::prelude::*;

// type Mat3x3 = ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64));

#[allow(dead_code)]
fn int2d_par(dx: f64, dy: f64, data: &[&[f64]]) -> f64 {
    #![allow(non_snake_case)]
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

#[allow(unused_variables)]
#[allow(non_snake_case)]
fn int2d<D, I>(dx: f64, dy: f64, data: D) -> f64
    where
        D: Iterator<Item=I>,
        I: Iterator<Item=f64> + Clone,
{
    use std::iter::{Map, Enumerate};
    // type I2 = Map<Enumerate<I>, impl FnMut((usize, f64)) -> u16>;
    // let first_row = data.next().unwrap();
    let mut data = data.map(|it: I| it.enumerate().map(|(i, _)| f64::from(i as u16)));
    let init = (data.next().unwrap(), data.next().unwrap(), 0.0);
    let clos = |(mut r1, mut r2, acc): (Map<Enumerate<I>, _>, Map<Enumerate<I>, _>, f64), mut r3: Map<Enumerate<I>, _>| {
        let r2_later = r2.clone();
        let r3_later = r3.clone();
        let init1 = (r1.next().unwrap(), r1.next().unwrap());
        let init2 = (r2.next().unwrap(), r2.next().unwrap());
        let init3 = (r3.next().unwrap(), r3.next().unwrap());
        let func = |st: &mut (f64, f64), r3: f64| {
            let res = (st.0, st.1, r3);
            *st = (st.1, r3);
            Some(res)
        };
        let res = r1.scan(init1, func)
            .zip(r2.scan(init2, func))
            .zip(r3.scan(init3, func))
            .map(|((cols1, cols2), cols3)| (cols1, cols2, cols3))
            .map(|(
                (f00, f01, f02),
                (f10, f11, f12),
                (f20, f21, f22)
            )| {
                println!("{:.2} {:.2} {:.2}", f00, f01, f02);
                println!("{:.2} {:.2} {:.2}", f10, f11, f12);
                println!("{:.2} {:.2} {:.2}\n", f20, f21, f22);
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
            .sum::<f64>();
        (r2_later, r3_later, acc + res)
    };
    data.fold(init, clos).2
}

fn main() {
    let (x1, x2) = (0., 1.);
    let (y1, y2) = (0., 1.);
    let nx = 10;
    let ny = 10;
    let dx = (x2 - x1) / f64::from(nx);
    let dy = (y2 - y1) / f64::from(ny);
    let data: Vec<Vec<f64>> =
        (-1..=ny).map(|yi| {
            let y = y1 + dy * f64::from(yi);
            (-1..=nx).map(|xi| {
                let x = x1 + dx * f64::from(xi);
                x*y
            }).collect()
        }).collect();
    println!("data = [");
    data.iter().for_each(|v| {
        print!("\t[");
        v.into_iter().for_each(|e| print!("\t{:.2}", e));
        println!("]");
        // println!("  {:?}", v)
    });
    println!("]");
    let expected = 0.25;
    let actual = int2d(
        dx, dy,
        data.iter().map(|v| v.iter().copied())
    );
    println!(
        "expected = {}, actual = {}, error = {}",
        expected, actual, (expected - actual).abs() / expected
    );
}
