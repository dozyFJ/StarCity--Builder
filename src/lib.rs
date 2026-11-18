//! StarCity Builder, a portrait city builder in Rust and wgpu.
pub mod game;
mod pet_render;
mod pet_ui;
mod updates;
mod board_ui;
mod gui_state;
mod sidebar_ui;
mod panels_ui;
mod inventory_ui;
mod save_format;

use std::{sync::Arc, time::Instant};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}};
use wolfcraft_common::gpu::GpuContext;

struct Graphics {
    gpu: GpuContext,
    egui: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gfx: Option<Graphics>,
    game: game::Game,
    last_tick: Option<Instant>,
    save_path: Option<std::path::PathBuf>,
    last_save: Option<Instant>,
    update_error: Option<String>,
    boot_healthy: bool,
}

fn save_game(game: &game::Game, path: Option<&std::path::Path>) {
    let Some(path) = path else { return; };
    let result = (|| -> std::io::Result<()> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let temp = path.with_extension("tmp");
        std::fs::write(&temp, game.save_text())?;
        std::fs::rename(temp, path)?;
        Ok(())
    })();
    if let Err(error) = result { log::warn!("Cannot save StarCity Builder: {error}"); }
}
impl App {
    fn with_save_path(path: std::path::PathBuf) -> Self {
        let game = std::fs::read_to_string(&path).ok().and_then(|text| game::Game::from_save(&text)).unwrap_or_default();
        Self { game, save_path: Some(path), ..Self::default() }
    }
}

fn amount(n: f64) -> String {
    if n >= 1_000_000_000.0 { format!("{:.1}B", n / 1_000_000_000.0) }
    else if n >= 1_000_000.0 { format!("{:.1}M", n / 1_000_000.0) }
    else if n >= 10_000.0 { format!("{:.1}K", n / 1_000.0) }
    else { format!("{:.0}", n.floor()) }
}


fn nav_icon(p: &egui::Painter, center: egui::Pos2, index: usize, color: egui::Color32) {
    use egui::{vec2, Rect, Stroke};
    let s = Stroke::new(2.0, color);
    match index {
        0 => {
            p.line_segment([center + vec2(-11.0, -1.0), center + vec2(0.0, -11.0)], s);
            p.line_segment([center + vec2(0.0, -11.0), center + vec2(11.0, -1.0)], s);
            p.rect_stroke(Rect::from_center_size(center + vec2(0.0, 4.0), vec2(15.0, 14.0)), 2.0, s);
        }
        1 => {
            let points = [(0.0,-11.0), (5.0,-9.0), (9.0,-2.0), (9.0,5.0), (5.0,10.0),
                (-5.0,10.0), (-9.0,5.0), (-9.0,-2.0), (-5.0,-9.0), (0.0,-11.0)];
            p.add(egui::Shape::line(points.into_iter().map(|(x, y)| center + vec2(x, y)).collect(), s));
        }
        2 => {
            p.line_segment([center + vec2(0.0, 11.0), center + vec2(0.0, -10.0)], s);
            p.line_segment([center + vec2(-8.0, -2.0), center + vec2(0.0, -10.0)], s);
            p.line_segment([center + vec2(8.0, -2.0), center + vec2(0.0, -10.0)], s);
        }
        3 => {
            for (x, h) in [(-8.0, 9.0), (0.0, 16.0), (8.0, 23.0)] {
                p.rect_filled(Rect::from_min_size(center + vec2(x - 2.0, 11.0 - h), vec2(4.0, h)), 1.0, color);
            }
        }
        5 => {
            p.circle_filled(center+vec2(0.0,5.0),6.0,color);
            for v in [vec2(-9.0,-3.0),vec2(0.0,-8.0),vec2(9.0,-3.0)] {p.circle_filled(center+v,3.5,color);}
        }
        _ => {
            p.circle_stroke(center, 10.0, s);
            p.line_segment([center + vec2(10.0, -10.0), center + vec2(10.0, 0.0)], s);
            p.line_segment([center + vec2(10.0, 0.0), center + vec2(0.0, 0.0)], s);
        }
    }
}

