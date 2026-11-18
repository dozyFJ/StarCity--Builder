use egui::{Align2, Color32 as C, Rect, RichText as T, Sense, Stroke, pos2, vec2};
use crate::{game::{Game, Tab}, gui_state::{GuiState, Panel, PetAction}, panels_ui, pet_render};

const GREEN: C = C::from_rgb(93, 161, 71);

pub fn draw(ctx: &egui::Context, workspace: Rect, game: &mut Game, state: &mut GuiState) {
    if state.panel != Panel::Pets { return; }
    let sheet = Rect::from_min_max(workspace.left_top() + vec2(0.0, 8.0), workspace.right_bottom());
    let frame = panels_ui::frame(ctx).rounding(egui::Rounding { nw: 20.0, ne: 20.0, sw: 0.0, se: 0.0 });
    let content_size = sheet.size() - frame.total_margin().sum();
    egui::Window::new("pets_inventory").title_bar(false).fixed_pos(sheet.min).fixed_size(content_size)
        .resizable(false).collapsible(false).frame(frame).show(ctx, |ui| {
        panels_ui::paint_background(ui);
        if panels_ui::heading(ui, "Питомцы") {
            state.close();
            return;
        }
        if state.selected_pet.is_none_or(|i| i >= game.pets.len()) {
            state.selected_pet = (!game.pets.is_empty()).then_some(0);
        }
        let body = Rect::from_min_max(ui.available_rect_before_wrap().min,
            sheet.max - vec2(10.0, 10.0));
        let left_width = (body.width() * 0.36).min(228.0);
        let left = Rect::from_min_size(body.min, vec2(left_width, body.height()));
        let right = Rect::from_min_max(pos2(left.right() + 10.0, body.top()), body.max);
        let line = left.right() + 5.0;
        ui.painter().line_segment([pos2(line, body.top()), pos2(line, body.bottom())],
            ui.visuals().widgets.noninteractive.bg_stroke);

        ui.allocate_ui_at_rect(left, |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(left));
            ui.set_width(left.width());
            egui::ScrollArea::vertical().id_salt("pet_detail_scroll").max_height(left.height()).show(ui, |ui| {
                if let Some(index) = state.selected_pet {
                    details(ui, game, state, index);
                }
            });
        });

        let footer_top = (right.bottom() - 86.0).max(right.top());
        let footer_rect = Rect::from_min_max(pos2(right.left(), footer_top), right.max);
        let list = Rect::from_min_max(right.min, pos2(right.right(), footer_top - 5.0));
        ui.allocate_ui_at_rect(footer_rect, |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(footer_rect));
            ui.set_width(footer_rect.width());
            footer(ui, game, state);
        });

        ui.allocate_ui_at_rect(list, |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(list));
            ui.set_width(list.width());
            if state.pet_multi_delete {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Выбрано: {}", state.pet_marked.len()));
                    if ui.button("Отмена").clicked() {
                        state.pet_multi_delete = false;
                        state.pet_marked.clear();
                    }
                    if ui.add_enabled(!state.pet_marked.is_empty() && state.pending.is_none() && !game.paused,
                        egui::Button::new("Удалить выбранных")).clicked()
                    {
                        state.pending = Some(PetAction::DeleteMany);
                    }
                });
            }
            grid(ui, game, state, list.width());
        });
    });
    confirm(ctx, workspace, game, state);
}

