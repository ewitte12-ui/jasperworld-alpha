use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

/// Spawns a textured quad on the XY plane with custom UV coordinates from an atlas.
///
/// - `uv_rect`: `[u_min, v_min, u_max, v_max]` in atlas UV space.
/// - `position`: world position (x, y, z). Z is visual depth only.
/// - `size`: (width, height) of the quad in world units.
///
/// Returns the spawned `Entity`.
pub fn spawn_textured_quad(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    image: Handle<Image>,
    uv_rect: [f32; 4],
    position: Vec3,
    size: Vec2,
) -> Entity {
    let [u_min, v_min, u_max, v_max] = uv_rect;

    // Create a Rectangle mesh in the XY plane.
    let mut mesh = Mesh::from(Rectangle::new(size.x, size.y));

    // Override UV coordinates to select the correct tile from the atlas.
    // Bevy Rectangle vertex order (confirmed at runtime): TR(0), TL(1), BL(2), BR(3)
    // Positions: [+x,+y], [-x,+y], [-x,-y], [+x,-y]
    // Default UVs: [1,0], [0,0], [0,1], [1,1]
    let uvs = vec![
        [u_max, v_min], // 0 = TR
        [u_min, v_min], // 1 = TL
        [u_min, v_max], // 2 = BL
        [u_max, v_max], // 3 = BR
    ];
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

    let mesh_handle = meshes.add(mesh);

    // WHY AlphaMode::Blend: per jasper_sprite_atlas_guardrail.txt Rule [5],
    // sprite quads must use Blend (not Mask) to avoid alpha-edge popping on
    // thin sprite details.  Mask(0.5) is unstable for soft sprite edges.
    let material = StandardMaterial {
        base_color_texture: Some(image),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };
    let material_handle = materials.add(material);

    commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_translation(position),
        ))
        .id()
}
