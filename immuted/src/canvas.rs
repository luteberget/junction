use std::ffi::CString;
use crate::model::*;
use std::collections::{HashSet, HashMap};
use crate::ui;
use crate::objects::*;
use crate::util;
use crate::config::*;
use crate::dispatch;
use crate::dispatch::*;
use crate::view::*;
use crate::interlocking::*;
use crate::viewmodel::*;
use crate::diagram::Diagram;
use crate::ui::ImVec2;
use backend_glfw::imgui::*;
use nalgebra_glm as glm;
use const_cstr::const_cstr;
use matches::matches;
use rolling::input::staticinfrastructure as rolling_inf;


#[derive(Debug)]
pub struct Canvas {
    pub action :Action,
    pub selection :HashSet<Ref>,
    pub view :View,
    pub active_dispatch :Option<(usize, f32, bool)>,
    pub instant_cache: dispatch::InstantCache,
}


#[derive(Debug)]
pub enum Action {
    Normal(NormalState),
    DrawingLine(Option<Pt>),
    InsertObject(Option<Object>),
}

#[derive(Debug,Copy,Clone)]
pub enum NormalState {
    Default,
    SelectWindow(ImVec2),
    DragMove(MoveType),
}

#[derive(Debug,Copy,Clone)]
pub enum MoveType { Grid(PtC), Continuous }

impl Canvas {
    pub fn new() -> Self {
        Self {
            action :Action::Normal(NormalState::Default),
            selection :HashSet::new(),
            view :View::default(),
            active_dispatch :None,
            instant_cache: dispatch::InstantCache::new(),
        }
    }

    //pub fn toolbar(&mut self, doc :&mut Undoable<Model>) {
    pub fn toolbar(&mut self, vm :&ViewModel) { unsafe {
        let m = vm.get_undoable().get();
        if tool_button(const_cstr!("select (A)").as_ptr(),
            'A' as _, matches!(&self.action, Action::Normal(_))) {
            self.action = Action::Normal(NormalState::Default);
        }
        igSameLine(0.0,-1.0);
        if tool_button(const_cstr!("insert object (S)").as_ptr(),
            'S' as _, matches!(&self.action, Action::InsertObject(_))) {
            self.action = Action::InsertObject(None);
        }
        igSameLine(0.0,-1.0);
        if tool_button(const_cstr!("draw track (D)").as_ptr(),
            'D' as _, matches!(&self.action, Action::DrawingLine(_))) {
            self.action = Action::DrawingLine(None);
        }

        // select active dispatch
        igSameLine(0.0,-1.0);
        let curr_name = if let Some((d,_t,_play)) = self.active_dispatch { CString::new(format!("Dispatch {}", d)).unwrap() } else { CString::new("None").unwrap() };
        if igBeginCombo(const_cstr!("Dispatch").as_ptr(), curr_name.as_ptr(), 0) {
            if igSelectable(const_cstr!("None").as_ptr(), self.active_dispatch.is_none(), 0 as _,
                            util::to_imvec(glm::zero())) {
                self.active_dispatch = None;
            }
            for (idx,_dispatch) in m.dispatches.iter().enumerate() {

                igPushIDInt(idx as _);
                if igSelectable(const_cstr!("##dispatch").as_ptr(), 
                                 self.active_dispatch.map(|(i,_t,_play)| i) == Some(idx), 0 as _,
                                util::to_imvec(glm::zero())) {
                    let t = self.instant_cache.dispatch_time(idx).unwrap_or(0.0);
                    self.active_dispatch = Some((idx, t, false));
                }
                igSameLine(0.0,-1.0); ui::show_text(&format!("Dispatch {}", idx));
                igPopID();
            }
            igEndCombo();
        }


    } }

