use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use loro::LoroDoc;
use sova_core::scene::Scene;

use crate::FrameTextId;

pub struct FrameTextStore {
    pub docs: RwLock<HashMap<FrameTextId, LoroDoc>>,
    pub layout: RwLock<HashMap<(usize, usize), FrameTextId>>,
    pub next_id: AtomicU64,
}

impl FrameTextStore {
    pub const SERVER_PEER_ID: u64 = 0xFFFF_FFFF_FFFF_FF00;
    pub const CONTENT_CONTAINER: &'static str = "content";

    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            docs: RwLock::new(HashMap::new()),
            layout: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        })
    }

    pub fn alloc_id(&self) -> FrameTextId {
        FrameTextId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn create_doc(&self, id: FrameTextId, seed: &str) {
        let doc = LoroDoc::new();
        let _ = doc.set_peer_id(Self::SERVER_PEER_ID);
        if !seed.is_empty() {
            let text = doc.get_text(Self::CONTENT_CONTAINER);
            let _ = text.insert(0, seed);
            doc.commit();
        }
        self.docs.write().unwrap().insert(id, doc);
    }

    pub fn drop_doc(&self, id: FrameTextId) {
        self.docs.write().unwrap().remove(&id);
    }

    pub fn rebuild_from_scene(&self, scene: &Scene) {
        let mut layout = self.layout.write().unwrap();
        let mut new_layout: HashMap<(usize, usize), FrameTextId> = HashMap::new();
        let mut keep_ids: HashSet<FrameTextId> = HashSet::new();
        for (li, line) in scene.lines.iter().enumerate() {
            for (fi, frame) in line.frames.iter().enumerate() {
                let id = match layout.get(&(li, fi)) {
                    Some(existing) => *existing,
                    None => {
                        let fresh = self.alloc_id();
                        self.create_doc(fresh, frame.script().content());
                        fresh
                    }
                };
                new_layout.insert((li, fi), id);
                keep_ids.insert(id);
            }
        }
        let to_drop: Vec<FrameTextId> = self
            .docs
            .read()
            .unwrap()
            .keys()
            .filter(|id| !keep_ids.contains(id))
            .copied()
            .collect();
        for id in to_drop {
            self.drop_doc(id);
        }
        *layout = new_layout;
    }

    pub fn lookup(&self, li: usize, fi: usize) -> Option<FrameTextId> {
        self.layout.read().unwrap().get(&(li, fi)).copied()
    }

    /// Snapshot the current `(li, fi) -> FrameTextId` layout as a wire-friendly Vec.
    pub fn layout_vec(&self) -> Vec<((usize, usize), FrameTextId)> {
        self.layout
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    /// Export every doc as a full Loro snapshot (`ExportMode::snapshot()`).
    pub fn export_full_snapshots(&self) -> Vec<(FrameTextId, Vec<u8>)> {
        self.docs
            .read()
            .unwrap()
            .iter()
            .map(|(id, doc)| {
                (
                    *id,
                    doc.export(loro::ExportMode::snapshot()).unwrap_or_default(),
                )
            })
            .collect()
    }
}

