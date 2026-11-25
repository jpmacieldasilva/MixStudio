use super::*;

impl App {
    pub(super) fn seed_library_demo_catalog(&mut self, demo_dir: &std::path::Path) {
        const TARGET: usize = 412;
        if self.lib.index.entries.len() >= TARGET {
            self.library_ui.demo_catalog_done = true;
            return;
        }
        let folder = demo_dir.display().to_string();
        let mut templates: Vec<crate::library::Entry> = self.lib.index.entries.clone();
        if templates.is_empty() {
            let folder = demo_dir.display().to_string();
            for path in self.store.keys() {
                let name = std::path::Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("Track");
                templates.push(crate::library::Entry {
                    path: path.clone(),
                    folder: folder.clone(),
                    title: name.into(),
                    ..Default::default()
                });
            }
        }
        if templates.is_empty() {
            return;
        }
        let titles = [
            "Vessel", "Bass Test E minor", "Concrete Bloom", "Low Tide Signal", "Pad Test C major", "Salt Glass",
            "Hollow Arcade", "Mirror Stage", "Kick Test A minor", "Neon Drive", "Midnight Pulse", "Gravity Well",
            "Velvet Rush", "Iron Bloom", "Echo Chamber", "High Voltage", "Amber Lines", "Static Dreams", "Phase Runner",
            "Deep Signal", "Midnight Ferry", "Pacific State", "Urban Motion", "Silver Line", "Amber Coast", "North Station",
            "Late Arrival", "Echo Rooms", "Coastal Drive", "Blue District", "Disco Arcade", "Neon Avenue",
        ];
        let artists = [
            "Vessel", "Odile Varga", "Skyline", "Metro Unit", "Depth Charge", "Low End Collective", "Night Driver",
            "Afterglow", "Maris Vega", "North Works", "Analog Youth", "Kite Theory", "Kick Lab", "Pad Unit",
            "Low End", "Raver", "Sunrise Avenue", "Pacific State", "Urban Motion",
        ];
        let keys = ["8A", "8B", "9A", "10A", "11B", "12A", "6A", "7B", "5A", "4B"];
        let bpms = [124.0, 126.0, 128.0, 130.0, 122.0, 118.0, 132.0, 127.5, 125.0, 129.0];
        let mut n = self.lib.index.entries.len();
        let mut i = 0usize;
        while n < TARGET {
            let mut e = templates[i % templates.len()].clone();
            e.path = format!("{}/catalog/track_{n:04}.wav", folder);
            e.title = titles[(n * 5 + 3) % titles.len()].into();
            e.artist = artists[(n * 7 + 1) % artists.len()].into();
            e.bpm = Some(bpms[i % bpms.len()]);
            e.key = Some(keys[i % keys.len()].into());
            e.duration = 360.0 + (i % 180) as f64;
            e.error = None;
            self.lib.index.entries.push(e);
            n += 1;
            i += 1;
        }
        self.lib.reindex();
        self.library_ui.demo_catalog_done = true;
        self.library_ui.tree_key = None;
        self.lib_view_key = None;
    }
    pub(super) fn seed_library_demo_metadata(&mut self) {
        const MAP: &[(&str, &str, &str)] = &[
            ("kick", "Vessel", "Odile Varga"),
            ("pad", "Skyline", "Metro Unit"),
            ("bass", "Depth Charge", "Low End Collective"),
            ("rave", "Afterglow", "Night Driver"),
        ];
        for e in self.lib.index.entries.iter_mut() {
            let low = if e.title.is_empty() {
                e.filename().to_lowercase()
            } else {
                e.title.to_lowercase()
            };
            for (needle, title, artist) in MAP {
                if low.contains(needle) {
                    e.title = (*title).into();
                    e.artist = (*artist).into();
                    break;
                }
            }
        }
    }
    fn lib_import_settings(&mut self, ui: &mut egui::Ui) {
        let dim = |s: &str| egui::RichText::new(s.to_string()).color(DIM).size(11.0);
        ui.separator();
        ui.label(egui::RichText::new("Other DJ software").strong().size(12.0));
        let mut sel: Option<Option<(usize, Vec<usize>)>> = None; let mut rm: Option<usize> = None;
        let cur = self.lib_node.clone();
        for (li, f) in self.lib.cfg.imports.clone().iter().enumerate() {
            match self.lib.ext.get(li) {
                Some(Ok(l)) => {
                    let missing = l.tracks.iter().filter(|t| !t.exists).count();
                    egui::CollapsingHeader::new(format!("{} ({} tracks)", l.source.name(), l.tracks.len())).id_salt(("imp", li)).default_open(true).show(ui, |ui| {
                        ui.add(egui::Label::new(dim(f)).truncate()).on_hover_text(f);
                        if missing > 0 { ui.label(egui::RichText::new(format!("⚠ {missing} files not found: check path remapping")).color(C::from_rgb(230, 170, 60)).size(11.0)); }
                        let c = cur.as_ref().map(|n| (n.0, n.1.as_slice()));
                        if ui.selectable_label(c == Some((li, &[][..])), format!("♫ All tracks ({})", l.tracks.len())).clicked() { sel = Some(Some((li, vec![]))); }
                        let mut path = vec![]; for (ci, ch) in l.root.children.iter().enumerate() { path.push(ci); node_ui(ui, ch, li, &mut path, c, &mut sel); path.pop(); }
                        if ui.small_button("✖ Remove import").on_hover_text("Forget this import (the other app's files are never touched)").clicked() { rm = Some(li); }
                    });
                }
                Some(Err(e)) => { ui.label(egui::RichText::new(format!("⚠ {f}: {e}")).color(C::from_rgb(230, 100, 80)).size(11.0)); if ui.small_button("✖ Remove").clicked() { rm = Some(li); } }
                None => { ui.horizontal(|ui| { ui.spinner(); ui.label(dim(&format!("Reading {f}…"))); }); }
            }
        }
        if let Some(s) = sel { self.library_ui.folder=None; self.lib_node = s; self.lib_sel.clear(); self.lib_anchor = None; }
        if let Some(i) = rm { self.lib.remove_import(i); self.lib_node = None; self.lib_sel.clear(); }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button("🔍 Find libraries").on_hover_text("Look in the usual places: home folder, Documents, Music and mounted drives (incl. Windows / macOS user folders)").clicked() { self.find_libs(); }
            if ui.button("Browse…").clicked() { self.pick_import(); } });
        for p in self.lib_found.clone() { let s = p.display().to_string(); let have = self.lib.cfg.imports.contains(&s);
            ui.horizontal(|ui| { let k = crate::import::sniff(&p).map(|k| k.name()).unwrap_or("?");
                if ui.add_enabled(!have, egui::Button::new(if have { "✔" } else { "Import" }).small()).clicked() { self.import_file(p.clone()); }
                ui.add(egui::Label::new(dim(&format!("{k}: {s}"))).truncate()).on_hover_text(&s); }); }
        ui.horizontal(|ui| { ui.add(egui::TextEdit::singleline(&mut self.lib_imp).hint_text("…or paste a library file path").desired_width(150.0));
            if ui.add_enabled(!self.lib_imp.trim().is_empty(), egui::Button::new("Import")).clicked() { let p = PathBuf::from(self.lib_imp.trim()); self.lib_imp.clear(); self.import_file(p); } });
        ui.label(dim("rekordbox.xml (File › Export Collection in xml format), Traktor collection.nml, _Serato_/database V2, Engine Library/Database2/m.db, iTunes / Music Library.xml. Read-only."));
        ui.separator();
        egui::CollapsingHeader::new("Path remapping").id_salt("remaps").show(ui, |ui| {
            if !self.lib_remaps_init { self.lib_remaps = self.lib.cfg.remaps.clone(); self.lib_remaps_init = true; }
            ui.label(dim("Rewrite path prefixes of imported tracks, e.g. C:\\Users\\Mike\\Music ➡ /mnt/windows/Users/Mike/Music"));
            let mut del = None;
            for (i, r) in self.lib_remaps.iter_mut().enumerate() { ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut r.from).hint_text("from").desired_width(95.0)); ui.label("➡");
                ui.add(egui::TextEdit::singleline(&mut r.to).hint_text("to").desired_width(95.0)); if ui.small_button("✖").clicked() { del = Some(i); } }); }
            if let Some(i) = del { self.lib_remaps.remove(i); }
            ui.horizontal(|ui| { if ui.button("+ Add rule").clicked() { self.lib_remaps.push(Default::default()); }
                let changed = self.lib_remaps != self.lib.cfg.remaps;
                if ui.add_enabled(changed, egui::Button::new("Apply")).clicked() { let v: Vec<_> = self.lib_remaps.iter().filter(|r| !r.from.trim().is_empty()).cloned().collect(); self.lib_remaps = v.clone(); self.lib.set_remaps(v); self.lib_cur_key = None; } });
        });
    }
}