    pub fn node_editor(doc :&mut ViewModel, pt :Pt) -> Option<()> {
        let (nd,_tangent) = doc.get_data().topology.as_ref()?.locations.get(&pt)?;
        unsafe {
        match nd {
            NDType::OpenEnd | NDType::BufferStop => {
                if let Some(new_value) = 
                    ui::radio_select(&[(const_cstr!("Open end").as_ptr(), *nd == NDType::OpenEnd, NDType::OpenEnd),
                                       (const_cstr!("Buffer stop").as_ptr(), *nd == NDType::BufferStop, NDType::BufferStop)]) {
                    let mut new_model = doc.get_undoable().get().clone();
                    new_model.node_data.insert(pt, *new_value);
                    doc.set_model(new_model, None);
                }
            },
            NDType::Sw(side) => {
                ui::show_text(&format!("Switch ({:?})", side));

                // TODO 
                let mut speed = 60.0;
                igInputFloat(const_cstr!("Deviating speed restr.").as_ptr(), &mut speed, 1.0, 10.0, 
                             const_cstr!("%.1f").as_ptr(), 0 as _);
            },
            NDType::Crossing(type_) => {
                ui::show_text(&format!("Crossing ({:?})", type_));
                if let Some(new_value) = 
                    ui::radio_select(&[(const_cstr!("Crossover").as_ptr(), *type_ == CrossingType::Crossover, CrossingType::Crossover),
                                       (const_cstr!("Single slip (above)").as_ptr(), *type_ == CrossingType::SingleSlip(Side::Left), CrossingType::SingleSlip(Side::Left)),
                                       (const_cstr!("Single slip (below)").as_ptr(), *type_ == CrossingType::SingleSlip(Side::Right), CrossingType::SingleSlip(Side::Right)),
                                       (const_cstr!("Double slip").as_ptr(), *type_ == CrossingType::DoubleSlip, CrossingType::DoubleSlip)]) {

                    let mut new_model = doc.get_undoable().get().clone();
                    new_model.node_data.insert(pt, NDType::Crossing(*new_value));
                    doc.set_model(new_model, None);
                }

                // TODO 
                let mut speed = 60.0;
                igInputFloat(const_cstr!("Deviating speed restr.").as_ptr(), &mut speed, 1.0, 10.0, 
                             const_cstr!("%.1f").as_ptr(), 0 as _);
            }
            _ => {},
        }
        }
        Some(())
    }

    pub fn context_menu_contents(&mut self, doc :&mut ViewModel, preview_route :&mut Option<usize>) {
        unsafe {
            ui::show_text(&format!("selection: {:?}", self.selection));
            //
            // TODO cache some info about selection? In case it is very big and we need to know
            // every frame whether it contains a Node or not.
            // 

            if self.selection.len() == 1 {
                if let Some(Ref::Node(pt)) = self.selection.iter().cloned().nth(0) {
                    Self::node_editor(doc, pt);
                }
            }

            ui::sep();
            if !self.selection.is_empty() {
                if igSelectable(const_cstr!("Delete").as_ptr(), false, 0 as _, util::to_imvec(glm::zero())) {
                    self.delete_selection(doc);
                }
            }

            ui::sep();
            let mut dispatch_action = None;
            if self.selection.len() == 1 {


                // Object menu
                if let Some(Ref::Object(pta)) = self.selection.iter().cloned().nth(0) {
                    self.object_menu(pta, doc);
                }

                if let Some(il) = doc.get_data().interlocking.as_ref() {
                    if let Some(Ref::Node(pt)) = self.selection.iter().cloned().nth(0) {
                        if let Some(rs) = il.boundary_routes.get(&pt) {
                            let (preview,action) = Self::route_selector(il,rs);
                            *preview_route = preview;
                            dispatch_action = action;
                        }
                    }
                    if let Some(Ref::Object(pta)) = self.selection.iter().cloned().nth(0) {
                        if let Some(rs) = il.signal_routes.get(&pta) {
                            let (preview,action) = Self::route_selector(il,rs);
                            *preview_route = preview;
                            dispatch_action = action;
                        }
                    }
                }
            }

            // This can be moved inside the route_selector?
            if let Some(route_id) = dispatch_action {
                self.start_route(doc, route_id);
            }
        }
    }

    fn object_menu(&mut self, pta :PtA, doc :&mut ViewModel) -> Option<()> {
        let m = doc.get_undoable().get();
        let obj = m.objects.get(&pta)?;

        let mut set_distant = None;
        for f in obj.functions.iter() {
            match f {
                Function::Detector => { ui::show_text("Detector"); },
                Function::MainSignal { has_distant } => {
                    ui::show_text("Main signal");
                    let mut has_distant = *has_distant;
                    unsafe {
                        igCheckbox(const_cstr!("Distant signal").as_ptr(), &mut has_distant);
                        if igIsItemEdited() { 
                            set_distant = Some(has_distant);
                        }
                    }
                }
            }
        }
        if let Some(d) = set_distant {
            doc.edit_model(|new| {
                new.objects.get_mut(&pta).unwrap().functions = vec![Function::MainSignal { has_distant: d }];
                None
            });
        }
        Some(())
    }

    //fn set_distant(&mut self, doc :&mut ViewModel, :W

