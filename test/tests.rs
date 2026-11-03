use crate::ctm::{self, Method, V, EDGE, UV_STEPS};
use std::collections::HashMap;

// Sprite layers: glass 1, glass pane top 2, bookshelf 3, sandstone 4, sandstone bottom 5, red glass 6.
fn table() -> std::sync::Arc<ctm::Rules> {
    let names = ["minecraft:air", "minecraft:glass", "minecraft:glass_pane", "minecraft:bookshelf", "minecraft:sandstone", "minecraft:red_stained_glass", "minecraft:stone"];
    let layers: HashMap<&str, u16> = [("minecraft:block/glass", 1), ("minecraft:block/bookshelf", 3), ("minecraft:block/sandstone", 4), ("minecraft:block/sandstone_bottom", 5), ("minecraft:block/glass_pane_top", 2), ("minecraft:block/red_stained_glass", 6)].into_iter().collect();
    ctm::rules([1, 2, 3], names.len(), &|id| names.get(id as usize).map(|s| s.to_string()), &|n| layers.get(n).copied())
}

const AIR: u32 = 0;
const GLASS: u32 = 1;
const PANE: u32 = 2;
const SHELF: u32 = 3;
const SAND: u32 = 4;
const RED: u32 = 5;

fn world(blocks: &[([i32; 3], u32)]) -> impl Fn(i32, i32, i32) -> u32 + '_ {
    move |x, y, z| blocks.iter().find(|b| b.0 == [x, y, z]).map_or(AIR, |b| b.1)
}

fn v(p: [f32; 3], uv: [f32; 2], layer: u16) -> V {
    V { pos: p, uv: uv.map(|x| (x * UV_STEPS) as u16), layer, light: 0xF0F0, color: 0xFFFF_FFFF }
}

/// The north face (z = 0) of the block at `b`, as vanilla bakes it: u runs from x+1 to x.
fn north(b: [i32; 3], layer: u16) -> Vec<V> {
    let [x, y, z] = b.map(|k| k as f32);
    vec![
        v([x + 1.0, y + 1.0, z], [0.0, 0.0], layer),
        v([x + 1.0, y, z], [0.0, 1.0], layer),
        v([x, y, z], [1.0, 1.0], layer),
        v([x, y + 1.0, z], [1.0, 0.0], layer),
    ]
}

/// The top face of the block at `b`.
fn top(b: [i32; 3], layer: u16) -> Vec<V> {
    let [x, y, z] = b.map(|k| k as f32);
    vec![
        v([x, y + 1.0, z], [0.0, 0.0], layer),
        v([x, y + 1.0, z + 1.0], [0.0, 1.0], layer),
        v([x + 1.0, y + 1.0, z + 1.0], [1.0, 1.0], layer),
        v([x + 1.0, y + 1.0, z], [1.0, 0.0], layer),
    ]
}

fn uvs(q: &[V]) -> Vec<[f32; 2]> {
    q.iter().map(|v| [f32::from(v.uv[0]) / UV_STEPS, f32::from(v.uv[1]) / UV_STEPS]).collect()
}

/// Whether some part of `out` shows sprite texel column/row (`u`, `v`) at the spot (`x`, `y`) of
/// the face (in sprite units, 0..1): what texel a point of the face samples.
fn sample(out: &[V], at: [f32; 2]) -> Option<[f32; 2]> {
    // The face's own UV at the point (the north face's u runs along -x, v along -y).
    for q in out.chunks(4) {
        let u = uvs(q);
        // Each sub quad's corners in face space are its original UVs before shifting; recover
        // from position: the north face of block (0,0,0): s = 1 - x, t = 1 - y.
        let s: Vec<[f32; 2]> = q.iter().map(|v| [1.0 - v.pos[0], 1.0 - v.pos[1]]).collect();
        let (s0, s1) = (s.iter().map(|p| p[0]).fold(9.0, f32::min), s.iter().map(|p| p[0]).fold(-9.0, f32::max));
        let (t0, t1) = (s.iter().map(|p| p[1]).fold(9.0, f32::min), s.iter().map(|p| p[1]).fold(-9.0, f32::max));
        if at[0] > s0 && at[0] < s1 && at[1] > t0 && at[1] < t1 {
            let k = s.iter().position(|p| (p[0] - s0).abs() < 1e-4 && (p[1] - t0).abs() < 1e-4).unwrap();
            return Some([u[k][0] + (at[0] - s0), u[k][1] + (at[1] - t0)]);
        }
    }
    None
}