fn theme_color(dark: bool, light: [u8; 3], night: [u8; 3]) -> egui::Color32 {
    let [r, g, b] = if dark { night } else { light };
    egui::Color32::from_rgb(r, g, b)
}

fn paint_aero_gradient(painter: &egui::Painter, rect: egui::Rect, colors: [egui::Color32; 4]) {
    use egui::{Mesh, Shape, pos2};
    let mut mesh = Mesh::default();
    for (position, color) in [0.0, 0.46, 0.48, 1.0].into_iter().zip(colors) {
        let y = rect.top() + rect.height() * position;
        mesh.colored_vertex(pos2(rect.left(), y), color);
        mesh.colored_vertex(pos2(rect.right(), y), color);
    }
    for left in [0, 2, 4] {
        mesh.add_triangle(left, left + 1, left + 3);
        mesh.add_triangle(left, left + 3, left + 2);
    }
    painter.add(Shape::mesh(mesh));
}

fn paint_white_aero(painter: &egui::Painter, rect: egui::Rect) {
    use egui::Color32 as C;
    paint_aero_gradient(painter, rect, [C::WHITE, C::from_rgb(247,250,252), C::from_rgb(234,240,244), C::from_rgb(249,251,252)]);
}

fn paint_dark_glass(painter: &egui::Painter, rect: egui::Rect, radius: f32, tint: egui::Color32) {
    use egui::{Color32 as C, Stroke, pos2};
    painter.rect_filled(rect, radius, tint);
    if radius == 0.0 {
        paint_aero_gradient(painter, rect, [
            C::from_rgba_unmultiplied(158,184,255,65), C::from_white_alpha(14),
            C::from_black_alpha(30), C::from_rgba_unmultiplied(93,131,245,28),
        ]);
        return;
    }
    let inset = rect.shrink(1.5);
    let shine = egui::Rect::from_min_max(inset.min, pos2(inset.right(), inset.top() + inset.height() * 0.48));
    painter.rect_filled(shine, egui::Rounding { nw: radius - 1.5, ne: radius - 1.5, sw: 2.0, se: 2.0 }, C::from_white_alpha(23));
    painter.rect_stroke(rect, radius, Stroke::new(1.0_f32, C::from_rgba_unmultiplied(177,196,255,112)));
}

fn apply_theme(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    visuals.panel_fill = theme_color(dark, [246,238,211], [24,23,42]);
    visuals.window_fill = theme_color(dark, [250,244,222], [32,29,53]);
    visuals.selection.bg_fill = theme_color(dark, [93,161,71], [76,112,27]);
    visuals.override_text_color = Some(theme_color(dark, [39,54,41], [235,244,255]));
    if dark {
        use egui::{Color32 as C, Stroke};
        visuals.panel_fill = C::from_rgba_unmultiplied(15,30,58,240);
        visuals.window_fill = C::from_rgba_unmultiplied(19,35,65,247);
        visuals.window_stroke = Stroke::new(1.0_f32, C::from_rgb(96,119,172));
        visuals.extreme_bg_color = C::from_rgb(13,19,39);
        visuals.widgets.inactive.weak_bg_fill = C::from_rgb(31,53,95);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, C::from_rgb(88,111,159));
        visuals.widgets.hovered.weak_bg_fill = C::from_rgb(43,73,134);
        visuals.widgets.active.weak_bg_fill = C::from_rgb(80,118,29);
    }
    ctx.set_visuals(visuals);
}

const FIELD_PAGES: [(game::Tab, &str); 3] = [
    (game::Tab::Development, "Развитие"),
    (game::Tab::Home, "Город"),
    (game::Tab::Capsules, "Яйца"),
];