    fn start_route(&mut self, doc:&mut ViewModel, route_idx :usize) {
        if let Some(il) = doc.get_data().interlocking.as_ref() {
            //println!("Dispatching route {}", route_idx);
            let mut model = doc.get_undoable().get().clone();
            let (dispatch_idx,time,play) = self.active_dispatch.unwrap_or_else(|| {
                model.dispatches.push_back(Default::default()); // empty dispatch
                let d = (model.dispatches.len()-1, 0.0, true);
                self.active_dispatch = Some(d);
                d
            });

            let dispatch = model.dispatches.get_mut(dispatch_idx).unwrap();
            let cmd = match (il.routes[route_idx].0).entry {
                rolling_inf::RouteEntryExit::Boundary(_) => 
                    Command::Train { route: route_idx, vehicle: 0 },
                rolling_inf::RouteEntryExit::Signal(_) | rolling_inf::RouteEntryExit::SignalTrigger {..} => 
                    Command::Route { route: route_idx },
            };
            dispatch.insert(time as f64, cmd);
            doc.set_model(model,None);
            //println!("DISPATCHES: {:?}", doc.get_undoable().get().dispatches);
        }
    }

    fn route_selector(il :&Interlocking, routes :&[usize]) -> (Option<usize>,Option<usize>) {
        unsafe {
            let mut some = false;
            let mut preview = None;
            let mut action = None;
            for idx in routes {
                some = true;
                igPushIDInt(*idx as _);
                if igSelectable(const_cstr!("##route").as_ptr(), false, 
                                0 as _, util::to_imvec(glm::zero())) {
                    //self.start_boundary_route(doc, *idx);
                    action = Some(*idx);
                }
                if igIsItemHovered(0) {
                    preview = Some(*idx);
                }
                igSameLine(0.0,-1.0); ui::show_text(&format!("Route to {:?}", 
                                                (il.routes[*idx].0).exit));

                igPopID();

            }
            if !some {
                ui::show_text("No routes.");
            }
            (preview,action)
        }
    }

    pub fn draw(&mut self, doc :&mut ViewModel, config :&Config, diagram :&mut Diagram) {
        self.toolbar(doc);

        let zero = ImVec2 { x: 0.0, y: 0.0 };
        use backend_glfw::imgui::*;
        let size = unsafe { igGetContentRegionAvail_nonUDT2().into() };
        ui::canvas(size, config.color_u32(RailUIColorName::CanvasBackground),
                   const_cstr!("railwaycanvas").as_ptr(), |draw_list, pos| { unsafe {

            // TODO move keyboard shortcuts out of Canvas
            // Hotkeys
            self.handle_global_keys(doc, diagram);
            //hotkey!(CTRL+Z, { doc.undo(); });
            let handle_keys = igIsItemActive() || !igIsAnyItemActive();
            if handle_keys { self.handle_keys(); }

            // Scroll action (wheel or ctrl-drag)
            self.scroll();

            let io = igGetIO();
            let pointer = (*io).MousePos - pos;
            let pointer_ongrid = self.view.screen_to_world_pt(pointer);
            let pointer_ingrid = self.view.screen_to_world_ptc(pointer);

            // Context menu 
            let mut preview_route = None;

            //igPushStyleVarColor(ImGuiStyleVar__ImGuiStyleVar_Alpha, 0.5);

            if igBeginPopup(const_cstr!("ctx").as_ptr(), 0 as _) {
                self.context_menu_contents(doc, &mut preview_route);
                igEndPopup();
            }
            //igPopStyleVar(1);

            if igIsItemHovered(0) && igIsMouseClicked(1,false) {
                if let Some((r,_)) = doc.get_closest(pointer_ingrid) {
                    if !self.selection.contains(&r) {
                        self.selection = std::iter::once(r).collect();
                    }
                }
                igOpenPopup(const_cstr!("ctx").as_ptr());
            }

            // Edit actions 
            match &mut self.action {
                Action::Normal(normal) => {
                    let normal = *normal;
                    self.normalstate(normal, doc, draw_list, pointer_ingrid, pos, config);
                }
                Action::DrawingLine(from) => {
                    let from = *from;
                    self.drawingline(doc,from,pos,pointer_ongrid,draw_list, config);
                }
                Action::InsertObject(None) => {
                },
                Action::InsertObject(Some(obj)) => {
                    let moved = obj.move_to(doc.get_undoable().get(),pointer_ingrid);
                    obj.draw(pos,&self.view,draw_list,config.color_u32(RailUIColorName::CanvasSymbol),&[],config);
                    if let Some(err) = moved {
                        let p = pos + self.view.world_ptc_to_screen(obj.loc);
                        let window = ImVec2 { x: 4.0, y: 4.0 };
                        ImDrawList_AddRect(draw_list, p - window, p + window,
                                           config.color_u32(RailUIColorName::CanvasSymbolLocError),
                                           0.0,0,4.0);
                    } else  {
                        if igIsMouseReleased(0) {
                            let mut m = doc.get_undoable().get().clone();
                            m.objects.insert(round_coord(obj.loc), obj.clone());
                            doc.set_model(m, None);
                        }
                    }
                },
            };

            // Draw background
            self.draw_background(&doc, draw_list, pos, size, config);

            // Draw highlightred route
            if let Some(idx) = preview_route {
                self.draw_route(&doc, draw_list, pos, size, idx, config);
            }

            // Draw occupied sections and signal aspects // TODO switch psoitions
            self.draw_inf_state(&doc, draw_list, pos, size, config);

            // Draw train locations
            self.draw_trains(&doc, draw_list, pos, size, config);

            Some(())
        }});
    }

