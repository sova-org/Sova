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

    /// Full scene replacement: drop every existing doc and seed fresh ones from
    /// the new scene's frame contents. Use this for `UpdatedScene` (reset / load),
    /// not for incremental layout changes.
    pub fn reset_from_scene(&self, scene: &Scene) {
        self.docs.write().unwrap().clear();
        let mut layout = self.layout.write().unwrap();
        let mut new_layout: HashMap<(usize, usize), FrameTextId> = HashMap::new();
        for (li, line) in scene.lines.iter().enumerate() {
            for (fi, frame) in line.frames.iter().enumerate() {
                let id = self.alloc_id();
                self.create_doc(id, frame.script().content());
                new_layout.insert((li, fi), id);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use sova_core::scene::{Line, Scene, script::Script};

    fn scene_with(content: &str) -> Scene {
        let mut line = Line::new(vec![1.0]);
        line.set_frame(0, Script::new(content.into(), "boinx".into()).into());
        Scene::new(vec![line])
    }

    fn doc_text(store: &FrameTextStore, id: FrameTextId) -> String {
        let docs = store.docs.read().unwrap();
        let doc = docs.get(&id).expect("doc must exist");
        doc.get_text(FrameTextStore::CONTENT_CONTAINER).to_string()
    }

    #[test]
    fn reset_from_scene_replaces_id_and_text_for_persisting_position() {
        let store = FrameTextStore::new();
        store.rebuild_from_scene(&scene_with("old"));
        let old_id = store.lookup(0, 0).expect("layout has (0, 0)");

        // Simulate live edits piling onto the doc.
        {
            let docs = store.docs.read().unwrap();
            let doc = docs.get(&old_id).unwrap();
            let _ = doc
                .get_text(FrameTextStore::CONTENT_CONTAINER)
                .insert(3, " + edits");
            doc.commit();
        }

        store.reset_from_scene(&scene_with("new"));

        let new_id = store.lookup(0, 0).expect("layout has (0, 0) after reset");
        assert_ne!(old_id, new_id, "reset must allocate a fresh id");
        assert!(
            !store.docs.read().unwrap().contains_key(&old_id),
            "old doc must be dropped"
        );
        assert_eq!(doc_text(&store, new_id), "new");
    }

    #[test]
    fn reset_from_scene_drops_unrelated_docs() {
        let store = FrameTextStore::new();
        let mut line = Line::new(vec![1.0, 1.0]);
        line.set_frame(0, Script::new("a".into(), "boinx".into()).into());
        line.set_frame(1, Script::new("b".into(), "boinx".into()).into());
        store.rebuild_from_scene(&Scene::new(vec![line]));
        assert_eq!(store.docs.read().unwrap().len(), 2);

        store.reset_from_scene(&scene_with("only"));

        let docs = store.docs.read().unwrap();
        assert_eq!(docs.len(), 1, "only the surviving frame keeps a doc");
        let layout = store.layout.read().unwrap();
        assert_eq!(layout.len(), 1);
        assert!(layout.contains_key(&(0, 0)));
    }

    #[test]
    fn rebuild_from_scene_preserves_id_for_persistent_position() {
        let store = FrameTextStore::new();
        store.rebuild_from_scene(&scene_with("first"));
        let id_before = store.lookup(0, 0).unwrap();

        // Add a second frame; (0, 0) should keep its id.
        let mut line = Line::new(vec![1.0, 1.0]);
        line.set_frame(0, Script::new("first".into(), "boinx".into()).into());
        line.set_frame(1, Script::new("second".into(), "boinx".into()).into());
        store.rebuild_from_scene(&Scene::new(vec![line]));

        let id_after = store.lookup(0, 0).unwrap();
        assert_eq!(id_before, id_after, "incremental rebuild preserves identity");
    }
}

