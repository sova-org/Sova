use std::fs;
use std::path::{Path, PathBuf};

const AUDIO_EXTENSIONS: &[&str] = &["wav", "flac", "ogg", "aiff", "aif", "mp3"];

#[derive(Clone)]
pub enum TreeLineKind {
    Root { expanded: bool },
    Folder { expanded: bool },
    File,
}

#[derive(Clone)]
pub struct TreeLine {
    pub depth: u8,
    pub kind: TreeLineKind,
    pub label: String,
    pub folder: String,
    pub index: usize,
    pub is_default: bool,
}

pub enum SampleNode {
    Root {
        label: String,
        children: Vec<SampleNode>,
        expanded: bool,
    },
    Folder {
        name: String,
        children: Vec<SampleNode>,
        expanded: bool,
    },
    File {
        name: String,
    },
}

impl SampleNode {
    fn expanded(&self) -> bool {
        match self {
            SampleNode::Root { expanded, .. } | SampleNode::Folder { expanded, .. } => *expanded,
            SampleNode::File { .. } => false,
        }
    }

    fn set_expanded(&mut self, val: bool) {
        match self {
            SampleNode::Root { expanded, .. } | SampleNode::Folder { expanded, .. } => {
                *expanded = val;
            }
            SampleNode::File { .. } => {}
        }
    }

    fn is_expandable(&self) -> bool {
        !matches!(self, SampleNode::File { .. })
    }

    fn children(&self) -> &[SampleNode] {
        match self {
            SampleNode::Root { children, .. } | SampleNode::Folder { children, .. } => children,
            SampleNode::File { .. } => &[],
        }
    }

    fn label(&self) -> &str {
        match self {
            SampleNode::Root { label, .. } => label,
            SampleNode::Folder { name, .. } | SampleNode::File { name, .. } => name,
        }
    }

    fn flatten(&self, depth: u8, parent_folder: &str, file_index: usize, is_default: bool, out: &mut Vec<TreeLine>) {
        let kind = match self {
            SampleNode::Root { expanded, .. } => TreeLineKind::Root {
                expanded: *expanded,
            },
            SampleNode::Folder { expanded, .. } => TreeLineKind::Folder {
                expanded: *expanded,
            },
            SampleNode::File { .. } => TreeLineKind::File,
        };
        out.push(TreeLine {
            depth,
            kind,
            label: self.label().to_string(),
            folder: parent_folder.to_string(),
            index: file_index,
            is_default,
        });
        if self.expanded() {
            let folder_name = self.label();
            let mut idx = 0;
            for child in self.children() {
                let child_idx = if matches!(child, SampleNode::File { .. }) {
                    let i = idx;
                    idx += 1;
                    i
                } else {
                    0
                };
                child.flatten(depth + 1, folder_name, child_idx, is_default, out);
            }
        }
    }
}

pub struct SampleTree {
    roots: Vec<SampleNode>,
    default_root_count: usize,
}

impl SampleTree {
    pub fn from_paths_with_default(default_path: Option<&Path>, user_paths: &[PathBuf]) -> Self {
        let mut roots = Vec::new();

        // Default samples first
        if let Some(dp) = default_path
            && let Some(root) = Self::scan_root(dp)
        {
            roots.push(root);
        }
        let default_root_count = roots.len();

        // User paths
        let all_user = default_root_count == 0 && user_paths.len() == 1;
        if all_user {
            let mut user_roots = Self::scan_children(&user_paths[0]);
            roots.append(&mut user_roots);
        } else {
            for path in user_paths {
                if let Some(root) = Self::scan_root(path) {
                    roots.push(root);
                }
            }
        }

        Self { roots, default_root_count }
    }

