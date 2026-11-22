use crate::offsets::{
    VIEW_FRUSTUM_NEAR_PLANE_NORMAL_OFFSET, VIEW_FRUSTUM_NEAR_PLANE_OFFSET_OFFSET,
    VIEW_FRUSTUM_PLANES_BLOCK_LENGTH, VIEW_FRUSTUM_PLANES_BLOCK_OFFSET,
    VIEW_FRUSTUM_SIDE_PLANE_COUNT, VIEW_FRUSTUM_SIDE_PLANE_STRIDE, VIEW_FRUSTUM_SIDE_PLANES_OFFSET,
};

const PLANE_OFFSET_THAT_NO_SPHERE_FALLS_BEHIND: f32 = 1.0e9;
const FLOAT_BYTES: usize = 4;
const NORMAL_FLOATS: usize = 3;

pub fn open_view_frustum_planes(block: &mut [u8; VIEW_FRUSTUM_PLANES_BLOCK_LENGTH]) {
    let start = VIEW_FRUSTUM_PLANES_BLOCK_OFFSET as usize;
    let mut set = |offset: u64, value: f32| {
        let at = offset as usize - start;
        block[at..at + FLOAT_BYTES].copy_from_slice(&value.to_le_bytes());
    };
    for axis in 0..NORMAL_FLOATS as u64 {
        set(
            VIEW_FRUSTUM_NEAR_PLANE_NORMAL_OFFSET + axis * FLOAT_BYTES as u64,
            0.0,
        );
    }
    set(
        VIEW_FRUSTUM_NEAR_PLANE_OFFSET_OFFSET,
        PLANE_OFFSET_THAT_NO_SPHERE_FALLS_BEHIND,
    );
    for plane in 0..VIEW_FRUSTUM_SIDE_PLANE_COUNT {
        let plane_start = VIEW_FRUSTUM_SIDE_PLANES_OFFSET + plane * VIEW_FRUSTUM_SIDE_PLANE_STRIDE;
        for axis in 0..NORMAL_FLOATS as u64 {
            set(plane_start + axis * FLOAT_BYTES as u64, 0.0);
        }
        set(
            plane_start + NORMAL_FLOATS as u64 * FLOAT_BYTES as u64,
            PLANE_OFFSET_THAT_NO_SPHERE_FALLS_BEHIND,
        );
    }
}