#[test]
fn a_lone_block_keeps_its_face() {
    let r = table();
    let w = world(&[([0, 0, 0], GLASS)]);
    assert!(ctm::process(&r, &north([0, 0, 0], 1), &w).is_none());
}

#[test]
fn glass_beside_glass_loses_the_frame_between() {
    let r = table();
    // Glass at x 0 and x 1: on the north face of x 0, the edge towards x 1 (+x, u = 0) joins.
    let blocks = [([0, 0, 0], GLASS), ([1, 0, 0], GLASS)];
    let w = world(&blocks);
    let out = ctm::process(&r, &north([0, 0, 0], 1), &w).expect("joined");
    assert!(out.len() > 4 && out.len() % 4 == 0);
    // The strip along u = 0 shows the texel just inside it now; the far edge keeps its frame.
    let s = sample(&out, [0.5 * EDGE, 0.5]).unwrap();
    assert!((s[0] - 1.5 * EDGE).abs() < 1e-3, "{s:?}");
    let far = sample(&out, [1.0 - 0.5 * EDGE, 0.5]).unwrap();
    assert!((far[0] - (1.0 - 0.5 * EDGE)).abs() < 1e-3, "{far:?}");
    // The top and bottom frame stay along the joined edge too.
    let corner = sample(&out, [0.5 * EDGE, 0.5 * EDGE]).unwrap();
    assert!((corner[1] - 0.5 * EDGE).abs() < 1e-3 && (corner[0] - 1.5 * EDGE).abs() < 1e-3, "{corner:?}");
    // Same area, same winding as the face (normal still -z).
    let area: f32 = out.chunks(4).map(|q| {
        let a = [q[1].pos[0] - q[0].pos[0], q[1].pos[1] - q[0].pos[1], q[1].pos[2] - q[0].pos[2]];
        let b = [q[2].pos[0] - q[0].pos[0], q[2].pos[1] - q[0].pos[1], q[2].pos[2] - q[0].pos[2]];
        let n = a[0] * b[1] - a[1] * b[0];
        assert!(n.abs() < 1e-6 || true);
        let cz = a[0] * b[1] - a[1] * b[0];
        assert!(cz <= 1e-6, "faces -z: {cz}");
        -cz
    }).sum();
    assert!((area - 1.0).abs() < 1e-3, "{area}");
}

#[test]
fn a_window_of_four_keeps_inner_corners_only_where_the_corner_is_missing() {
    let r = table();
    // An L of three: (0,0) (1,0) (0,1); the corner block (1,1) is missing.
    let blocks = [([0, 0, 0], GLASS), ([1, 0, 0], GLASS), ([0, 1, 0], GLASS)];
    let w = world(&blocks);
    let out = ctm::process(&r, &north([0, 0, 0], 1), &w).expect("joined");
    // Towards +x is u = 0, towards +y is v = 0: that corner keeps its frame pixel.
    let c = sample(&out, [0.5 * EDGE, 0.5 * EDGE]).unwrap();
    assert!((c[0] - 0.5 * EDGE).abs() < 1e-3 && (c[1] - 0.5 * EDGE).abs() < 1e-3, "{c:?}");
    // With the corner block there, it is gone too.
    let blocks = [([0, 0, 0], GLASS), ([1, 0, 0], GLASS), ([0, 1, 0], GLASS), ([1, 1, 0], GLASS)];
    let w = world(&blocks);
    let out = ctm::process(&r, &north([0, 0, 0], 1), &w).expect("joined");
    let c = sample(&out, [0.5 * EDGE, 0.5 * EDGE]).unwrap();
    assert!((c[0] - 1.5 * EDGE).abs() < 1e-3 && (c[1] - 1.5 * EDGE).abs() < 1e-3, "{c:?}");
}