    fn scan_children(path: &Path) -> Vec<SampleNode> {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut files: Vec<String> = Vec::new();
        let mut folders: Vec<(String, PathBuf)> = Vec::new();

        for entry in entries.flatten() {
            let ft = entry.file_type().ok();
            let name = entry.file_name().to_string_lossy().into_owned();
            if ft.is_some_and(|t| t.is_dir()) {
                folders.push((name, entry.path()));
            } else if is_audio_file(&name) {
                files.push(name);
            }
        }

        folders.sort_by_key(|a| a.0.to_lowercase());
        files.sort_by_key(|a| a.to_lowercase());

        let mut nodes = Vec::new();
        for (name, folder_path) in folders {
            if let Some(folder) = Self::scan_folder(&name, &folder_path) {
                nodes.push(folder);
            }
        }
        for name in files {
            nodes.push(SampleNode::File { name });
        }
        nodes
    }

    fn scan_root(path: &Path) -> Option<SampleNode> {
        let entries = fs::read_dir(path).ok()?;
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        let mut files: Vec<String> = Vec::new();
        let mut folders: Vec<(String, PathBuf)> = Vec::new();

        for entry in entries.flatten() {
            let ft = entry.file_type().ok();
            let name = entry.file_name().to_string_lossy().into_owned();
            if ft.is_some_and(|t| t.is_dir()) {
                folders.push((name, entry.path()));
            } else if is_audio_file(&name) {
                files.push(name);
            }
        }

        folders.sort_by_key(|a| a.0.to_lowercase());
        files.sort_by_key(|a| a.to_lowercase());

        let mut children = Vec::new();
        for (name, folder_path) in folders {
            if let Some(folder) = Self::scan_folder(&name, &folder_path) {
                children.push(folder);
            }
        }
        for name in files {
            children.push(SampleNode::File { name });
        }

        Some(SampleNode::Root {
            label,
            children,
            expanded: false,
        })
    }

    fn scan_folder(name: &str, path: &Path) -> Option<SampleNode> {
        let entries = fs::read_dir(path).ok()?;
        let mut files: Vec<String> = Vec::new();

        for entry in entries.flatten() {
            let ft = entry.file_type().ok();
            let entry_name = entry.file_name().to_string_lossy().into_owned();
            if ft.is_some_and(|t| t.is_file()) && is_audio_file(&entry_name) {
                files.push(entry_name);
            }
        }
        files.sort_by_key(|a| a.to_lowercase());

        if files.is_empty() {
            return None;
        }

        let children = files
            .into_iter()
            .map(|name| SampleNode::File { name })
            .collect();

        Some(SampleNode::Folder {
            name: name.to_string(),
            children,
            expanded: false,
        })
    }

    pub fn visible_entries(&self) -> Vec<TreeLine> {
        let mut out = Vec::new();
        for (i, root) in self.roots.iter().enumerate() {
            root.flatten(0, "", 0, i < self.default_root_count, &mut out);
        }
        out
    }

    fn node_at_mut(&mut self, visible_index: usize) -> Option<&mut SampleNode> {
        let mut count = 0;
        for root in &mut self.roots {
            if let Some(node) = Self::walk_mut(root, visible_index, &mut count) {
                return Some(node);
            }
        }
        None
    }

