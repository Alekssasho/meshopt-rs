use crate::ffi;
use crate::{DecodePosition, VertexDataAdapter};

pub type Bounds = ffi::meshopt_Bounds;
pub type Meshlet = ffi::meshopt_Meshlet;

/// Splits the mesh into a set of meshlets where each meshlet has a micro index buffer
/// indexing into meshlet vertices that refer to the original vertex buffer.
///
/// The resulting data can be used to render meshes using NVidia programmable mesh shading
/// pipeline, or in other cluster-based renderers.
///
/// For maximum efficiency the index buffer being converted has to be optimized for vertex
/// cache first.
///
/// Note: `max_vertices` must be <= 64 and `max_triangles` must be <= 126
pub fn build_meshlets(
    indices: &[u32],
    vertices: &VertexDataAdapter,
    max_vertices: usize,
    max_triangles: usize,
    cone_weight: f32,
) -> (Vec<Meshlet>, Vec<u32>, Vec<u8>) {
    let meshlet_count =
        unsafe { ffi::meshopt_buildMeshletsBound(indices.len(), max_vertices, max_triangles) };
    let mut meshlets: Vec<Meshlet> = vec![unsafe { ::std::mem::zeroed() }; meshlet_count];
    let mut meshlet_vertices: Vec<u32> = vec![unsafe { ::std::mem::zeroed() }; meshlet_count * max_vertices];
    let mut meshlet_indices: Vec<u8> = vec![unsafe { ::std::mem::zeroed() }; meshlet_count * max_triangles * 3];
    let count = unsafe {
        ffi::meshopt_buildMeshlets(
            meshlets.as_mut_ptr(),
            meshlet_vertices.as_mut_ptr(),
            meshlet_indices.as_mut_ptr(),
            indices.as_ptr(),
            indices.len(),
            vertices.pos_ptr(),
            vertices.vertex_count,
            vertices.vertex_stride,
            max_vertices,
            max_triangles,
            cone_weight
        )
    };
    meshlets.resize(count, unsafe { ::std::mem::zeroed() });
    let last_meshlet = meshlets.last().unwrap();
    meshlet_vertices.resize((last_meshlet.vertex_offset + last_meshlet.vertex_count) as usize, unsafe { ::std::mem::zeroed() });
    meshlet_indices.resize((last_meshlet.triangle_offset + (last_meshlet.triangle_count * 3)) as usize, unsafe { ::std::mem::zeroed() });
    (meshlets, meshlet_vertices, meshlet_indices)
}

/// Creates bounding volumes that can be used for frustum, backface and occlusion culling.
///
/// For backface culling with orthographic projection, use the following formula to reject backfacing clusters:
///   `dot(view, cone_axis) >= cone_cutoff`
///
/// For perspective projection, use the following formula that needs cone apex in addition to axis & cutoff:
///   `dot(normalize(cone_apex - camera_position), cone_axis) >= cone_cutoff`
///
/// Alternatively, you can use the formula that doesn't need cone apex and uses bounding sphere instead:
///   `dot(normalize(center - camera_position), cone_axis) >= cone_cutoff + radius / length(center - camera_position)`
///
/// or an equivalent formula that doesn't have a singularity at center = camera_position:
///   `dot(center - camera_position, cone_axis) >= cone_cutoff * length(center - camera_position) + radius`
///
/// The formula that uses the apex is slightly more accurate but needs the apex; if you are already using bounding sphere
/// to do frustum/occlusion culling, the formula that doesn't use the apex may be preferable.
///
/// `index_count` should be <= 256*3 (the function assumes clusters of limited size)
pub fn compute_cluster_bounds(indices: &[u32], vertices: &VertexDataAdapter) -> Bounds {
    unsafe {
        ffi::meshopt_computeClusterBounds(
            indices.as_ptr(),
            indices.len(),
            vertices.pos_ptr(),
            vertices.vertex_count,
            vertices.vertex_stride,
        )
    }
}

/// Creates bounding volumes that can be used for frustum, backface and occlusion culling.
///
/// For backface culling with orthographic projection, use the following formula to reject backfacing clusters:
///   `dot(view, cone_axis) >= cone_cutoff`
///
/// For perspective projection, use the following formula that needs cone apex in addition to axis & cutoff:
///   `dot(normalize(cone_apex - camera_position), cone_axis) >= cone_cutoff`
///
/// Alternatively, you can use the formula that doesn't need cone apex and uses bounding sphere instead:
///   `dot(normalize(center - camera_position), cone_axis) >= cone_cutoff + radius / length(center - camera_position)`
///
/// or an equivalent formula that doesn't have a singularity at center = camera_position:
///   `dot(center - camera_position, cone_axis) >= cone_cutoff * length(center - camera_position) + radius`
///
/// The formula that uses the apex is slightly more accurate but needs the apex; if you are already using bounding sphere
/// to do frustum/occlusion culling, the formula that doesn't use the apex may be preferable.
///
/// `index_count` should be <= 256*3 (the function assumes clusters of limited size)
pub fn compute_cluster_bounds_decoder<T: DecodePosition>(
    indices: &[u32],
    vertices: &[T],
) -> Bounds {
    let vertices = vertices
        .iter()
        .map(|vertex| vertex.decode_position())
        .collect::<Vec<[f32; 3]>>();
    let positions = vertices.as_ptr() as *const f32;
    unsafe {
        ffi::meshopt_computeClusterBounds(
            indices.as_ptr(),
            indices.len(),
            positions,
            vertices.len() * 3,
            ::std::mem::size_of::<f32>() * 3,
        )
    }
}

pub fn compute_meshlet_bounds(meshlet: &Meshlet, meshlet_vertices: &[u32], meshlet_triangles: &[u8], vertices: &VertexDataAdapter) -> Bounds {
    let vertex_data = vertices.reader.get_ref();
    let vertex_data = vertex_data.as_ptr() as *const u8;
    let positions = unsafe { vertex_data.add(vertices.position_offset) };
    unsafe {
        ffi::meshopt_computeMeshletBounds(
            meshlet_vertices.as_ptr().add(meshlet.vertex_offset as usize),
            meshlet_triangles.as_ptr().add(meshlet.triangle_offset as usize),
            meshlet.triangle_count as usize,
            positions as *const f32,
            vertices.vertex_count,
            vertices.vertex_stride,
        )
    }
}

// pub fn compute_meshlet_bounds_decoder<T: DecodePosition>(
//     meshlet: &Meshlet,
//     vertices: &[T],
// ) -> Bounds {
//     let vertices = vertices
//         .iter()
//         .map(|vertex| vertex.decode_position())
//         .collect::<Vec<[f32; 3]>>();
//     let positions = vertices.as_ptr() as *const f32;
//     unsafe {
//         ffi::meshopt_computeMeshletBounds(
//             meshlet,
//             positions,
//             vertices.len() * 3,
//             ::std::mem::size_of::<f32>() * 3,
//         )
//     }
// }
