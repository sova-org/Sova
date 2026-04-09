use sova_core::scene::{Line, Scene};

// Language demos (auto-generated from assets/demos/cagire/ and assets/demos/boinx/)
include!(concat!(env!("OUT_DIR"), "/demos_generated.rs"));

pub fn random_demo() -> crate::Snapshot {
    let demos = DEMOS_GENERAL;
    if demos.is_empty() {
        return crate::Snapshot {
            scene: Scene::new(vec![Line::new(vec![1.0])]),
            tempo: 120.0,
            beat: 0.0,
            micros: 0,
            quantum: 4.0,
            devices: vec![],
        };
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    // Splitmix64 mixing step for uniform distribution
    let mut z = nanos.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^= z >> 31;
    let idx = z as usize % demos.len();
    let (name, bytes) = demos[idx];
    match serde_json::from_slice::<crate::Snapshot>(bytes) {
        Ok(snap) => {
            println!("Loaded demo: {name}");
            snap
        }
        Err(e) => {
            eprintln!("Failed to load demo '{name}': {e}");
            crate::Snapshot {
                scene: Scene::new(vec![Line::new(vec![1.0])]),
                tempo: 120.0,
                beat: 0.0,
                micros: 0,
                quantum: 4.0,
                devices: vec![],
            }
        }
    }
}

// General demos with curated display names
pub const DEMOS_GENERAL: &[(&str, &[u8])] = &[
    (
        "Aliens near us",
        include_bytes!("../assets/demos/general/aliens_near_us.sova"),
    ),
    (
        "2005 algorave",
        include_bytes!("../assets/demos/general/2005_algorave.sova"),
    ),
    (
        "By the pond",
        include_bytes!("../assets/demos/general/by_the_pond.sova"),
    ),
    (
        "Classic move",
        include_bytes!("../assets/demos/general/classic_move.sova"),
    ),
    (
        "Lush elegiac stuff",
        include_bytes!("../assets/demos/general/lush_elegiac_stuff.sova"),
    ),
    (
        "Intense boots and cats",
        include_bytes!("../assets/demos/general/intense_boots_and_cats.sova"),
    ),
    (
        "Infinite gongs",
        include_bytes!("../assets/demos/general/infinite_gongs.sova"),
    ),
    (
        "Chill 808",
        include_bytes!("../assets/demos/general/chill_808.sova"),
    ),
    (
        "Mayo sandwich",
        include_bytes!("../assets/demos/general/mayo_sandwich.sova"),
    ),
    (
        "First day with my modular",
        include_bytes!("../assets/demos/general/first_day_with_my_modular.sova"),
    ),
    (
        "Bit after bit",
        include_bytes!("../assets/demos/general/bit_after_bit.sova"),
    ),
    (
        "Some soup ?",
        include_bytes!("../assets/demos/general/some_soup.sova"),
    ),
    (
        "Storm of sand",
        include_bytes!("../assets/demos/general/darude.sova"),
    ),
    (
        "Dirty & Crunchy",
        include_bytes!("../assets/demos/general/crado.sova"),
    ),
    (
        "Studious",
        include_bytes!("../assets/demos/general/chords.sova"),
    ),
    (
        "People with basses",
        include_bytes!("../assets/demos/general/people_with_basses.sova"),
    ),
];