    fn walk_mut<'a>(
        node: &'a mut SampleNode,
        target: usize,
        count: &mut usize,
    ) -> Option<&'a mut SampleNode> {
        if *count == target {
            return Some(node);
        }
        *count += 1;
        if node.expanded() {
            let children = match node {
                SampleNode::Root { children, .. } | SampleNode::Folder { children, .. } => children,
                SampleNode::File { .. } => return None,
            };
            for child in children.iter_mut() {
                if let Some(found) = Self::walk_mut(child, target, count) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn find_folder_mut(&mut self, name: &str) -> Option<&mut SampleNode> {
        for root in &mut self.roots {
            if let Some(node) = Self::find_folder_in(root, name) {
                return Some(node);
            }
        }
        None
    }

    fn find_folder_in<'a>(node: &'a mut SampleNode, name: &str) -> Option<&'a mut SampleNode> {
        match node {
            SampleNode::Folder { name: n, .. } if n == name => Some(node),
            SampleNode::Root { children, .. } | SampleNode::Folder { children, .. } => {
                for child in children.iter_mut() {
                    if let Some(found) = Self::find_folder_in(child, name) {
                        return Some(found);
                    }
                }
                None
            }
            SampleNode::File { .. } => None,
        }
    }

    fn filtered_entries(&self, names: &[String], collapsed: bool) -> Vec<TreeLine> {
        let mut out = Vec::new();
        for name in names {
            for (i, root) in self.roots.iter().enumerate() {
                let is_default = i < self.default_root_count;
                Self::emit_filtered(root, name, collapsed, is_default, &mut out);
            }
        }
        out
    }

    fn emit_filtered(
        node: &SampleNode,
        target_name: &str,
        collapsed: bool,
        is_default: bool,
        out: &mut Vec<TreeLine>,
    ) {
        match node {
            SampleNode::Folder {
                name,
                children,
                expanded,
            } if name == target_name => {
                let show_children = !collapsed && *expanded;
                out.push(TreeLine {
                    depth: 0,
                    kind: TreeLineKind::Folder {
                        expanded: show_children,
                    },
                    label: name.clone(),
                    folder: String::new(),
                    index: 0,
                    is_default,
                });
                if show_children {
                    let mut idx = 0;
                    for child in children {
                        if let SampleNode::File { name: fname } = child {
                            out.push(TreeLine {
                                depth: 1,
                                kind: TreeLineKind::File,
                                label: fname.clone(),
                                folder: name.clone(),
                                index: idx,
                                is_default,
                            });
                            idx += 1;
                        }
                    }
                }
            }
            SampleNode::Root { children, .. } => {
                for child in children {
                    Self::emit_filtered(child, target_name, collapsed, is_default, out);
                }
            }
            _ => {}
        }
    }
}

pub struct SampleBrowserState {
    pub tree: SampleTree,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    filter: Option<Vec<String>>,
    cached_entries: Vec<TreeLine>,
}

impl SampleBrowserState {
    pub fn new(default_path: Option<&Path>, user_paths: &[PathBuf]) -> Self {
        let tree = SampleTree::from_paths_with_default(default_path, user_paths);
        let cached_entries = tree.visible_entries();
        Self {
            tree,
            cursor: 0,
            scroll_offset: 0,
            search_query: String::new(),
            filter: None,
            cached_entries,
        }
    }

    fn rebuild_cache(&mut self) {
        self.cached_entries = match &self.filter {
            Some(names) => self.tree.filtered_entries(names, false),
            None => self.tree.visible_entries(),
        };
    }

    pub fn entries(&self) -> &[TreeLine] {
        &self.cached_entries
    }

    pub fn current_entry(&self) -> Option<&TreeLine> {
        self.cached_entries.get(self.cursor)
    }

    fn visible_count(&self) -> usize {
        self.cached_entries.len()
    }

    fn clamp_view(&mut self) {
        let count = self.cached_entries.len();
        if count == 0 {
            self.cursor = 0;
            self.scroll_offset = 0;
            return;
        }
        if self.cursor >= count {
            self.cursor = count - 1;
        }
        if self.scroll_offset > self.cursor {
            self.scroll_offset = self.cursor;
        }
    }

    pub fn move_up(&mut self, n: usize) {
        self.cursor = self.cursor.saturating_sub(n);
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
    }