    pub fn handle_keys(&mut self) {
        unsafe {
        if igIsKeyPressed('A' as _, false) {
            self.action = Action::Normal(NormalState::Default);
        }
        if igIsKeyPressed('D' as _, false) {
            self.action = Action::DrawingLine(None);
        }
        if igIsKeyPressed('S' as _, false) {
            let current_object_function = if let Action::InsertObject(Some(obj)) = &self.action {
                obj.functions.iter().next()
            } else { None };

            if current_object_function == Some(&Function::Detector) {
                    self.action = Action::InsertObject(Some(
                            Object { 
                                loc: glm::vec2(0.0,0.0), 
                                tangent :glm::vec2(1,0),
                                functions: vec![Function::MainSignal { has_distant: false }] } ));

            } else {
                    self.action = Action::InsertObject(Some(
                            Object { 
                                loc: glm::vec2(0.0,0.0), 
                                tangent :glm::vec2(1,0),
                                functions: vec![Function::Detector] } ));
            }
        }
        }
    }

    pub fn handle_global_keys(&mut self, doc :&mut ViewModel, diagram :&mut Diagram) { unsafe {
        let io = igGetIO();
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            doc.undo();
        }
        if (*io).KeyCtrl && (*io).KeyShift && igIsKeyPressed('Z' as _, false) {
            doc.redo();
        }
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('Y' as _, false) {
            doc.redo();
        }

        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('S' as _, false) {
            crate::mainmenu::save(doc);
        }
        if (*io).KeyCtrl && (*io).KeyShift && igIsKeyPressed('S' as _, false) {
            crate::mainmenu::save_as(doc);
        }
        if (*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed('O' as _, false) {
            crate::mainmenu::load(doc, self, diagram);
        }

        if !(*io).KeyCtrl && !(*io).KeyShift && igIsKeyPressed(' ' as _, false) {
            if let Some((_,_,play)) = self.active_dispatch.as_mut() {
                *play = !*play;
            }
        }

    } }

    pub fn scroll(&mut self) {
        unsafe {
            //if !igIsWindowFocused(0 as _) { return; }
            if !igIsItemHovered(0){ return; }
            let io = igGetIO();
            let wheel = (*io).MouseWheel;
            if wheel != 0.0 {
                self.view.zoom(wheel);
            }
            if ((*io).KeyCtrl && igIsMouseDragging(0,-1.0)) || igIsMouseDragging(2,-1.0) {
                self.view.translate((*io).MouseDelta);
            }
        }
    }

