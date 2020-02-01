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

#[allow(non_snake_case)]
fn int2d<D, I>(dx: f64, dy: f64, mut data: D) -> impl Iterator<Item=f64>
    where
        D: Iterator<Item=I>,
        I: Iterator<Item=f64> + Clone,
{
    let init = (data.next().unwrap(), data.next().unwrap(), 0.0);
    let clos =
        move |(ref mut r1, ref mut r2, ref mut acc): &mut (I, I, f64), mut r3: I|
    {
        // let (ref mut r1, ref mut r2, _) = st;
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
                (_f00, f01, _f02),
                (f10, f11, f12),
                (_f20, f21, f22)
            )| {
                let Dx_1 = (f11 - f10) / dx;
                let Dx_2 = (f12 - f11) / dx;
                let Dy_1 = (f11 - f01) / dy;
                let Dy_2 = (f21 - f11) / dy;
                // todo: mean ?
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
        *r1 = r2_later;
        *r2 = r3_later;
        *acc += res;
        // *st = (r2_later, r3_later, st.2 + res);
        Some(*acc)
    };
    data.scan(init, clos)
}

use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
    marker::PhantomData
};

struct FileIter<R, T>
    where
        R: BufRead,
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug
{
    stream: R,
    buffer: String,
    _phantom: PhantomData<T>
}

impl<R, T> FileIter<R, T>
    where
        R: BufRead,
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug
{
    fn new(stream: R) -> Self {
        FileIter {
            stream,
            buffer: String::new(),
            _phantom: PhantomData::<T>
        }
    }
}

impl<R, T> Iterator for FileIter<R, T>
    where
        R: BufRead,
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Vec<T>> {
        if self.stream.read_line(&mut self.buffer).unwrap() == 0 {
            return None;
        }
        Some(
            self.buffer
                .split_whitespace()
                .map(|s| T::from_str(s).unwrap())
                .collect()
        )
    }
}

#[allow(dead_code)]
fn test() {
    let (x1, x2) = (0., 1.);
    let (y1, y2) = (0., 1.);
    let nx = 100;
    let ny = 100;
    let dx = (x2 - x1) / f64::from(nx);
    let dy = (y2 - y1) / f64::from(ny);
    let data: Vec<Vec<f64>> =
        (-1..=ny).map(|yi| {
            let y = y1 + dy * f64::from(yi);
            (-1..=nx).map(|xi| {
                let x = x1 + dx * f64::from(xi);
                x*x*y + y*y*y*x + x
            }).collect()
        }).collect();
    let expected = 19.0 / 24.0;
    let middle: Vec<f64> = int2d(
        dx, dy,
        data.iter().map(|v| v.iter().copied())
    ).collect();
    let actual = middle[middle.len() - 1];
    println!("({}) {:?}", middle.len(), middle);
    println!(
        "expected = {}, actual = {}, error = {}",
        expected, actual, (expected - actual).abs() / expected
    );
}

// посчитать для h := 13 км и 9 км
// максимум -- 1e7
// rho := 1e-3
fn main() {
    let fit: FileIter<_, f64> = FileIter::new(
        BufReader::new(File::open("ya_concat_NFL").unwrap())
    );
    let solutions = int2d(1.0, 1.0, fit.map(|v| v.into_iter().skip(1)));
    solutions.for_each(|f| println!("{}", f));
}