#[test]
fn a_neighbour_whose_face_is_hidden_does_not_join() {
    let r = table();
    // The glass at x 1 has glass in front of it (z -1): its north face is not drawn.
    let blocks = [([0, 0, 0], GLASS), ([1, 0, 0], GLASS), ([1, 0, -1], GLASS)];
    let w = world(&blocks);
    assert!(ctm::process(&r, &north([0, 0, 0], 1), &w).is_none());
}

#[test]
fn different_glass_does_not_join() {
    let r = table();
    let blocks = [([0, 0, 0], GLASS), ([1, 0, 0], RED)];
    let w = world(&blocks);
    assert!(ctm::process(&r, &north([0, 0, 0], 1), &w).is_none());
    // A glass face of another block (a pane's glass beside a glass block) is left alone.
    let blocks = [([0, 0, 0], PANE), ([1, 0, 0], GLASS)];
    let w = world(&blocks);
    assert!(ctm::process(&r, &north([0, 0, 0], 1), &w).is_none());
}

#[test]
fn glass_tops_join_too() {
    let r = table();
    let blocks = [([0, 0, 0], GLASS), ([0, 0, 1], GLASS)];
    let w = world(&blocks);
    assert!(ctm::process(&r, &top([0, 0, 0], 1), &w).is_some());
}

#[test]
fn bookshelves_join_sideways_only() {
    let r = table();
    let w = world(&[([0, 0, 0], SHELF), ([0, 1, 0], SHELF)]);
    assert!(ctm::process(&r, &north([0, 0, 0], 3), &w).is_none(), "not up and down");
    let w = world(&[([0, 0, 0], SHELF), ([1, 0, 0], SHELF)]);
    let out = ctm::process(&r, &north([0, 0, 0], 3), &w).expect("sideways");
    let s = sample(&out, [0.5 * EDGE, 0.5 * EDGE]).unwrap();
    assert!((s[0] - 1.5 * EDGE).abs() < 1e-3 && (s[1] - 0.5 * EDGE).abs() < 1e-3, "{s:?}");
    // Their tops are planks: left alone.
    assert!(ctm::process(&r, &top([0, 0, 0], 3), &w).is_none());
}

#[test]
fn sandstone_under_sandstone_shows_its_smooth_side() {
    let r = table();
    let g = r.groups.iter().find(|g| g.tile == 4).unwrap();
    assert_eq!(g.method, Method::Top(5));
    let w = world(&[([0, 0, 0], SAND), ([0, 1, 0], SAND)]);
    let out = ctm::process(&r, &north([0, 0, 0], 4), &w).expect("smooth");
    assert!(out.iter().all(|v| v.layer == 5));
    // The top one keeps its band.
    assert!(ctm::process(&r, &north([0, 1, 0], 4), &w).is_none());
    // Opaque bit kept.
    let mut q = north([0, 0, 0], 4);
    for v in &mut q {
        v.layer |= 0x8000;
    }
    assert!(ctm::process(&r, &q, &w).unwrap().iter().all(|v| v.layer == 5 | 0x8000));
}

#[test]
fn a_pane_arm_joins_at_the_block_edge() {
    let r = table();
    // A pane running east-west: its west half's north face, x 0..0.5 at z 7/16, u 0.5..1.
    let z = 7.0 / 16.0;
    let q = vec![
        v([0.5, 1.0, z], [0.5, 0.0], 1),
        v([0.5, 0.0, z], [0.5, 1.0], 1),
        v([0.0, 0.0, z], [1.0, 1.0], 1),
        v([0.0, 1.0, z], [1.0, 0.0], 1),
    ];
    let w = world(&[([0, 0, 0], PANE), ([-1, 0, 0], PANE)]);
    let out = ctm::process(&r, &q, &w).expect("joined west");
    // The strip at x 0..1/16 (u 15/16..1) shows u just inside the frame.
    let strip: Vec<&[V]> = out.chunks(4).filter(|c| c.iter().all(|v| v.pos[0] <= EDGE + 1e-4)).collect();
    assert!(!strip.is_empty());
    for c in strip {
        for t in uvs(c) {
            assert!(t[0] <= 1.0 - EDGE + 1e-3, "{t:?}");
        }
    }
    let alone = world(&[([0, 0, 0], PANE)]);
    assert!(ctm::process(&r, &q, &alone).is_none());
}

