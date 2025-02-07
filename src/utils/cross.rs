use nalgebra::{Point2, Vector3};
use std::f64;

/// <https://habr.com/ru/articles/267037/#comment_8597117>
pub fn cross(
    p11: Point2<f64>,
    p12: Point2<f64>,
    p21: Point2<f64>,
    p22: Point2<f64>,
) -> Option<Point2<f64>> {
    let p11 = Vector3::from(p11);
    let p12 = Vector3::from(p12);
    let p21 = Vector3::from(p21);
    let p22 = Vector3::from(p22);
    // Ообщее уравнение прямой, проходящей через две точки ((x1, y1) и (x2,
    // y2)) - векторное произведение наших точек в однородных координатах:
    // `cross((x1, y1, 1), (x2, y2, 1))`
    let cross1 = p11.cross(&p12);
    let cross2 = p21.cross(&p22);
    // Уравнение прямой в общем виде (ax + by + c = 0) можно записать как
    // вектор (a, b, c). Точка пересечения двух таких прямых - векторное
    // произведение: `cross((a1, b1, c1), (a2, b2, c2))` (мы получим
    // однородные координаты в которых лежит наша точка пересечения).
    let cross = cross1.cross(&cross2);
    println!("cross1: {cross1}");
    println!("cross2: {cross2}");
    println!("cross: {cross}");
    ((cross.z - 0.0).abs() > f64::EPSILON).then_some((cross / cross.z).xy().into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let point = cross(
            Point2::new(1.0, 2.0),
            Point2::new(1.0, 2.0),
            Point2::new(1.0, 2.0),
            Point2::new(1.0, 2.0),
        );
        println!("cross: {point:?}");
        // let point = cross(
        //     Point2::new(0.0, 0.0),
        //     Point2::new(2.0, 4.0),
        //     Point2::new(3.0, 6.0),
        //     Point2::new(4.0, 8.0),
        // );
        // println!("cross: {point:?}");
        // let point = cross(
        //     Point2::new(3.0, -3.0),
        //     Point2::new(0.0, 0.0),
        //     Point2::new(0.0, 0.0),
        //     Point2::new(4.0, 4.0),
        // );
        // println!("cross: {point:?}");

        // let point = cross(
        //     Point2::new(1.0, 1.0),
        //     Point2::new(3.0, 3.0),
        //     Point2::new(1.0, 3.0),
        //     Point2::new(2.0, 1.0),
        // );
        // let point = cross(
        //     Point2::new(1.0, 1.0),
        //     Point2::new(2.0, 1.0),
        //     Point2::new(1.0, 3.0),
        //     Point2::new(2.0, 3.0),
        // );
        // println!("cross: {point:?}");
    }
}
