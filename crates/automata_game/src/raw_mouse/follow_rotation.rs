use crate::raw_mouse::CameraAngles;

pub const CAMERA_BASIS_MATRIX_OFFSET: u64 = 0x280;
pub const CAMERA_TARGET_BODY_POSITION_OFFSET: u64 = 0x50;
pub const BASIS_ORTHONORMAL_TOLERANCE: f32 = 1e-3;
pub const IDENTITY_BASIS: Basis = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

pub type Basis = [[f32; 3]; 3];

pub fn rotation_basis(rows: Basis) -> Option<Basis> {
    for (a, row_a) in rows.iter().enumerate() {
        for (b, row_b) in rows.iter().enumerate() {
            let wanted = if a == b { 1.0 } else { 0.0 };
            if (dot(*row_a, *row_b) - wanted).abs() > BASIS_ORTHONORMAL_TOLERANCE {
                return None;
            }
        }
    }
    Some(rows)
}

pub fn rotate_eased_offset(
    offset: [f32; 3],
    before: CameraAngles,
    after: CameraAngles,
    basis: &Basis,
) -> [f32; 3] {
    let in_game_frame = row_times_transpose(offset, basis);
    let turned = about_y(
        about_x(
            about_x(about_y(in_game_frame, -before.yaw), -before.pitch),
            after.pitch,
        ),
        after.yaw,
    );
    row_times(turned, basis)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn row_times(row: [f32; 3], matrix: &Basis) -> [f32; 3] {
    [
        row[0] * matrix[0][0] + row[1] * matrix[1][0] + row[2] * matrix[2][0],
        row[0] * matrix[0][1] + row[1] * matrix[1][1] + row[2] * matrix[2][1],
        row[0] * matrix[0][2] + row[1] * matrix[1][2] + row[2] * matrix[2][2],
    ]
}

fn row_times_transpose(row: [f32; 3], matrix: &Basis) -> [f32; 3] {
    [
        dot(row, matrix[0]),
        dot(row, matrix[1]),
        dot(row, matrix[2]),
    ]
}

fn about_x(row: [f32; 3], angle: f32) -> [f32; 3] {
    let (sin, cos) = angle.sin_cos();
    [
        row[0],
        row[1] * cos - row[2] * sin,
        row[1] * sin + row[2] * cos,
    ]
}

fn about_y(row: [f32; 3], angle: f32) -> [f32; 3] {
    let (sin, cos) = angle.sin_cos();
    [
        row[0] * cos + row[2] * sin,
        row[1],
        -row[0] * sin + row[2] * cos,
    ]
}