    pub fn move_down(&mut self, n: usize, visible_height: usize) {
        let count = self.visible_count();
        if count == 0 {
            return;
        }
        self.cursor = (self.cursor + n).min(count - 1);
        if self.cursor >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor - visible_height + 1;
        }
    }

    pub fn toggle_expand(&mut self) {
        if self.filter.is_some() {
            let is_folder = self
                .cached_entries
                .get(self.cursor)
                .is_some_and(|e| matches!(e.kind, TreeLineKind::Folder { .. }));
            if is_folder {
                let label = self.cached_entries[self.cursor].label.clone();
                if let Some(node) = self.tree.find_folder_mut(&label) {
                    let new_val = !node.expanded();
                    node.set_expanded(new_val);
                }
            }
        } else if let Some(node) = self.tree.node_at_mut(self.cursor)
            && node.is_expandable()
        {
            let new_val = !node.expanded();
            node.set_expanded(new_val);
        }
        self.rebuild_cache();
        self.clamp_view();
    }

    pub fn collapse_at_cursor(&mut self) {
        let is_file = match self.cached_entries.get(self.cursor) {
            Some(e) => matches!(e.kind, TreeLineKind::File),
            None => return,
        };
        if is_file {
            let parent = (0..self.cursor).rev().find(|&i| {
                matches!(
                    self.cached_entries[i].kind,
                    TreeLineKind::Folder { .. } | TreeLineKind::Root { .. }
                )
            });
            if let Some(i) = parent {
                let label = self.cached_entries[i].label.clone();
                if self.filter.is_some() {
                    if let Some(node) = self.tree.find_folder_mut(&label) {
                        node.set_expanded(false);
                    }
                } else if let Some(node) = self.tree.node_at_mut(i) {
                    node.set_expanded(false);
                }
                self.cursor = i;
                if self.cursor < self.scroll_offset {
                    self.scroll_offset = self.cursor;
                }
                self.rebuild_cache();
                return;
            }
        } else {
            let label = self.cached_entries[self.cursor].label.clone();
            let is_expanded = match &self.cached_entries[self.cursor].kind {
                TreeLineKind::Folder { expanded } | TreeLineKind::Root { expanded } => *expanded,
                _ => false,
            };
            if is_expanded {
                if self.filter.is_some() {
                    if let Some(node) = self.tree.find_folder_mut(&label) {
                        node.set_expanded(false);
                    }
                } else if let Some(node) = self.tree.node_at_mut(self.cursor) {
                    node.set_expanded(false);
                }
            }
        }
        self.rebuild_cache();
        self.clamp_view();
    }

    pub fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filter = None;
        } else {
            let query = &self.search_query;
            let mut scored: Vec<(i32, String)> = Vec::new();
            for root in &self.tree.roots {
                Self::collect_matching_folder_names(root, query, &mut scored);
            }
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            let names = scored.into_iter().map(|(_, name)| name).collect();
            self.filter = Some(names);
        }
        self.cursor = 0;
        self.scroll_offset = 0;
        self.rebuild_cache();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.filter = None;
        self.cursor = 0;
        self.scroll_offset = 0;
        self.rebuild_cache();
    }

    pub fn clear_filter(&mut self) {
        self.filter = None;
        self.search_query.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.rebuild_cache();
    }

    fn collect_matching_folder_names(node: &SampleNode, query: &str, out: &mut Vec<(i32, String)>) {
        match node {
            SampleNode::Root { children, .. } => {
                for child in children {
                    Self::collect_matching_folder_names(child, query, out);
                }
            }
            SampleNode::Folder { name, .. } => {
                if let Some((score, _)) = crate::widgets::fuzzy_score(query, name) {
                    out.push((score, name.clone()));
                }
            }
            SampleNode::File { .. } => {}
        }
    }
}

pub fn resolve_sample_path(sample_paths: &[PathBuf], entry: &TreeLine) -> Option<PathBuf> {
    if !matches!(entry.kind, TreeLineKind::File) {
        return None;
    }
    for root in sample_paths {
        if let Some(path) = find_file_in(root, &entry.folder, &entry.label) {
            return Some(path);
        }
    }
    None
}

fn find_file_in(root: &Path, folder: &str, filename: &str) -> Option<PathBuf> {
    if folder.is_empty() {
        let p = root.join(filename);
        if p.is_file() {
            return Some(p);
        }
    }
    // Search for folder/filename recursively
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let ft = entry.file_type().ok()?;
        if ft.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == folder {
                let p = entry.path().join(filename);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn is_audio_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    AUDIO_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}
