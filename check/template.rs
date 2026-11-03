// Type check of the helpers against stand-ins for the Horizon types they touch (as at the pinned
// revision: mesher.rs, registry.rs, native_hooks.rs, game.rs).
#![allow(dead_code, unused)]
pub mod horizon_client_render {
    #[path = "//CTM_PATH//"]
    pub mod native_connected_textures;
    pub mod registry {
        #[derive(Clone, Copy)] pub struct BlockStateId(pub u32);
        pub struct BlockState { pub name: String, pub properties: std::collections::BTreeMap<String, String> }
        pub struct BlockRegistry;
        impl BlockRegistry { pub fn len(&self) -> usize { 0 } pub fn state(&self, id: BlockStateId) -> Option<&BlockState> { None } }
    }
    pub mod mesher {
        use super::registry::{BlockRegistry, BlockStateId};
        pub struct Chunk;
        pub struct Assets { pub registry: &'static BlockRegistry }
        pub struct RenderAssets { pub sprite_layers: std::collections::BTreeMap<String, u32> }
        #[derive(Clone, Copy)] pub struct TerrainVertex { pub pos: [f32; 3], pub(crate) uv: [u16; 2], pub layer: u16, pub light: u16, pub color: u32 }
        #[derive(Default)] pub struct SectionMesh { pub solid: Vec<TerrainVertex>, pub translucent: Vec<TerrainVertex> }
        #[derive(Default)] struct Padded { blocks: Vec<u32> }
        #[derive(Default)] pub struct MeshScratch { padded: Padded }
        fn pi(x: i32, y: i32, z: i32) -> usize { 0 }
        pub struct Mesher<'a> { pub assets: &'a Assets, pub render: &'a RenderAssets }
        impl Mesher<'_> {
            pub(crate) fn mesh_reusing(&mut self, region: &[[Option<&Chunk>; 3]; 3], sy: i32, mut out: SectionMesh, scratch: &mut MeshScratch) -> SectionMesh {
                if let ::core::ops::ControlFlow::Break(v) = __mesh(&mut *self, region, sy, &mut out, &mut *scratch) { return v; }
                out
            }
        }
        //MESH//
    }
}
pub mod game {
    pub struct Settings;
    pub struct Game<'a> { p: std::marker::PhantomData<&'a ()> }
    impl Game<'_> {
        fn all_sections_changed(&mut self) {}
        pub fn native_config_dir() -> std::io::Result<std::path::PathBuf> { Err(std::io::ErrorKind::Other.into()) }
    }
    pub mod native_hooks {
        use super::Game;
        #[derive(Default)] pub(crate) struct NativeMod { pub(crate) id: String, pub(crate) title: String, pub(crate) description: String, pub(crate) keys: Vec<[String; 3]> }
        impl Game<'_> {
            pub(super) fn native_on(&self, mod_id: &str) -> bool { true }
            pub(super) fn native_frame(&mut self) { __frame(&mut *self) }
        }
        use crate::horizon_client_render;
        //LISTED//
        //SETTINGS//
        //SETTINGS_CLICK//
        //FRAME//
    }
}
pub use self::horizon_client_render as hcr;