fn board_swipe(
    ctx: &egui::Context, field: egui::Rect, controls: egui::Rect, current: game::Tab,
    start: &mut Option<(egui::TouchId, egui::Pos2)>,
) -> Option<game::Tab> {
    ctx.input(|input| {
        for event in &input.events {
            let egui::Event::Touch { id, phase, pos, .. } = event else { continue; };
            match phase {
                egui::TouchPhase::Start if field.contains(*pos) && !controls.contains(*pos) => {
                    *start = Some((*id, *pos));
                }
                egui::TouchPhase::End | egui::TouchPhase::Cancel if start.is_some_and(|(touch, _)| touch == *id) => {
                    let (_, origin) = start.take().unwrap();
                    let delta = *pos - origin;
                    if *phase == egui::TouchPhase::End && delta.x.abs() >= 55.0 && delta.x.abs() >= 1.65 * delta.y.abs() {
                        if current == game::Tab::Home
                            && (delta.x > 0.0) != (origin.x <= field.center().x)
                        { return None; }
                        let index = FIELD_PAGES.iter().position(|(tab, _)| *tab == current)?;
                        let next = index.checked_add_signed(if delta.x < 0.0 { 1 } else { -1 })?;
                        return FIELD_PAGES.get(next).map(|(tab, _)| *tab);
                    }
                }
                _ => {}
            }
        }
        None
    })
}

