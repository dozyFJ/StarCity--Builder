use egui::{RichText as T,Rect,vec2,pos2};
use crate::{game::Game,gui_state::{GuiState,Panel,Modal},amount};

pub fn frame(ctx: &egui::Context) -> egui::Frame {
    let style = ctx.style();
    let fill = if style.visuals.dark_mode { style.visuals.window_fill } else { egui::Color32::from_rgb(249,251,252) };
    egui::Frame::none().fill(fill).stroke(style.visuals.window_stroke).inner_margin(egui::Margin::same(10.0))
}

pub fn paint_background(ui: &egui::Ui) {
    if ui.visuals().dark_mode {
        crate::paint_dark_glass(ui.painter(), ui.max_rect(), 0.0, ui.visuals().window_fill);
    } else {
        crate::paint_white_aero(ui.painter(), ui.max_rect());
    }
}
pub fn centered_modal(
    ctx: &egui::Context, workspace: Rect, title: &'static str, size: egui::Vec2,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let time = ctx.input(|input| input.time);
    let key = egui::Id::new("centered_modal_animation");
    let started = ctx.data_mut(|data| {
        let entry = data.get_temp_mut_or_insert_with(key, || (title, time));
        if entry.0 != title { *entry = (title, time); }
        entry.1
    });
    let t = ((time - started) / 0.18).clamp(0.0, 1.0) as f32;
    let eased = t * (2.0 - t);
    if t < 1.0 { ctx.request_repaint(); }
    ctx.layer_painter(egui::LayerId::background()).rect_filled(workspace, 0.0,
        egui::Color32::from_black_alpha((115.0 * eased) as u8));
    let size = size.min(workspace.size() - vec2(24.0, 24.0)).max(vec2(40.0, 40.0));
    let bounds = Rect::from_center_size(workspace.center() + vec2(0.0, (1.0 - eased) * 10.0), size);
    let frame = frame(ctx).rounding(14.0);
    let content = bounds.size() - frame.total_margin().sum();
    egui::Window::new(title).title_bar(false).fixed_pos(bounds.min).fixed_size(content)
        .resizable(false).collapsible(false).frame(frame).show(ctx, |ui| {
            paint_background(ui);
            ui.multiply_opacity(0.55 + 0.45 * eased);
            add_contents(ui);
        });
}

