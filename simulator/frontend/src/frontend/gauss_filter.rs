use std::f64::consts::PI;

pub fn create_alpha_filter(size: usize) -> Vec<u8> {
    let radius = size / 2;
    let mut data = vec![0u8; size * size];
    let center = f64::from(size as u32) / 2.0;

    const SHIFT: f64 = 0.0;
    const SCALAR: f64 = 4.0;

    for row in 0..size {
        for col in 0..size {
            let pixel = &mut data[(row * size) + col];
            let sigma_squared = f64::from(radius as u32) / 1.5;
            *pixel = (255.0
                * ((sigma_squared
                    * SCALAR
                    * (calc_gauss(
                        sigma_squared.powf(2.0),
                        dist((col as f64, row as f64), (center, center)),
                    )))
                    + SHIFT)) as u8;
        }
    }

    data
}

fn calc_gauss(sigma_squared: f64, x: f64) -> f64 {
    let scale = 1.0 / ((sigma_squared * 2.0 * PI).powf(0.5));
    let exponent = (-1.0 * x.powi(2)) / (2.0 * sigma_squared);
    scale * exponent.exp()
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).powf(0.5)
}