fn ui(ctx:&egui::Context,game:&mut game::Game,update_error:Option<&str>)->bool {
    use egui::{Color32 as C,RichText as T,Rect,vec2};
    use gui_state::{GuiState,Panel,Modal};
    if ctx.style().visuals.dark_mode != game.dark_mode || ctx.style().visuals.override_text_color.is_none() {
        apply_theme(ctx, game.dark_mode);
    }
    let mut state=GuiState::load(ctx);
    match game.tab {
        game::Tab::Statistics=>{state.profile(true);game.tab=game::Tab::Home;}
        game::Tab::Upgrades=>{state.modal(Modal::Upgrades);game.tab=game::Tab::Home;}
        _=>{}
    }
    if game.hatch.is_some() {
        game.interface_open=true;
        pet_ui::overlays(ctx,game);
        return false;
    }
    sidebar_ui::draw(ctx,game,&mut state);
    game.pets_open=state.panel==Panel::Pets;game.interface_open=state.busy();
    let workspace=ctx.available_rect();
    let rendered_tab = game.tab;
    let blocked=game.paused||game.hatch.is_some()||game.interface_open;
    let drawer = if matches!(rendered_tab, game::Tab::Development | game::Tab::Capsules) {
        let width = (workspace.width() * 0.94).min(660.0);
        let x = if rendered_tab == game::Tab::Development { workspace.left() } else { workspace.right() - width };
        Some(Rect::from_min_size(egui::pos2(x, workspace.top()), vec2(width, workspace.height())))
    } else { None };
    let canvas_fill = theme_color(game.dark_mode, [166,207,224], [10,20,39]);
    egui::CentralPanel::default().frame(egui::Frame::none().fill(canvas_fill)).show(ctx, |ui| {
        ui.add_enabled_ui(!blocked && drawer.is_none(), |ui| { board_ui::show(ui, game); });
        let Some(drawer) = drawer else { return; };
        let scrim = if rendered_tab == game::Tab::Development {
            Rect::from_min_max(drawer.right_top(), workspace.max)
        } else {
            Rect::from_min_max(workspace.min, drawer.left_bottom())
        };
        ui.painter().rect_filled(scrim, 0.0, C::from_black_alpha(95));
        if !blocked && ui.interact(scrim, egui::Id::new("field_drawer_scrim"), egui::Sense::click()).clicked() {
            game.tab = game::Tab::Home;
            return;
        }
        let fill = if rendered_tab == game::Tab::Capsules { canvas_fill } else { ctx.style().visuals.window_fill };
        ui.painter().rect_filled(drawer, 0.0, fill);
        ui.painter().rect_stroke(drawer, 0.0, ctx.style().visuals.window_stroke);
        let inner = if rendered_tab == game::Tab::Capsules { drawer.shrink(8.0) } else { drawer };
        ui.allocate_ui_at_rect(inner, |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(inner));
            ui.set_width(inner.width());
            ui.add_enabled_ui(!blocked, |ui| {
                if rendered_tab == game::Tab::Capsules { pet_ui::shop(ui, game); }
                else {
                    panels_ui::paint_background(ui);
                    ui.heading("Развитие города");
                    ui.label(format!("Постоянный бонус дохода ×{}", amount(game.rebirth_bonus())));
                    ui.label("Новый город сбросит баланс, постройки и улучшения. Питомцы и уровень сохранятся.");
                    let label = format!("Основать новый город · {}", amount(game.rebirth_price()));
                    if ui.add_enabled(game.coins >= game.rebirth_price(), egui::Button::new(label)).clicked() {
                        game.rebirth();
                    }
                }
            });
        });
    });
    panels_ui::draw(ctx,workspace,game,&mut state);
    inventory_ui::draw(ctx,workspace,game,&mut state);
    if FIELD_PAGES.iter().any(|(tab, _)| *tab == game.tab) && !blocked {
        let previous = game.tab;
        let controls = pet_ui::board_controls(ctx, drawer.unwrap_or(workspace), game);
        if game.tab == game::Tab::Home { pet_ui::boost_status(ctx, workspace, controls, game); }
        if !state.busy() && game.tab == previous && game.tab == rendered_tab {
            let mut swipe_area = workspace;
            if game.tab == game::Tab::Capsules {
                swipe_area.min.y = (swipe_area.top() + 55.0).min(swipe_area.bottom());
            }
            if let Some(tab) = board_swipe(ctx, swipe_area, controls, previous, &mut state.board_swipe) {
                game.tab = tab;
            }
        } else {
            state.board_swipe = None;
        }
    } else {
        state.board_swipe = None;
    }
    if game.tab != game::Tab::Capsules {
        ctx.data_mut(|data| data.remove::<f64>(egui::Id::new("egg_opened")));
    }
    let mut update_requested=false;
    if game.paused {
        panels_ui::centered_modal(ctx, workspace, "Пауза", vec2(350.0, 250.0), |ui| {
            if panels_ui::heading(ui, "Пауза") { game.paused = false; return; }
            ui.vertical_centered(|ui| {
                ui.label("Доход и строительство приостановлены");
                let build=option_env!("GITHUB_SHA").unwrap_or("development");ui.label(format!("StarCity Builder {} · {}",env!("CARGO_PKG_VERSION"),&build[..build.len().min(8)]));
                if ui.add_sized(vec2(ui.available_width(),44.0),egui::Button::new("Обновить игру (OTA)")).clicked() {update_requested=true;}
                if let Some(error)=update_error {ui.colored_label(C::LIGHT_RED,error);}
                if ui.add_sized(vec2(ui.available_width(),44.0),egui::Button::new(T::new("Продолжить").strong())).clicked() {game.paused=false;}
            });
        });
    }
    game.pets_open=state.panel==Panel::Pets;game.interface_open=state.busy();state.store(ctx);
    if !game.paused && state.modal == Modal::None { panels_ui::clear_modal_animation(ctx); }
    update_requested
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() { return; }
        let window = self.window.get_or_insert_with(|| Arc::new(event_loop.create_window(
            Window::default_attributes().with_title("StarCity Builder").with_inner_size(winit::dpi::LogicalSize::new(420.0, 860.0))
        ).expect("window"))).clone();
        let gpu = GpuContext::new(window.clone(), true, false).expect("WolfCraft GPU");
        let egui = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("ButtonSans".into(), egui::FontData::from_static(include_bytes!("../assets/fonts/ButtonSans-Regular.ttf")).into());
        fonts.families.get_mut(&egui::FontFamily::Proportional).expect("proportional font family").insert(0, "ButtonSans".into());
        egui.set_fonts(fonts);
        apply_theme(&egui, self.game.dark_mode);
        let state = egui_winit::State::new(egui.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, None);
        let renderer = egui_wgpu::Renderer::new(&gpu.device, gpu.config.format, None, 1, false);
        self.gfx = Some(Graphics { gpu, egui, state, renderer });
        self.last_tick = Some(Instant::now());
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) { save_game(&self.game, self.save_path.as_deref()); event_loop.exit(); return; }
        let Some(g) = self.gfx.as_mut() else { return };
        let response = g.state.on_window_event(&g.gpu.window, &event);
        match event {
            WindowEvent::Resized(size) => g.gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_tick.replace(now).unwrap_or(now)).as_secs_f64().min(0.25);
                let before = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.upgrades, self.game.rebirths, self.game.building_count(), self.game.dark_mode);
                self.game.tick(dt);
                let input = g.state.take_egui_input(&g.gpu.window);
                let mut update_requested = false;
                let update_error = self.update_error.clone();
                let output = g.egui.run(input, |ctx| { update_requested = ui(ctx, &mut self.game, update_error.as_deref()); });
                if update_requested {
                    save_game(&self.game, self.save_path.as_deref());
                    self.update_error = updates::open().err();
                }
                let after = (self.game.pets.len(), self.game.equipped_pets.clone(), self.game.upgrades, self.game.rebirths, self.game.building_count(), self.game.dark_mode);
                if before != after || self.last_save.map(|t| now.duration_since(t).as_secs() >= 5).unwrap_or(true) {
                    save_game(&self.game, self.save_path.as_deref());
                    self.last_save = Some(now);
                }
                g.state.handle_platform_output(&g.gpu.window, output.platform_output);
                let size = g.gpu.window.inner_size();
                if size.width == 0 || size.height == 0 { return; }
                let pixels_per_point = g.egui.pixels_per_point();
                let shapes = g.egui.tessellate(output.shapes, pixels_per_point);
                let descriptor = egui_wgpu::ScreenDescriptor { size_in_pixels: [size.width, size.height], pixels_per_point };
                for (id, delta) in &output.textures_delta.set {
                    g.renderer.update_texture(&g.gpu.device, &g.gpu.queue, *id, delta);
                }
                let Ok(frame) = g.gpu.surface.as_ref().expect("active surface").get_current_texture() else { g.gpu.resize(size.width, size.height); return; };
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = g.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("starcity-builder") });
                g.renderer.update_buffers(&g.gpu.device, &g.gpu.queue, &mut encoder, &shapes, &descriptor);
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("starcity-builder-ui"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view, resolve_target: None,
                            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.25, b: 0.45, a: 1.0 }), store: wgpu::StoreOp::Store },
                        })],
                        depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
                    });
                    g.renderer.render(&mut pass.forget_lifetime(), &shapes, &descriptor);
                }
                g.gpu.queue.submit(Some(encoder.finish()));
                frame.present();
                if !self.boot_healthy { updates::mark_ready(); self.boot_healthy = true; }
                for id in &output.textures_delta.free { g.renderer.free_texture(id); }
                g.gpu.window.request_redraw();
            }
            _ => { if response.repaint { g.gpu.window.request_redraw(); } }
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) { save_game(&self.game, self.save_path.as_deref()); self.gfx = None; self.last_tick = None; }
}

pub fn run_desktop() {
    let event_loop = EventLoop::new().expect("event loop");
    let root = std::env::var_os("XDG_DATA_HOME").map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| std::path::PathBuf::from(p).join(".local/share")))
        .unwrap_or_else(std::env::temp_dir);
    let mut app = App::with_save_path(root.join("starcity-builder/save.txt"));
    event_loop.run_app(&mut app).expect("application loop");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::{event_loop::EventLoopBuilder, platform::android::EventLoopBuilderExtAndroid};
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    updates::configure(android_app.clone());
    let save_path = android_app.internal_data_path().unwrap_or_else(std::env::temp_dir).join("starcity-builder-save.txt");
    let event_loop = EventLoopBuilder::default().with_android_app(android_app).build().expect("android event loop");
    let mut app = App::with_save_path(save_path);
    event_loop.run_app(&mut app).expect("application loop");
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn portrait_playfield_shows_city_building_tools() {
        let ctx = egui::Context::default();
        let mut game = game::Game::default();
        let out = ctx.run(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(360.0, 740.0))),
            ..Default::default()
        }, |ctx| { ui(ctx, &mut game, None); });
        assert!(out.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text)
            if text.galley.job.text == "Дом · 120")));
    }
}