pub fn clear_modal_animation(ctx: &egui::Context) {
    ctx.data_mut(|data| data.remove::<(&'static str, f64)>(egui::Id::new("centered_modal_animation")));
}

pub fn heading(ui:&mut egui::Ui,title:&str)->bool {
    let mut close=false;ui.horizontal(|ui| {ui.label(T::new(title).size(19.0).strong());ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {close=ui.button("X").clicked();});});ui.separator();close
}
fn card(ui:&mut egui::Ui,title:&str,value:&str) {
    egui::Frame::none().fill(crate::theme_color(ui.visuals().dark_mode, [231,235,207], [27,47,86])).inner_margin(egui::Margin::same(10.0)).show(ui,|ui| {
        ui.set_min_width((ui.available_width()-2.0).max(50.0));ui.label(T::new(title).size(11.0).color(crate::theme_color(ui.visuals().dark_mode, [92,109,77], [185,205,239])));ui.label(T::new(value).size(18.0).strong());
    });ui.add_space(5.0);
}
pub fn draw(ctx:&egui::Context,workspace:Rect,game:&mut Game,state:&mut GuiState) {
    if state.panel==Panel::Profile {
        let width = workspace.width().min(if workspace.width() < 600.0 { workspace.width() } else { 460.0 });
        let height = (workspace.height() - 8.0).clamp(0.0, 415.0);
        let menu = Rect::from_min_size(workspace.left_top(), vec2(width, height));
        let frame = frame(ctx).rounding(egui::Rounding { nw: 0.0, ne: 0.0, sw: 18.0, se: 18.0 });
        let content_size = menu.size() - frame.total_margin().sum();
        egui::Window::new("profile_panel").title_bar(false).fixed_pos(menu.min).fixed_size(content_size)
            .resizable(false).collapsible(false).frame(frame).show(ctx, |ui| {
            paint_background(ui);
            if heading(ui,"Мой профиль") { state.close(); return; }
            ui.horizontal(|ui| {
                let width = ((ui.available_width() - ui.spacing().item_spacing.x) * 0.5).max(0.0);
                for (stats, title) in [(false, "Профиль"), (true, "Статистика")] {
                    let selected = state.profile_stats == stats;
                    if ui.add_sized(vec2(width, 40.0), egui::Button::new(T::new(title).strong()).selected(selected)).clicked() {
                        state.profile_stats = stats;
                    }
                }
            });
            ui.add_space(6.0);
            egui::ScrollArea::vertical().id_salt("profile_cards").show(ui,|ui| {
                if !state.profile_stats {
                    card(ui,"Игрок",game.rank());card(ui,"Уровень",&game.level().to_string());
                    ui.add(egui::ProgressBar::new(game.level_progress()).desired_width(ui.available_width()).fill(ui.visuals().selection.bg_fill).text(format!("{} / 100 XP",game.city_xp%100)));
                    card(ui,"Деньги",&amount(game.coins));card(ui,"Новые города",&game.rebirths.to_string());
                    card(ui,"Команда питомцев",&format!("{} / 3 · ×{:.1}",game.equipped_pets.len(),game.pet_bonus()));
                } else {
                    for (title,value) in [("Постройки",game.building_count().to_string()),("Заработано денег",amount(game.total_earned)),("Доход / сек",amount(game.coins_per_second())),("Уровень",game.level().to_string()),("Новые города",game.rebirths.to_string()),("Питомцы",game.pets.len().to_string()),("Жители",game.population().to_string()),("В игре",format!("{} мин",(game.played_seconds/60.0) as u64))] {card(ui,title,&value);}
                }
            });
        });
    }
    if state.modal!=Modal::None {
        let modal = state.modal;
        let (title, size) = match modal {
            Modal::Upgrades => ("Улучшения", vec2(450.0, 470.0)),
            Modal::Settings => ("Настройки", vec2(380.0, 220.0)),
            Modal::Shop => ("Магазин", vec2(420.0, 305.0)),
            Modal::None => return,
        };
        centered_modal(ctx, workspace, title, size, |ui| {
            if heading(ui, title) { state.close(); return; }
            egui::ScrollArea::vertical().id_salt(("modal_cards",modal==Modal::Upgrades)).show(ui,|ui| {
                if modal==Modal::Settings {
                    ui.checkbox(&mut game.dark_mode, "Тёмная тема");
                    ui.label("Масштаб интерфейса");
                    ui.horizontal(|ui| {
                        for (label,scale) in [("Меньше",0.9),("Обычный",1.0),("Крупнее",1.1)] {if ui.button(label).clicked() {ctx.set_zoom_factor(scale);}}
                    });
                    if ui.button("Пауза и обновление игры").clicked() {state.close();game.paused=true;}
                } else if modal==Modal::Shop {
                    ui.label("Яйца за игровые деньги");
                    if ui.button("Перейти к яйцам").clicked() {state.close();game.tab=crate::game::Tab::Capsules;}
                    ui.separator();
                    for capsule in [crate::game::Capsule::Meadow,crate::game::Capsule::Neon] {
                        ui.label(T::new(capsule.name()).strong());
                        if ui.add_enabled(!game.paused&&game.coins>=capsule.price()&&game.pets.len()<1000,egui::Button::new(format!("Купить · {} денег",amount(capsule.price()))).fill(ui.visuals().selection.bg_fill)).clicked() {game.buy_capsule(capsule);state.close();}
                        ui.add_space(8.0);
                    }
                } else if modal==Modal::Upgrades {
                    for step in 0..4 {
                        let level=game.upgrades+step;
                        egui::Frame::none().fill(crate::theme_color(ui.visuals().dark_mode, [231,235,207], [27,47,86])).inner_margin(egui::Margin::same(12.0)).show(ui,|ui| {
                            ui.set_min_width((ui.available_width()-2.0).max(120.0));ui.label(T::new(format!("Доход · уровень {}",level+1)).strong());
                            ui.label(format!("Базовый доход {} / сек → {} / сек",amount(game.base_income+step as f64),amount(game.base_income+step as f64+1.0)));
                            let cost=50.0*2.0_f64.powi(level.min(30) as i32);
                            if step==0 {
                                if ui.add_enabled(!game.paused&&game.coins>=cost,egui::Button::new(format!("Улучшить · {} денег",amount(cost))).fill(ui.visuals().selection.bg_fill)).clicked() {game.upgrade();}
                                if game.coins<cost {ui.label(T::new(format!("Не хватает {} денег",amount(cost-game.coins))).size(11.0));}
                            } else {ui.add_enabled(false,egui::Button::new(format!("После предыдущего · {} денег",amount(cost))));}
                        });ui.add_space(8.0);
                    }
                }
            });
        });
    }
}