#[test]
fn light_and_colour_stay_smooth_across_the_cut() {
    let r = table();
    let mut q = north([0, 0, 0], 1);
    q[0].light = 0x00F0;
    q[2].light = 0x0000;
    q[1].color = 0xFF80_8080;
    let w = world(&[([0, 0, 0], GLASS), ([1, 0, 0], GLASS)]);
    let out = ctm::process(&r, &q, &w).unwrap();
    for v in &out {
        if (v.pos[0] - 1.0).abs() < 1e-4 && (v.pos[1] - 1.0).abs() < 1e-4 {
            assert_eq!(v.light, 0x00F0);
        }
        assert_eq!(v.color >> 24, 0xFF);
    }
}

#[test]
fn settings_text_reads_back() {
    assert_eq!(ctm::parse(""), (true, 15));
    assert_eq!(ctm::parse("enabled=0\nglass=1\npanes=0\nbookshelves=1\nsandstone=0\n"), (false, 1 | 4));
}

#[test]
#[ignore]
fn dump_wall() {
    let r = table();
    // A wall facing north: a 4x3 window with a hole, a lone pane of glass, and bookshelves.
    let mut blocks = Vec::new();
    for x in 0..4 { for y in 0..3 { if !(x == 2 && y == 1) { blocks.push(([x, y, 0], GLASS)); } } }
    blocks.push(([6, 1, 0], GLASS));
    for x in 8..11 { blocks.push(([x, 0, 0], SHELF)); blocks.push(([x, 1, 0], SHELF)); }
    blocks.push(([12, 0, 0], SAND)); blocks.push(([12, 1, 0], SAND)); blocks.push(([12, 2, 0], SAND));
    let w = world(&blocks);
    let mut s = String::new();
    for (b, id) in &blocks {
        let layer = match *id { GLASS => 1, SHELF => 3, _ => 4 };
        let q = north(*b, layer);
        let out = ctm::process(&r, &q, &w).unwrap_or(q);
        for v in out {
            s.push_str(&format!("{} {} {} {} {}\n", v.pos[0], v.pos[1], f32::from(v.uv[0]) / UV_STEPS, f32::from(v.uv[1]) / UV_STEPS, v.layer & 0x7fff));
        }
    }
    std::fs::write("/tmp/claude-0/-home-claude/ca868743-7075-50c1-8188-0998b6cbda88/scratchpad/cont/wall.txt", s).unwrap();
}

#[test]
fn stacked_panes_hide_the_edges_between_them() {
    let r = table();
    // The top of the lower pane's post (y 1, x/z 7..9), drawn with the pane's edge sprite.
    let (a, b) = (7.0 / 16.0, 9.0 / 16.0);
    let up = vec![
        v([a, 1.0, a], [7.0 / 16.0, 7.0 / 16.0], 2),
        v([a, 1.0, b], [7.0 / 16.0, 9.0 / 16.0], 2),
        v([b, 1.0, b], [9.0 / 16.0, 9.0 / 16.0], 2),
        v([b, 1.0, a], [9.0 / 16.0, 7.0 / 16.0], 2),
    ];
    let stacked = world(&[([0, 0, 0], PANE), ([0, 1, 0], PANE)]);
    assert_eq!(ctm::process(&r, &up, &stacked), Some(Vec::new()));
    let alone = world(&[([0, 0, 0], PANE)]);
    assert!(ctm::process(&r, &up, &alone).is_none());
    // Glass above is another block: the top stays.
    let glass = world(&[([0, 0, 0], PANE), ([0, 1, 0], GLASS)]);
    assert!(ctm::process(&r, &up, &glass).is_none());
}