    pub fn get_symm<'a, K:std::hash::Hash+std::cmp::Eq+Copy, V>
            (map :&'a HashMap<(K,K), V>, (a,b) :(K,K)) -> Option<&'a V> {
        if let Some(x) = map.get(&(a,b)) { return Some(x); }
        if let Some(x) = map.get(&(b,a)) { return Some(x); }
        None
    }

    pub fn draw_inf_state(&mut self, vm :&ViewModel, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2,config :&Config) -> Option<()> {
        let (idx,time,_play) = self.active_dispatch.as_ref()?;
        let instant = self.instant_cache.get_instant(vm, *idx, *time)?;

        for (_tvd, status, lines) in instant.infrastructure.sections.iter() {
            let color = match status {
                SectionStatus::Occupied => config.color_u32(RailUIColorName::CanvasTVDOccupied),
                SectionStatus::Reserved => config.color_u32(RailUIColorName::CanvasTVDReserved),
                _ => config.color_u32(RailUIColorName::CanvasTVDFree),
            };

            for (p1,p2) in lines.iter() {
                unsafe {
                    ImDrawList_AddLine(draw_list,
                                       pos + self.view.world_ptc_to_screen(*p1),
                                       pos + self.view.world_ptc_to_screen(*p2),
                                       color, 4.0);
                }
            }
        }

        Some(())
    }

    pub fn draw_trains(&mut self, vm :&ViewModel, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2, config :&Config) ->Option<()> {
        let (idx,time,_play) = self.active_dispatch.as_ref()?;
        let instant = self.instant_cache.get_instant(vm, *idx, *time)?;
        let color = config.color_u32(RailUIColorName::CanvasTrain);
        let sight_color = config.color_u32(RailUIColorName::CanvasTrainSight);
        for t in instant.trains.iter() {
            for (p1,p2) in t.lines.iter() {
                unsafe {
                ImDrawList_AddLine(draw_list,
                                   pos + self.view.world_ptc_to_screen(*p1),
                                   pos + self.view.world_ptc_to_screen(*p2),
                                   color, 7.0);
                }
            }

            if let Some(front) = t.get_front() {
                for pta in t.signals_sighted.iter() {
                    unsafe {
                    ImDrawList_AddLine(draw_list,
                                       pos + self.view.world_ptc_to_screen(front),
                                       pos + self.view.world_ptc_to_screen(unround_coord(*pta)),
                                       sight_color, 1.0);
                    }
                }
            }

        }


        Some(())
    }

    pub fn draw_route(&self, vm :&ViewModel, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2, route_idx: usize, config :&Config) -> Option<()> {
        unsafe {
        let il = vm.get_data().interlocking.as_ref()?;
        let dgraph = vm.get_data().dgraph.as_ref()?;
        let (route,route_nodes) = &il.routes[route_idx];
        let color_path = config.color_u32(RailUIColorName::CanvasRoutePath);
        let color_section = config.color_u32(RailUIColorName::CanvasRouteSection);

        for sec in route.resources.sections.iter() {
            if let Some(edges) = dgraph.tvd_edges.get(sec) {
                for (a,b) in edges.iter() {
                    if let Some(v) = Self::get_symm(&dgraph.edge_lines, (*a,*b)) {
                        for (pt_a,pt_b) in v.iter().zip(v.iter().skip(1)) {
                            ImDrawList_AddLine(draw_list,
                                               pos + self.view.world_ptc_to_screen(*pt_a),
                                               pos + self.view.world_ptc_to_screen(*pt_b),
                                               color_section, 3.5);
                        }
                    }
                }
            }
        }

        for (a,b) in route_nodes {
            if let Some(v) = Self::get_symm(&dgraph.edge_lines, (*a,*b)) {
                for (pt_a,pt_b) in v.iter().zip(v.iter().skip(1)) {
                    ImDrawList_AddLine(draw_list,
                                       pos + self.view.world_ptc_to_screen(*pt_a),
                                       pos + self.view.world_ptc_to_screen(*pt_b),
                                       color_path, 6.0);
                }
            }
        }
        // TODO highlight end signal/boundary

        Some(())
        }
    }

    pub fn draw_background(&self, vm :&ViewModel, draw_list :*mut ImDrawList, pos :ImVec2, size :ImVec2, config :&Config) {
        let empty_state = HashMap::new();
        let mut object_states :&HashMap<PtA,Vec<ObjectState>> = &empty_state;
        if let Some((idx,time,_play)) = self.active_dispatch.as_ref() {
            if let Some(instant) = self.instant_cache.get_cached_instant(vm, *idx, *time) {
                object_states = &instant.infrastructure.object_state;
            }
        }

        let m = vm.get_undoable().get();
        let d = vm.get_data();

        unsafe {

            let sel_window = if let Action::Normal(NormalState::SelectWindow(a)) = &self.action {
                Some((*a, *a + igGetMouseDragDelta_nonUDT2(0,-1.0).into()))
            } else { None };

            let (lo,hi) = self.view.points_in_view(size);
            let color_grid = config.color_u32(RailUIColorName::CanvasGridPoint);
            for x in lo.x..=hi.x {
                for y in lo.y..=hi.y {
                    let pt = self.view.world_pt_to_screen(glm::vec2(x,y));
                    ImDrawList_AddCircleFilled(draw_list, pos+pt, 3.0, color_grid, 4);
                }
            }

            let color_line = config.color_u32(RailUIColorName::CanvasTrack);
            let color_line_selected = config.color_u32(RailUIColorName::CanvasTrackSelected);
            for l in &m.linesegs {
                let p1 = self.view.world_pt_to_screen(l.0);
                let p2 = self.view.world_pt_to_screen(l.1);
                let selected = self.selection.contains(&Ref::LineSeg(l.0,l.1));
                let preview = sel_window
                    .map(|(a,b)| util::point_in_rect(p1,a,b) || util::point_in_rect(p2,a,b))
                    .unwrap_or(false) ;
                let col = if selected || preview { color_line_selected } else { color_line };
                ImDrawList_AddLine(draw_list, pos + p1, pos + p2, col, 2.0);
            }

            let color_node = config.color_u32(RailUIColorName::CanvasNode);
            let color_node_selected = config.color_u32(RailUIColorName::CanvasNodeSelected);
            if let Some(topo) = d.topology.as_ref() {
                use nalgebra_glm::{vec2, rotate_vec2, radians, vec1, normalize};
                for (pt0,(t,vc)) in &topo.locations {
                    let selected = self.selection.contains(&Ref::Node(*pt0));
                    let preview = sel_window.map(|(a,b)| 
                             util::point_in_rect(self.view.world_pt_to_screen(*pt0),a,b)).unwrap_or(false);
                    let col = if selected || preview { color_node_selected } 
                                else { color_node };

                    let pt :PtC = vec2(pt0.x as _ ,pt0.y as _ );
                    let tangent :PtC = vec2(vc.x as _ ,vc.y as _ );
                    match t {
                        NDType::OpenEnd => {
                            for angle in &[-45.0,45.0] {
                                ImDrawList_AddLine(draw_list,
                                   pos + self.view.world_ptc_to_screen(pt),
                                   pos + self.view.world_ptc_to_screen(pt) 
                                    + util::to_imvec(8.0*rotate_vec2(&normalize(&tangent),radians(&vec1(*angle)).x)), col, 2.0);
                            }
                        },
                        NDType::Cont => {
                            ImDrawList_AddCircleFilled(draw_list, 
                                pos + self.view.world_ptc_to_screen(pt), 4.0, col, 8);
                        },
                        NDType::Sw(side) => {
                            let angle = if matches!(side, Side::Left) { 45.0 } else { -45.0 };
                            let p1 = pos + self.view.world_ptc_to_screen(pt);
                            let p2 = p1 + util::to_imvec(15.0*normalize(&tangent));
                            let p3 = p1 + util::to_imvec(15.0*rotate_vec2(&(1.41*normalize(&tangent)), radians(&vec1(angle)).x));
                            ImDrawList_AddTriangleFilled(draw_list, p1,p2,p3, col);
                        },
                        NDType::Err =>{
                            let p = pos + self.view.world_ptc_to_screen(pt);
                            let window = ImVec2 { x: 4.0, y: 4.0 };
                            ImDrawList_AddRect(draw_list, p - window, p + window,
                                               config.color_u32(RailUIColorName::CanvasNodeError),
                                               0.0,0,4.0);
                        },
                        NDType::BufferStop => {
                            let tangent = util::to_imvec(normalize(&tangent));
                            let normal = ImVec2 { x: -tangent.y, y: tangent.x };

                            let node = pos + self.view.world_ptc_to_screen(pt);
                            let pline :&[ImVec2] = &[node + 8.0*normal + 4.0 * tangent,
                                                  node + 8.0*normal,
                                                  node - 8.0*normal,
                                                  node - 8.0*normal + 4.0 * tangent];

                            ImDrawList_AddPolyline(draw_list,pline.as_ptr(), pline.len() as i32, col, false, 2.0);

                        },
                        NDType::Crossing(type_) => {
                            let left_conn  = matches!(type_, CrossingType::DoubleSlip | CrossingType::SingleSlip(Side::Left));
                            let right_conn = matches!(type_, CrossingType::DoubleSlip | CrossingType::SingleSlip(Side::Right));

                            let tangenti = util::to_imvec(normalize(&tangent));
                            let normal = ImVec2 { x: tangenti.y, y: tangenti.x };

                            if right_conn {
                                let base = pos + self.view.world_ptc_to_screen(pt) - 4.0*normal - 2.0f32.sqrt()*2.0*tangenti;
                                let pline :&[ImVec2] = &[base - 8.0*tangenti,
                                                         base,
                                                         base + 8.0*util::to_imvec(rotate_vec2(&tangent, radians(&vec1(45.0)).x))];
                                ImDrawList_AddPolyline(draw_list,pline.as_ptr(), pline.len() as i32, col, false, 2.0);
                            }

                            if left_conn {
                                let base = pos + self.view.world_ptc_to_screen(pt) + 4.0*normal + 2.0f32.sqrt()*2.0*tangenti;
                                let pline :&[ImVec2] = &[base + 8.0*tangenti,
                                                         base,
                                                         base - 8.0*util::to_imvec(rotate_vec2(&tangent, radians(&vec1(45.0)).x))];
                                ImDrawList_AddPolyline(draw_list,pline.as_ptr(), pline.len() as i32, col, false, 2.0);
                            }

                            if left_conn || right_conn {
                                let p = pos + self.view.world_ptc_to_screen(pt);
                                let pa = util::to_imvec(15.0*normalize(&tangent));
                                let pb = util::to_imvec(15.0*rotate_vec2(&normalize(&tangent), radians(&vec1(45.0)).x));
                                ImDrawList_AddTriangleFilled(draw_list,p,p+pa,p+pb,col);
                                ImDrawList_AddTriangleFilled(draw_list,p,p-pa,p-pb,col);
                            } else {
                                ImDrawList_AddCircleFilled(draw_list, pos + self.view.world_ptc_to_screen(pt), 4.0, col, 8);
                            }
                        },
                    }
                }
            }


            let color_obj = config.color_u32(RailUIColorName::CanvasSymbol);
            let color_obj_selected = config.color_u32(RailUIColorName::CanvasSymbolSelected);

            for (pta,obj) in &m.objects {
                let selected = self.selection.contains(&Ref::Object(*pta));
                let preview = sel_window.map(|(a,b)| 
                         util::point_in_rect(self.view.
                                 world_ptc_to_screen(unround_coord(*pta)),a,b)).unwrap_or(false);
                let col = if selected || preview { color_obj_selected } else { color_obj };
                let empty = vec![];
                let state = object_states.get(pta).unwrap_or(&empty);
                obj.draw(pos, &self.view, draw_list, col, state, config);
            }
        }
    }

    pub fn set_selection_window(&mut self, doc :&ViewModel, a :ImVec2, b :ImVec2) {
        self.selection = doc.get_rect(self.view.screen_to_world_ptc(a),
                                            self.view.screen_to_world_ptc(b))
                        .into_iter().collect();
    }

    pub fn move_selected_objects(&mut self, doc :&mut ViewModel, delta :PtC) {
        let mut model = doc.get_undoable().get().clone();
        let mut changed_ptas = Vec::new();
        for id in self.selection.iter() {
            match id {
                Ref::Object(pta) => {
                    let mut obj = model.objects.get_mut(pta).unwrap().clone();
                    obj.move_to(&model, obj.loc + delta);
                    let new_pta = round_coord(obj.loc);
                    model.objects.remove(pta);
                    model.objects.insert(new_pta,obj);
                    if *pta != new_pta { changed_ptas.push((*pta,new_pta)); }
                },
                _ => {},
            }
        }

        let selection_before = self.selection.clone();

        for (a,b) in changed_ptas {
            self.selection.remove(&Ref::Object(a));
            self.selection.insert(Ref::Object(b));
        }

        doc.set_model(model, Some(EditClass::MoveObjects(selection_before)));
        doc.override_edit_class(EditClass::MoveObjects(self.selection.clone()));
    }

    pub fn normalstate(&mut self, state: NormalState, doc :&mut ViewModel,
                       draw_list :*mut ImDrawList, pointer_ingrid :PtC, pos :ImVec2, config :&Config) {
        unsafe {
        let io = igGetIO();
        match state {
            NormalState::SelectWindow(a) => {
                let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                if igIsMouseDragging(0,-1.0) {
                    ImDrawList_AddRect(draw_list, pos + a, pos + b,
                                       config.color_u32(RailUIColorName::CanvasSelectionWindow),
                                       0.0, 0, 1.0);
                } else {
                    self.set_selection_window(doc, a,b);
                    self.action = Action::Normal(NormalState::Default);
                }
            },
            NormalState::DragMove(typ) => {
                if igIsMouseDragging(0,-1.0) {
                    let delta = self.view.screen_to_world_ptc((*io).MouseDelta) -
                                self.view.screen_to_world_ptc(ImVec2 { x:0.0, y: 0.0 });
                    match typ {
                        MoveType::Continuous => { if delta.x != 0.0 || delta.y != 0.0 {
                            self.move_selected_objects(doc, delta); }},
                        MoveType::Grid(p) => {
                            self.action = Action::Normal(NormalState::DragMove(MoveType::Grid(p + delta)));
                        },
                    }
                } else {
                    self.action = Action::Normal(NormalState::Default);
                }
            }
            NormalState::Default => {
                if !(*io).KeyCtrl && igIsItemHovered(0) && igIsMouseDragging(0,-1.0) {
                    if let Some((r,_)) = doc.get_closest(pointer_ingrid) {
                        if !self.selection.contains(&r) {
                            self.selection = std::iter::once(r).collect();
                        }
                        if self.selection.iter().any(|x| matches!(x, Ref::Node(_)) || matches!(x, Ref::LineSeg(_,_))) {
                            self.action = Action::Normal(NormalState::DragMove(
                                    MoveType::Grid(glm::zero())));
                        } else {
                            self.action = Action::Normal(NormalState::DragMove(MoveType::Continuous));
                        }
                    } else {
                        let a = (*io).MouseClickedPos[0] - pos;
                        //let b = a + igGetMouseDragDelta_nonUDT2(0,-1.0).into();
                        self.action = Action::Normal(NormalState::SelectWindow(a));
                    }
                } else {
                    if igIsItemHovered(0) && igIsMouseReleased(0) {
                        if !(*io).KeyShift { self.selection.clear(); }
                        if let Some((r,_)) = doc.get_closest(pointer_ingrid) {
                            self.selection.insert(r);
                        } 
                    }
                }
            },
        }
        }
    }

    pub fn drawingline(&mut self,  doc :&mut ViewModel, from :Option<Pt>,
                       pos :ImVec2, pointer_ongrid :Pt, draw_list :*mut ImDrawList, config :&Config
                       ) {
        unsafe {
            let color = config.color_u32(RailUIColorName::CanvasTrackDrawing);
        // Draw preview
        if let Some(pt) = from {
            for (p1,p2) in util::route_line(pt, pointer_ongrid) {
                ImDrawList_AddLine(draw_list, pos + self.view.world_pt_to_screen(p1),
                                              pos + self.view.world_pt_to_screen(p2), 
                                              color, 2.0);
            }

            if !igIsMouseDown(0) {
                let mut new_model = doc.get_undoable().get().clone();
                let mut any_lines = false;
                for (p1,p2) in util::route_line(pt,pointer_ongrid) {
                    let unit = util::unit_step_diag_line(p1,p2);
                    for (pa,pb) in unit.iter().zip(unit.iter().skip(1)) {
                        any_lines = true;
                        new_model.linesegs.insert(util::order_ivec(*pa,*pb));
                    }
                }
                if any_lines { doc.set_model(new_model, None); }
                self.selection = std::iter::empty().collect();
                self.action = Action::DrawingLine(None);
            }
        } else {
            if igIsItemHovered(0) && igIsMouseDown(0) {
                self.action = Action::DrawingLine(Some(pointer_ongrid));
            }
        }
    } }

    pub fn delete_selection(&mut self, doc :&mut ViewModel) {
        let mut new_model = doc.get_undoable().get().clone();
        for x in self.selection.drain() {
            new_model.delete(x);
        }
        doc.set_model(new_model, None);
    }
}


fn tool_button(name :*const i8, char :i8, selected :bool) -> bool {
        unsafe {
        if selected {
            let c1 = ImVec4 { x: 0.4, y: 0.65,  z: 0.4, w: 1.0 };
            let c2 = ImVec4 { x: 0.5, y: 0.85, z: 0.5, w: 1.0 };
            let c3 = ImVec4 { x: 0.6, y: 0.9,  z: 0.6, w: 1.0 };
            igPushStyleColor(ImGuiCol__ImGuiCol_Button as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonHovered as _, c1);
            igPushStyleColor(ImGuiCol__ImGuiCol_ButtonActive as _, c1);
        }
        let clicked = igButton( name , ImVec2 { x: 0.0, y: 0.0 } );
        if selected {
            igPopStyleColor(3);
        }
        clicked
    }
}

pub fn unround_coord(p :PtA) -> PtC {
    let coeff = 10.0;
    glm::vec2(p.x as f32 / coeff, p.y as f32 / coeff)
}
pub fn round_coord(p :PtC) -> PtA {
    let coeff = 10.0;
    glm::vec2((p.x * coeff) as _, (p.y * coeff) as _)
}