fn node_ui(ui: &mut egui::Ui, n: &crate::import::Node, li: usize, path: &mut Vec<usize>, cur: Option<(usize, &[usize])>, sel: &mut Option<Option<(usize, Vec<usize>)>>) {
    let me = cur == Some((li, path.as_slice()));
    if n.children.is_empty() { if ui.selectable_label(me, format!("♫ {} ({})", n.name, n.tracks.len())).clicked() { *sel = Some(Some((li, path.clone()))); } return; }
    egui::CollapsingHeader::new(format!("🗀 {}", n.name)).id_salt(("node", li, path.clone())).show(ui, |ui| {
        if ui.selectable_label(me, format!("♫ All in {} ({})", n.name, n.all_tracks().len())).clicked() { *sel = Some(Some((li, path.clone()))); }
        for (ci, c) in n.children.iter().enumerate() { path.push(ci); node_ui(ui, c, li, path, cur, sel); path.pop(); }
    });
}

#[derive(Default)]
pub(super) struct State {
    pub folder:Option<String>,pub details:bool,pub geometry:Option<library_drop::Geometry>,
    tree:Vec<crate::library::browse::FolderNode>,pub(crate) tree_key:Option<(u64,Vec<String>)>,
    /// Studio sidebar: 0 = library picker, 1 = mix track list
    pub side_tab:u8,
    pub import_open:bool,
    pub demo_meta_done:bool,
    pub demo_catalog_done:bool,
    pub capture_autoorder_menu:bool,
}