fn details(ui: &mut egui::Ui, game: &mut Game, state: &mut GuiState, index: usize) {
    let pet = game.pets[index];
    let dark = ui.visuals().dark_mode;
    ui.vertical_centered(|ui| {
        ui.label(T::new(pet.name()).size(17.0).strong());
        let width = (ui.available_width() - 4.0).max(0.0);
        let (viewport, _) = ui.allocate_exact_size(vec2(width, width.clamp(65.0, 135.0)), Sense::hover());
        let painter = ui.painter().with_clip_rect(viewport);
        painter.rect_filled(viewport, 0.0, crate::theme_color(dark, [195,220,223], [38,49,79]));
        let floor = Rect::from_min_max(pos2(viewport.left(), viewport.top() + viewport.height() * 0.64), viewport.max);
        painter.rect_filled(floor, 0.0, crate::theme_color(dark, [154,177,123], [52,63,94]));
        painter.rect_filled(Rect::from_center_size(viewport.center() + vec2(0.0, viewport.height() * 0.29),
            vec2(viewport.width() * 0.48, 9.0)), 5.0, C::from_black_alpha(65));
        pet_render::pet(&painter, viewport.shrink(5.0), pet, game.played_seconds as f32);
        painter.rect_stroke(viewport, 0.0, Stroke::new(1.0_f32, crate::theme_color(dark, [103,122,87], [133,117,198])));
        ui.label(T::new(format!("{} · тир {}", pet.rarity(), pet.evolution_tier())).size(11.0));
        for label in [format!("Доход ×{:.1}", pet.bonus()), format!("Тир {}", pet.evolution_tier())] {
            egui::Frame::none().fill(ui.visuals().selection.bg_fill).rounding(6.0)
                .inner_margin(egui::Margin::symmetric(7.0, 4.0)).show(ui, |ui| {
                    ui.set_width((ui.available_width() - 2.0).max(0.0));
                    ui.vertical_centered(|ui| { ui.label(T::new(label).strong()); });
                });
        }
        ui.add_space(4.0);
        let xp = game.pet_xp(index);
        ui.add(egui::ProgressBar::new((xp % 100) as f32 / 100.0)
            .desired_width(ui.available_width()).text(format!("Ур. {} · {} / 100 XP", xp / 100 + 1, xp % 100)));
        ui.add_space(4.0);

        ui.add_enabled_ui(!game.paused && state.pending.is_none(), |ui| {
            let button = |label| egui::Button::new(T::new(label).size(11.0)).min_size(vec2(width, 27.0));
            let equipped = game.equipped_pets.contains(&index);
            if ui.add(button(if equipped { "Снять" } else { "Надеть" })).clicked() {
                if equipped { game.unequip_pet(index); } else { game.equip_pet(index); }
            }
            if ui.add_enabled(!state.pet_multi_delete, button("Удалить")).clicked() {
                state.pending = Some(PetAction::Delete(index));
            }
            if ui.add_enabled(!state.pet_multi_delete && game.craft_materials(index).is_some(), button("Крафт"))
                .on_hover_text("Нужны три одинаковых питомца").clicked()
            { state.pending = Some(PetAction::Craft(index)); }
            if ui.add(button("Надеть лучшего")).clicked() { game.equip_best_pet(); }
            if ui.add_enabled(!game.equipped_pets.is_empty(), button("Снять всех")).clicked() {
                game.unequip_all_pets();
            }
            let label = if state.pet_multi_delete { "Отменить выбор" } else { "Мультиудаление" };
            if ui.add(button(label).selected(state.pet_multi_delete)).clicked() {
                state.pet_multi_delete = !state.pet_multi_delete;
                state.pet_marked.clear();
            }
        });
    });
}

fn footer(ui: &mut egui::Ui, game: &Game, state: &mut GuiState) {
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label(T::new(format!("Надето {}/3", game.equipped_pets.len())).size(10.0).strong());
        ui.label(T::new(format!("Инвентарь {}/1000", game.pets.len())).size(10.0));
    });
    ui.horizontal_wrapped(|ui| {
        for (i, title) in ["Инвентарь", "Надеты", "Крафт"].into_iter().enumerate() {
            ui.selectable_value(&mut state.pet_tab, i, T::new(title).size(11.0));
        }
    });
    let rarities = ["Все редкости", "Обычные", "Редкие", "Эпические", "Легендарные"];
    egui::ComboBox::from_id_salt("pet_rarity_filter").width(ui.available_width())
        .selected_text(rarities[state.rarity]).show_ui(ui, |ui| {
            for (i, title) in rarities.into_iter().enumerate() {
                ui.selectable_value(&mut state.rarity, i, title);
            }
        });
}

