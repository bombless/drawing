fn get_radian(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    ((p2.0 - p1.0) / (p2.1 - p1.1)).atan()
}

pub fn draw_line(p1: (f32, f32), p2: (f32, f32), vertices: &mut Vec<f32>, indices: &mut Vec<u16>,
             radius: f32, origin: u16)
{

    let radian = get_radian(p1, p2);
    let offset_x1 = (radian + std::f32::consts::PI / 2.0).sin() * radius;
    let offset_x2 = (radian - std::f32::consts::PI / 2.0).sin() * radius;
    let offset_y1 = (radian + std::f32::consts::PI / 2.0).cos() * radius;
    let offset_y2 = (radian - std::f32::consts::PI / 2.0).cos() * radius;

    vertices.push(offset_x1 + p1.0);
    vertices.push(offset_y1 + p1.1);
    vertices.push(offset_x2 + p1.0);
    vertices.push(offset_y2 + p1.1);
    vertices.push(offset_x1 + p2.0);
    vertices.push(offset_y1 + p2.1);
    vertices.push(offset_x2 + p2.0);
    vertices.push(offset_y2 + p2.1);

    indices.push(origin);
    indices.push(origin + 1);
    indices.push(origin + 2);

    indices.push(origin + 1);
    indices.push(origin + 2);
    indices.push(origin + 3);

}