fn library_capture_demo() -> bool {
    std::env::var("MIXSTUDIO_DEMO_TAB").as_deref() == Ok("library")
}
/// Full-width navigation row: name truncated to fit, count right-aligned.
fn nav_row(ui:&mut egui::Ui,name:&str,count:usize,selected:bool)->egui::Response {
    let (rect,response)=ui.allocate_exact_size(vec2(ui.available_width(),24.0),Sense::click());
    let painter=ui.painter();
    if selected {painter.rect_filled(rect,Rounding::same(5.0),FOCUS_SOFT);} else if response.hovered() {painter.rect_filled(rect,Rounding::same(5.0),CTRL_HOVER);}
    let counter=painter.layout_no_wrap(count.to_string(),FontId::proportional(11.0),DIM);
    let right=rect.right()-8.0-counter.size().x; painter.galley(pos2(right,rect.center().y-counter.size().y*0.5),counter,DIM);
    let mut job=egui::text::LayoutJob::simple_singleline(name.to_string(),FontId::proportional(13.0),TXT);
    job.wrap=egui::text::TextWrapping::truncate_at_width((right-rect.left()-16.0).max(20.0));
    let text=ui.fonts(|f|f.layout_job(job)); painter.galley(pos2(rect.left()+6.0,rect.center().y-text.size().y*0.5),text,TXT);
    response
}
fn folder_node(ui:&mut egui::Ui,node:&crate::library::browse::FolderNode,current:Option<&str>,selected:&mut Option<String>,remove:&mut Option<String>,root:bool) {
    let row=|ui:&mut egui::Ui,selected:&mut Option<String>,remove:&mut Option<String>| {
        let r=nav_row(ui,&node.name,node.count,current==Some(node.path.as_str()));
        if r.clicked(){*selected=Some(node.path.clone());}
        let r=r.on_hover_text(&node.path);
        if root {r.context_menu(|ui|{if ui.button("Remove from library").clicked(){*remove=Some(node.path.clone());ui.close_menu();}});}
    };
    if node.children.is_empty(){ui.horizontal(|ui|{ui.add_space(18.0);row(ui,selected,remove);});}
    else {egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(),ui.id().with(("music_folder",&node.path)),root)
        .show_header(ui,|ui|row(ui,selected,remove)).body(|ui|{for child in &node.children {folder_node(ui,child,current,selected,remove,false);}});}
}
impl App {
    pub(super) fn library_demo_maintain(&mut self) {
        if !library_capture_demo() {
            return;
        }
        self.lib.abort_scan();
        self.lib_q.clear();
        self.lib_key = false;
        self.lib_bpm = false;
        if self.library_ui.folder.is_some() {
            self.choose_library_folder(None);
        }
        if self.lib.index.entries.len() < 412 {
            self.library_ui.demo_catalog_done = false;
            if let Some(root) = self.lib.cfg.folders.first().cloned() {
                self.seed_library_demo_catalog(std::path::Path::new(&root));
                self.lib_view_key = None;
                self.refresh_library_view();
            }
        }
        if self.lib.index.entries.len() >= 412 {
            self.lib_view_key = None;
            self.refresh_library_view();
        }
    }
    pub(super) fn library_entries(&self)->&[crate::library::Entry] {if self.lib_node.is_some(){&self.lib_cur}else{&self.lib.index.entries}}
    pub(super) fn library_selected_paths(&self)->Vec<String> {
        self.lib_view.iter().filter_map(|&i|self.library_entries().get(i)).filter(|e|self.lib_sel.contains(&e.path)).map(|e|e.path.clone()).collect()
    }
    fn choose_library_folder(&mut self,folder:Option<String>) {
        self.library_ui.folder=folder;self.lib_node=None;self.lib_sel.clear();self.lib_anchor=None;self.lib_view_key=None;
    }
    fn dj_import_track_counts(&self) -> (usize, usize) {
        let mut rekordbox = 0usize;
        let mut serato = 0usize;
        for (li, path) in self.lib.cfg.imports.iter().enumerate() {
            let n = self
                .lib
                .ext
                .get(li)
                .and_then(|r| r.as_ref().ok())
                .map(|l| l.tracks.len())
                .unwrap_or(0);
            let name = crate::import::sniff(std::path::Path::new(path))
                .map(|k| k.name().to_lowercase())
                .unwrap_or_default();
            if name.contains("rekordbox") {
                rekordbox += n;
            } else if name.contains("serato") {
                serato += n;
            }
        }
        (rekordbox, serato)
    }
    pub(super) fn library_import_window(&mut self, ctx: &egui::Context) {
        if !self.library_ui.import_open {
            return;
        }
        let mut close = false;
        egui::Window::new("Import & sources")
            .collapsible(false)
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| {
                self.lib_import_settings(ui);
                ui.add_space(6.0);
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        if close {
            self.library_ui.import_open = false;
        }
    }
    pub(super) fn library_sidebar(&mut self,ui:&mut egui::Ui) {
        ui.spacing_mut().item_spacing.y=4.0;
        ui.horizontal(|ui|{
            ui.label(egui::RichText::new("SOURCES").size(10.5).color(DIM).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if ui.link(egui::RichText::new("Add Folder…").size(11.0).color(FOCUS)).on_hover_text("Add a music folder").clicked(){self.pick_folder();}
            });
        });
        let all_selected=self.lib_node.is_none()&&self.library_ui.folder.is_none();
        if nav_row(ui,"All Music",self.lib.index.entries.len(),all_selected).clicked(){self.choose_library_folder(None);}
        ui.add_space(4.0);
        ui.horizontal(|ui|{
            ui.label(egui::RichText::new("Folders").size(11.0).strong().color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if ui.link(egui::RichText::new("Add Folder…").size(11.0).color(FOCUS)).on_hover_text("Add a music folder").clicked(){self.pick_folder();}
            });
        });
        if library_capture_demo() {
            let _ = nav_row(ui, "Music", 366, false);
            ui.indent("demo-mx", |ui| {
                let _ = nav_row(ui, "mx-demo", 4, false);
                let _ = nav_row(ui, "Promos 2026", 46, false);
            });
        } else {
            let key = (self.lib.path_gen, self.lib.cfg.folders.clone());
            if self.library_ui.tree_key.as_ref() != Some(&key) {
                self.library_ui.tree = crate::library::browse::folder_tree(&self.lib.cfg.folders, &self.lib.index.entries);
                self.library_ui.tree_key = Some(key);
            }
            let mut selected = None;
            let mut remove = None;
            for node in &self.library_ui.tree {
                folder_node(ui, node, self.library_ui.folder.as_deref(), &mut selected, &mut remove, true);
            }
            if let Some(folder) = selected {
                self.choose_library_folder(Some(folder));
            }
            if let Some(folder) = remove {
                self.lib.remove_folder(&folder);
                self.choose_library_folder(None);
            }
        }
        ui.add_space(8.0);
        let (rb, ser) = self.dj_import_track_counts();
        ui.horizontal(|ui|{
            ui.label(egui::RichText::new("DJ Software").size(11.0).strong().color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if ui.link(egui::RichText::new("Import…").size(11.0).color(FOCUS)).on_hover_text("Find libraries, browse, paste paths").clicked(){self.library_ui.import_open=true;}
            });
        });
        if library_capture_demo() {
            let _ = nav_row(ui, "rekordbox", 284, false);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").color(C::from_rgb(230, 170, 60)).size(11.0));
                ui.label(egui::RichText::new("Path remapping may be required").size(10.5).color(DIM));
            });
            let _ = nav_row(ui, "Serato", 96, false);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").color(C::from_rgb(230, 170, 60)).size(11.0));
                ui.label(egui::RichText::new("Some crates are offline").size(10.5).color(DIM));
            });
        } else {
            let _ = nav_row(ui, "rekordbox", rb, self.lib_node.is_some());
            let _ = nav_row(ui, "Serato", ser, self.lib_node.is_some());
        }
        ui.add_space(6.0);
        ui.horizontal(|ui|{
            ui.label(egui::RichText::new("Sources").size(11.0).strong().color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if ui.link(egui::RichText::new("Import…").size(11.0).color(FOCUS)).on_hover_text("Find libraries, browse, paste paths").clicked(){self.library_ui.import_open=true;}
            });
        });
        let _ = nav_row(ui, "Path Remapping", self.lib.cfg.remaps.len(), false);
    }
    pub(super) fn refresh_library_view(&mut self) {
        if let Some(node)=self.lib_node.clone(){if self.lib_cur_key.as_ref()!=Some(&(self.lib.gen,node.clone())){
            self.lib_cur=match self.lib.ext.get(node.0).and_then(|r|r.as_ref().ok()) {Some(l)=>{
                let ids=if node.1.is_empty(){(0..l.tracks.len()).collect()}else{l.root.at(&node.1).map(|n|n.all_tracks()).unwrap_or_default()};let mut seen=std::collections::HashSet::new();
                ids.into_iter().filter(|&i|seen.insert(i)).filter_map(|i|l.tracks.get(i)).map(|t|{let ours=self.lib.entry(&t.path);crate::library::Entry{path:t.path.clone(),folder:crate::library::EXT_FOLDER.into(),title:t.title.clone(),artist:t.artist.clone(),album:t.album.clone(),genre:t.genre.clone(),duration:if t.duration>0.0{t.duration}else{ours.map(|e|e.duration).unwrap_or(0.0)},bpm:ours.and_then(|e|e.bpm),key:ours.and_then(|e|e.key.clone()),error:if t.exists{None}else{Some("Source file unavailable; check path remapping".into())},..Default::default()}}).collect()
            },None=>{self.lib_node=None;vec![]}};self.lib_cur_key=Some((self.lib.gen,node));self.lib_view_key=None;
        }}
        let reference=self.lib_ref();let key_ref=if self.lib_key{reference.as_ref().map(|r|r.1.clone())}else{None};let bpm_ref=if self.lib_bpm{reference.as_ref().map(|r|(r.2,self.lib_pct))}else{None};
        let gen={use std::hash::{Hash,Hasher};let mut h=std::collections::hash_map::DefaultHasher::new();self.lib_node.hash(&mut h);self.library_ui.folder.hash(&mut h);self.lib.gen^h.finish()};
        let key=(gen,self.lib_q.clone(),self.lib_sort,self.lib_asc,key_ref.clone(),bpm_ref.map(|(b,p)|(b.to_bits(),p.to_bits())));
        if self.lib_view_key.as_ref()!=Some(&key){
            self.lib_view=crate::library::browse::folder_view(self.library_entries(),&crate::library::Filter{query:&self.lib_q,key_ref:key_ref.as_deref(),bpm_ref},self.lib_sort,self.lib_asc,if self.lib_node.is_none(){self.library_ui.folder.as_deref()}else{None});
            let visible:std::collections::HashSet<_>=self.lib_view.iter().map(|&i|self.library_entries()[i].path.clone()).collect();self.lib_sel.retain(|p|visible.contains(p));if self.lib_anchor.as_ref().is_some_and(|p|!visible.contains(p)){self.lib_anchor=None;}self.lib_view_key=Some(key);
        }
    }
    pub(super) fn library_tab(&mut self,ui:&mut egui::Ui) {
        if library_capture_demo() {
            if !self.library_ui.demo_catalog_done {
                if let Some(root) = self.lib.cfg.folders.first().cloned() {
                    self.seed_library_demo_catalog(std::path::Path::new(&root));
                    self.refresh_library_view();
                }
            }
            if !self.library_ui.demo_meta_done && !self.lib.index.entries.is_empty() {
                self.seed_library_demo_metadata();
                self.library_ui.demo_meta_done = true;
                self.refresh_library_view();
            }
        }
        if self.top == 2 {
            self.library_files(ui);
            return;
        }
        // The folder list is its own card; a gutter and a solid scrollbar keep it from running into the files.
        egui::SidePanel::left("library_navigation_v3").resizable(true).default_width(260.0).width_range(170.0..=400.0)
            .frame(egui::Frame::none().fill(PANEL).rounding(Rounding::same(8.0)).inner_margin(egui::Margin::same(8.0)).stroke(Stroke::new(1.0_f32, LINE_SOFT)).outer_margin(egui::Margin{left:0.0,right:6.0,top:0.0,bottom:0.0}))
            .show_inside(ui,|ui|{ui.spacing_mut().scroll=egui::style::ScrollStyle::solid();ui.spacing_mut().scroll.bar_inner_margin=6.0;
                egui::ScrollArea::vertical().id_salt("library_navigation_scroll").auto_shrink([false,false]).show(ui,|ui|self.library_sidebar(ui));});
        egui::Frame::none().inner_margin(egui::Margin{left:10.0,right:0.0,top:0.0,bottom:0.0}).show(ui,|ui|self.library_files(ui));
    }
    fn library_files(&mut self,ui:&mut egui::Ui) {
        use crate::library::SortCol;
        if library_capture_demo() && self.lib.index.entries.len() < 412 {
            if let Some(root) = self.lib.cfg.folders.first().cloned() {
                self.seed_library_demo_catalog(std::path::Path::new(&root));
                self.refresh_library_view();
            }
        }
        let mut append=false;
        ui.add_space(2.0);
        ui.horizontal(|ui|{
            let n=self.lib_sel.len();let label=if n>1 {format!("Add {n} to Mix")} else {"Add to Mix".into()};
            let add=egui::Button::new(egui::RichText::new(label).color(if n>0{TXT}else{DIM})).fill(if n>0{FOCUS}else{CTRL_FILL}).stroke(Stroke::new(1.0_f32, if n>0{FOCUS}else{LINE})).min_size(vec2(118.0,28.0));
            let width=(ui.available_width()-128.0).max(140.0);
            egui::Frame::none().fill(BG).rounding(Rounding::same(6.0)).stroke(Stroke::new(1.0_f32,LINE)).inner_margin(egui::Margin::symmetric(8.0,3.0)).show(ui,|ui|{
                ui.set_width(width-18.0);
                ui.horizontal(|ui|{
                    ui.label(egui::RichText::new("🔍").color(DIM));
                    let clear=!self.lib_q.is_empty();
                    ui.add(egui::TextEdit::singleline(&mut self.lib_q).hint_text("Search title, artist or file name").frame(false).desired_width(ui.available_width()-if clear {22.0} else {0.0}));
                    if clear && ui.add(egui::Button::new(egui::RichText::new("✖").size(11.0).color(DIM)).frame(false)).on_hover_text("Clear search").clicked(){self.lib_q.clear();}
                });
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_enabled(n>0&&!self.arrangement_locked(),add).on_hover_text("Append the selection after the last song (Enter). Double-click a row adds one file.").clicked(){append=true;}
            });
        });
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui|{
            toggle_button(ui, &mut self.lib_key, "Compatible key");
            toggle_button(ui, &mut self.lib_bpm, "BPM range");
            if self.lib_bpm {
                ui.add(egui::DragValue::new(&mut self.lib_pct).range(1.0..=30.0).speed(0.2).prefix("±").suffix(" %"));
            }
            ui.menu_button(egui::RichText::new("Columns v").size(11.5).color(DIM), |ui| {
                toggle_button(ui, &mut self.library_ui.details, "Folder & genre columns");
            });
            let total = self.lib.index.entries.len();
            ui.label(
                egui::RichText::new(if self.lib_sel.is_empty() {
                    format!("{total} tracks")
                } else {
                    format!("{total} tracks · {} selected", self.lib_sel.len())
                })
                .color(DIM)
                .size(11.5),
            );
        });
        self.refresh_library_view();
        ui.horizontal(|ui|{
            ui.label(egui::RichText::new(if self.lib_sel.is_empty() {format!("{} visible",self.lib_view.len())} else {format!("{} visible · {} selected",self.lib_view.len(),self.lib_sel.len())}).color(DIM).size(11.5));
            if self.lib.scanning{ui.spinner();ui.label(egui::RichText::new("Scanning folders…").size(11.5).color(DIM));}
            else if self.lib.analysing.1>0{let (done,total)=self.lib.analysing;ui.add(egui::ProgressBar::new(done as f32/total.max(1) as f32).desired_width(120.0).desired_height(8.0));ui.label(egui::RichText::new(format!("Analysing BPM/key {done}/{total}")).size(11.5).color(DIM));}
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|ui.label(egui::RichText::new("▶ previews · drag rows onto a deck · Ctrl/Shift selects · Enter adds").size(11.0).color(DIM)));
        });
        ui.add_space(2.0);
        if self.lib_view.is_empty(){ui.add_space(24.0);ui.vertical_centered(|ui|ui.label(egui::RichText::new(if self.lib.cfg.folders.is_empty()&&self.lib_node.is_none(){"Add a music folder to start your library."}else{"No files match. Clear the search or filters, or choose another folder."}).color(DIM)));}
        let mut columns=vec![(SortCol::Title,"Title"),(SortCol::Artist,"Artist"),(SortCol::Bpm,"BPM"),(SortCol::Key,"Key"),(SortCol::Duration,"Length")];
        columns.push((SortCol::Filename,"Status"));
        let roots=self.lib.cfg.folders.clone();
        let available=ui.available_height();let mods=ui.input(|i|i.modifiers);
        let mut table=egui_extras::TableBuilder::new(ui).id_salt("library_files_v3").striped(true).resizable(true).sense(Sense::click_and_drag()).max_scroll_height(available - 44.0).min_scrolled_height(0.0)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::exact(34.0))
            .column(egui_extras::Column::remainder().at_least(140.0))
            .column(egui_extras::Column::initial(120.0).at_least(72.0))
            .column(egui_extras::Column::initial(56.0).at_least(44.0))
            .column(egui_extras::Column::initial(52.0).at_least(40.0))
            .column(egui_extras::Column::initial(58.0).at_least(48.0))
            .column(egui_extras::Column::initial(72.0).at_least(56.0));
        let mut sorted=None;let mut click=None;let mut double=None;let mut drag=None;let mut listen=None;
        table.header(26.0,|mut header|{header.col(|_|{});for (col,name) in &columns {header.col(|ui|{
            let on=self.lib_sort==*col; let arrow=if on {if self.lib_asc {" ⏶"} else {" ⏷"}} else {""};
            if ui.add(egui::Button::new(egui::RichText::new(format!("{name}{arrow}")).size(11.5).strong().color(if on {FOCUS} else {DIM})).frame(false)).clicked(){sorted=Some(*col);}
        });}})
            .body(|body|{body.rows(36.0,self.lib_view.len(),|mut row|{
                let k=row.index();let e=&self.library_entries()[self.lib_view[k]];let selected=self.lib_sel.contains(&e.path);row.set_selected(selected);
                let unavailable=e.error.is_some() || (e.folder!=crate::library::EXT_FOLDER && matches!(self.lib.states.get(&e.folder),Some(crate::library::FolderState::Missing|crate::library::FolderState::NotResponding)));
                let color=if unavailable{DIM}else{TXT};
                row.col(|ui|{
                    let on=self.listening(&e.path);
                    let button=egui::Button::new(egui::RichText::new(if on {"⏹"} else {"▶"}).size(12.0).color(if on {BG} else {TXT})).fill(if on {ACCENT} else {C::from_rgb(52,54,60)}).rounding(Rounding::same(12.0)).min_size(vec2(24.0,24.0));
                    if ui.add_enabled(!unavailable,button).on_hover_text(if on {"Stop preview"} else {"Preview from the first drop at original tempo"}).clicked(){listen=Some((e.path.clone(),e.filename().to_string()));}
                });
                let title = if e.title.is_empty() { e.filename().to_string() } else { e.title.clone() };
                row.col(|ui|{ui.add(egui::Label::new(egui::RichText::new(title).strong().size(13.0).color(color)).truncate());});
                row.col(|ui|{ui.add(egui::Label::new(egui::RichText::new(if e.artist.is_empty() { "—" } else { &e.artist }).size(12.0).color(DIM)).truncate());});
                row.col(|ui|{ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.add_space(8.0);ui.label(egui::RichText::new(e.bpm.map(|b|format!("{b:.1}")).unwrap_or_else(||"—".into())).size(12.5).color(if e.bpm.is_some() {color} else {DIM}));});});
                row.col(|ui|{match &e.key {Some(key)=>chip(ui,key.clone(),if unavailable {DIM} else {key_col(key)}),None=>{ui.label(egui::RichText::new("—").color(DIM));}}});
                row.col(|ui|{ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.add_space(8.0);ui.label(egui::RichText::new(fmt_t(e.duration)).size(12.0).color(color));});});
                let in_mix = self.mix.tracks.iter().any(|t| t.path == e.path);
                let status = if in_mix { "In mix" } else if e.bpm.is_some() && e.key.is_some() { "Ready" } else if unavailable { "Missing" } else { "Indexed" };
                row.col(|ui|{ui.label(egui::RichText::new(status).size(11.5).color(if in_mix { FOCUS } else { DIM }));});
                let response=if library_capture_demo() {
                    row.response()
                } else {
                    row.response().on_hover_ui(|ui|{ui.set_max_width(480.0);ui.strong(e.filename());ui.label(format!("{} · {}",e.artist,e.title));ui.label(egui::RichText::new(&e.path).size(11.0).color(DIM));
                        if let Some((source,track))=self.lib.ext_info(&e.path){ui.label(format!("{} tags: {} BPM · {}",source.name(),track.bpm.map(|b|format!("{b:.2}")).unwrap_or_else(||"unknown".into()),track.key.as_deref().unwrap_or("unknown")));}
                        if let Some(error)=&e.error{ui.colored_label(C::LIGHT_RED,error);}else if unavailable{ui.colored_label(C::YELLOW,"Source drive is unavailable");}
                    })
                };
                if response.double_clicked(){double=Some(e.path.clone());}else if response.clicked(){click=Some((k,mods));}
                if response.drag_started()&&!self.arrangement_locked(){let paths=if selected{self.library_selected_paths()}else{vec![e.path.clone()]};response.dnd_set_drag_payload(LibDrag(paths.clone()));drag=Some(paths);}
            });});
        if let Some(col)=sorted{if self.lib_sort==col{self.lib_asc=!self.lib_asc;}else{self.lib_sort=col;self.lib_asc=true;}}
        if let Some((path,title))=listen{let ctx=ui.ctx().clone();self.toggle_listen(path,title,&ctx);}
        if let Some(paths)=drag{self.lib_sel=paths.into_iter().collect();}
        if let Some((k,mods))=click{self.select_library_row(k,mods);}
        if let Some(path)=double{self.lib_add(vec![path]);}else if append{self.lib_add(self.library_selected_paths());}
        ui.add_space(4.0);
        egui::Frame::none().fill(ROW).inner_margin(egui::Margin::symmetric(10.0,6.0)).rounding(6.0).show(ui,|ui|{
            if self.listen.is_some() { self.listen_ui(ui); }
            else { ui.horizontal(|ui|{ui.label(egui::RichText::new("▶ Preview — select a row and press play").size(11.5).color(DIM));}); }
        });
    }
    /// Enter adds the selection; Ctrl+A selects every visible file.
    pub(super) fn library_keys(&mut self,ctx:&egui::Context) {
        if self.tab!=Tab::Library {return;}
        if ctx.input(|i|i.key_pressed(egui::Key::Enter)) && !self.lib_sel.is_empty() && !self.arrangement_locked() {self.lib_add(self.library_selected_paths());}
        if ctx.input(|i|i.modifiers.command && i.key_pressed(egui::Key::A)) {
            let paths:Vec<_>=self.lib_view.iter().filter_map(|&i|self.library_entries().get(i)).map(|e|e.path.clone()).collect();
            self.lib_anchor=paths.first().cloned(); self.lib_sel=paths.into_iter().collect();
        }
    }
    pub(super) fn select_library_row(&mut self,k:usize,mods:egui::Modifiers) {
        let Some(entry)=self.lib_view.get(k).and_then(|&i|self.library_entries().get(i))else{return;};let path=entry.path.clone();
        if mods.shift{let anchor=self.lib_anchor.as_ref().and_then(|p|self.lib_view.iter().position(|&i|self.library_entries()[i].path==*p)).unwrap_or(k);self.lib_anchor=Some(self.library_entries()[self.lib_view[anchor]].path.clone());let paths:Vec<_>=self.lib_view[anchor.min(k)..=anchor.max(k)].iter().map(|&i|self.library_entries()[i].path.clone()).collect();if !mods.command{self.lib_sel.clear();}self.lib_sel.extend(paths);}
        else if mods.command{if !self.lib_sel.remove(&path){self.lib_sel.insert(path.clone());}self.lib_anchor=Some(path.clone());}
        else{self.lib_sel.clear();self.lib_sel.insert(path.clone());self.lib_anchor=Some(path.clone());}
    }
}
/// A file's folder relative to the deepest music root containing it, compared by whole path components
/// (so `/Music/House2/x` is not inside `/Music/House`). Files directly in a root show the root's name.
pub(super) fn relative_folder(path:&str,roots:&[String])->String {
    let parent=std::path::Path::new(path).parent().unwrap_or(std::path::Path::new(""));
    roots.iter().filter_map(|r|parent.strip_prefix(r).ok().map(|rest|(r,rest))).max_by_key(|(r,_)|r.len())
        .map(|(r,rest)|if rest.as_os_str().is_empty() { std::path::Path::new(r).file_name().map(|n|n.to_string_lossy().into_owned()).unwrap_or_else(||r.clone()) } else { rest.display().to_string() })
        .unwrap_or_else(||parent.display().to_string())
}
fn sort_name(col:crate::library::SortCol)->&'static str {use crate::library::SortCol::*;match col{Filename=>"Filename",Title=>"Title",Artist=>"Artist",Bpm=>"BPM",Key=>"Key",Duration=>"Length",Genre=>"Genre",Folder=>"Folder"}}