fn grid(ui: &mut egui::Ui, game: &mut Game, state: &mut GuiState, available_width: f32) {
    let filtered: Vec<_> = game.pets.iter().enumerate().filter_map(|(index, &pet)| {
        let rank = usize::from(pet.evolution_tier().min(4));
        let matches = (state.rarity == 0 || state.rarity == rank)
            && (state.pet_tab != 1 || game.equipped_pets.contains(&index))
            && (state.pet_tab != 2 || Game::craft_result(pet).is_some());
        matches.then_some(index)
    }).collect();
    if filtered.is_empty() {
        ui.label(if game.pets.is_empty() { "Нет питомцев. Открой яйцо." } else { "В этой категории нет питомцев." });
        if game.pets.is_empty() && ui.button("К яйцам").clicked() { state.close(); game.tab = Tab::Capsules; }
        return;
    }
    let width = (available_width - 12.0).max(1.0);
    let columns = ((width / 76.0).floor() as usize).max(1);
    let tile = (width - (columns - 1) as f32 * 4.0) / columns as f32;
    egui::ScrollArea::vertical().id_salt(("pet_slots", state.pet_tab, state.rarity)).max_width(available_width).auto_shrink([false, false]).show_rows(ui, 94.0,
        filtered.len().div_ceil(columns), |ui, rows| {
        for row in rows {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for &index in filtered.iter().skip(row * columns).take(columns) {
                    let pet = game.pets[index];
                    let (rect, response) = ui.allocate_exact_size(vec2(tile, 94.0), Sense::click());
                    let p = ui.painter();
                    let marked = state.pet_marked.contains(&index);
                    let selected = if state.pet_multi_delete { marked } else { state.selected_pet == Some(index) };
                    let dark = ui.visuals().dark_mode;
                    p.rect_filled(rect, 5.0, crate::theme_color(dark, [230,235,207], [27,47,86]));
                    p.rect_stroke(rect, 5.0, Stroke::new(if selected { 2.0 } else { 1.0 },
                        if selected { C::from_rgb(221,166,52) } else { crate::theme_color(dark, [153,173,117], [88,123,181]) }));
                    pet_render::pet(p, Rect::from_min_max(rect.min + vec2(5.0,3.0), rect.max - vec2(5.0,30.0)), pet, game.played_seconds as f32);
                    let ink = ui.visuals().text_color();
                    let number = if marked { format!("✓ {}", index + 1) } else { (index + 1).to_string() };
                    p.text(rect.left_top() + vec2(5.0,5.0), Align2::LEFT_TOP, number, egui::FontId::proportional(10.0), ink);
                    if game.equipped_pets.contains(&index) { p.circle_filled(rect.right_top() + vec2(-8.0,8.0), 4.0, GREEN); }
                    p.text(rect.center_bottom() - vec2(0.0,21.0), Align2::CENTER_CENTER, pet.name(), egui::FontId::proportional(10.0), ink);
                    p.text(rect.center_bottom() - vec2(0.0,8.0), Align2::CENTER_CENTER, format!("×{:.1}", pet.bonus()), egui::FontId::proportional(9.0), ink);
                    if response.clicked() && state.pending.is_none() {
                        if state.pet_multi_delete {
                            if let Some(position) = state.pet_marked.iter().position(|&entry| entry == index) {
                                state.pet_marked.swap_remove(position);
                            } else { state.pet_marked.push(index); }
                        } else { state.selected_pet = Some(index); }
                    }
                }
            });
        }
    });
}

fn confirm(ctx: &egui::Context, workspace: Rect, game: &mut Game, state: &mut GuiState) {
    let Some(action) = state.pending else { return; };
    let message = match action {
        PetAction::Delete(i) => game.pets.get(i).map(|pet| format!("Удалить питомца «{}»?", pet.name())),
        PetAction::Craft(i) => game.pets.get(i).and_then(|pet| Game::craft_result(*pet)
            .map(|next| format!("Обменять 3 × {} на {} (×{:.1})?", pet.name(), next.name(), next.bonus()))),
        PetAction::DeleteMany => Some(format!("Удалить выбранных питомцев ({})?", state.pet_marked.len())),
    };
    let Some(message) = message else { state.pending = None; return; };
    let rect = Rect::from_center_size(workspace.center(), vec2((workspace.width() - 32.0).min(390.0), 128.0));
    egui::Window::new("pet_confirmation").title_bar(false).fixed_rect(rect)
        .resizable(false).collapsible(false).frame(panels_ui::frame(ctx)).show(ctx, |ui| {
        panels_ui::paint_background(ui);
        ui.heading(message);
        if matches!(action, PetAction::Delete(_) | PetAction::DeleteMany) { ui.label("Действие нельзя отменить."); }
        ui.horizontal(|ui| {
            if ui.button("Отмена").clicked() { state.pending = None; }
            if ui.add_enabled(!game.paused, egui::Button::new("Подтвердить")).clicked() {
                match action {
                    PetAction::Delete(index) => { if game.delete_pet(index) { state.selected_pet = None; } }
                    PetAction::Craft(index) => { state.selected_pet = game.craft_pet(index); }
                    PetAction::DeleteMany => {
                        if game.delete_pets(state.pet_marked.clone()) {
                            state.pet_marked.clear();
                            state.pet_multi_delete = false;
                            state.selected_pet = None;
                        }
                    }
                }
                state.pending = None;
            }
        });
    });
}
