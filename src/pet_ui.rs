use egui::{Color32 as C, RichText as T, vec2, Rect, Align2, Sense, Stroke, pos2};
use crate::{game::{Game,Capsule},pet_render,amount};
const EGG_COUNTS: [usize; 4] = [1, 3, 5, 10];

fn egg_card(ui: &mut egui::Ui, game: &mut Game, egg: Capsule) {
    let width = ui.available_width();
    let dark = ui.visuals().dark_mode;
    ui.push_id(egg == Capsule::Neon, |ui| {
        let quantity_id = ui.id().with("quantity");
        let expanded_id = ui.id().with("expanded");
        let luck_id = ui.id().with("luck");
        let mut quantity = ui.ctx().data_mut(|data| data.get_temp::<usize>(quantity_id)).unwrap_or(0);
        let mut luck = ui.ctx().data_mut(|data| data.get_temp::<u8>(luck_id)).unwrap_or(0);
        let mut expanded = ui.ctx().data_mut(|data| data.get_temp::<bool>(expanded_id)).unwrap_or(false);
        egui::Frame::none().fill(crate::theme_color(dark, [249,241,214], [27,49,91]))
            .stroke(Stroke::new(1.0_f32, crate::theme_color(dark, [201,190,151], [81,103,156])))
            .rounding(10.0).inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
            ui.set_width((width - 16.0).max(0.0));
            let header = Rect::from_min_size(ui.available_rect_before_wrap().min, vec2(ui.available_width(), 80.0));
            let preview = Rect::from_min_size(header.min, vec2((header.width() * 0.34).min(78.0), 76.0));
            let info = Rect::from_min_max(pos2(preview.right() + 7.0, header.top() + 5.0), header.max);
            let details = ui.interact(header, ui.id().with("details"), Sense::click());
            if details.on_hover_text("Содержимое яйца").clicked() { expanded = !expanded; }
            pet_render::egg_preview(ui.painter(), preview, egg == Capsule::Neon, game.played_seconds as f32, 0.0);
            ui.allocate_ui_at_rect(info, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(info));
                ui.label(T::new(egg.name()).size(14.0).strong());
                ui.label(T::new(format!("{} за яйцо", amount(egg.price_with_luck(luck)))).size(11.0));
                ui.label(T::new(if expanded { "Свернуть питомцев ‹" } else { "Посмотреть питомцев ›" }).size(10.0));
            });
            ui.allocate_rect(header, Sense::hover());
            ui.horizontal(|ui| {
                ui.label(T::new(format!("Кол-во ×{}", EGG_COUNTS[quantity])).size(11.0));
                ui.spacing_mut().slider_width = (ui.available_width() - 4.0).clamp(34.0, 100.0);
                ui.add(egui::Slider::new(&mut quantity, 0..=EGG_COUNTS.len() - 1).show_value(false))
                    .on_hover_text("Количество яиц");
            });
            ui.horizontal(|ui| {
                ui.label(T::new(format!("Удача {luck}/5")).size(11.0));
                ui.spacing_mut().slider_width = (ui.available_width() - 4.0).clamp(34.0, 100.0);
                ui.add(egui::Slider::new(&mut luck, 0..=5).show_value(false))
                    .on_hover_text("Повышает шанс редких питомцев и цену яйца");
            });
            let count = EGG_COUNTS[quantity];
            let cost = egg.price_with_luck(luck) * count as f64;
            let label = format!("Купить ×{count} · {}", amount(cost));
            if ui.add_enabled(game.coins >= cost && count <= 1000usize.saturating_sub(game.pets.len()),
                egui::Button::new(T::new(label).size(11.0)).fill(ui.visuals().selection.bg_fill)
                    .min_size(vec2(ui.available_width(), 31.0))).clicked()
            {
                game.buy_eggs(egg, count, luck);
            }
            if expanded {
                ui.separator();
                for pair in egg.drops_with_luck(luck).chunks(2) {
                    ui.columns(2, |columns| {
                        for (column, &(pet, chance)) in columns.iter_mut().zip(pair) {
                            pet_slot(column, pet, &format!("{chance}%"), game.played_seconds as f32);
                        }
                    });
                    ui.add_space(4.0);
                }
            }
        });
        ui.ctx().data_mut(|data| {
            data.insert_temp(quantity_id, quantity);
            data.insert_temp(luck_id, luck);
            data.insert_temp(expanded_id, expanded);
        });
    });
}

fn pet_slot(ui: &mut egui::Ui, pet: crate::game::PetKind, detail: &str, time: f32) {
    let width = ui.available_width();
    let dark = ui.visuals().dark_mode;
    egui::Frame::none().fill(crate::theme_color(dark, [230,235,207], [27,47,86]))
        .stroke(Stroke::new(1.0_f32, crate::theme_color(dark, [153,173,117], [88,123,181])))
        .rounding(5.0).inner_margin(egui::Margin::same(3.0)).show(ui, |ui| {
        ui.set_width((width - 6.0).max(0.0));
        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 53.0), Sense::hover());
            pet_render::pet(ui.painter(), rect, pet, time);
            ui.label(T::new(pet.name()).size(10.0));
            ui.label(T::new(detail).size(10.0));
        });
    });
}

pub fn shop(ui: &mut egui::Ui, game: &mut Game) {
    let time = ui.input(|input| input.time);
    let started = ui.ctx().data_mut(|data| *data.get_temp_mut_or_insert_with(egui::Id::new("egg_opened"), || time));
    let t = ((time - started) / 0.16).clamp(0.0, 1.0) as f32;
    ui.multiply_opacity(0.25 + 0.75 * t * (2.0 - t));
    if t < 1.0 { ui.ctx().request_repaint(); }
    let category_id = egui::Id::new("egg_category");
    let search_id = egui::Id::new("egg_search");
    let mut category = ui.ctx().data_mut(|data| data.get_temp::<usize>(category_id)).unwrap_or(0);
    let mut search = ui.ctx().data_mut(|data| data.get_temp::<String>(search_id)).unwrap_or_default();
    let mut close = false;
    ui.horizontal(|ui| {
        let (icon, _) = ui.allocate_exact_size(vec2(24.0, 32.0), Sense::hover());
        crate::nav_icon(ui.painter(), icon.center(), 1, ui.visuals().text_color());
        ui.label(T::new("Яйца").size(19.0).strong());
        let (icon, _) = ui.allocate_exact_size(vec2(17.0, 30.0), Sense::hover());
        let center = icon.center() - vec2(1.0, 2.0);
        let stroke = Stroke::new(1.5_f32, ui.visuals().text_color());
        ui.painter().circle_stroke(center, 4.5, stroke);
        ui.painter().line_segment([center + vec2(3.5,3.5), center + vec2(7.0,7.0)], stroke);
        let width = (ui.available_width() - 40.0).max(30.0);
        ui.add(egui::TextEdit::singleline(&mut search).hint_text("Поиск").desired_width(width));
        close = ui.add_sized(vec2(30.0, 30.0), egui::Button::new("×")).on_hover_text("Закрыть").clicked();
    });
    ui.ctx().data_mut(|data| data.insert_temp(search_id, search.clone()));
    if close { game.tab = crate::game::Tab::Home; return; }
    ui.separator();

    let body = ui.available_rect_before_wrap();
    let sidebar_width = (body.width() * 0.27).clamp(75.0, 145.0);
    let sidebar = Rect::from_min_size(body.min, vec2(sidebar_width, body.height()));
    let content = Rect::from_min_max(pos2(sidebar.right() + 7.0, body.top()), body.max);
    ui.allocate_ui_at_rect(sidebar, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(sidebar));
        ui.set_width(sidebar_width);
        egui::ScrollArea::vertical().id_salt("egg_categories").max_height(body.height()).show(ui, |ui| {
            for (index, title) in ["Все", "Обычные", "Редкие", "Эпические", "Легендарные"].into_iter().enumerate() {
                if ui.add_sized(vec2(sidebar_width, 34.0), egui::Button::new(T::new(title).size(11.0)).selected(category == index)).clicked() {
                    category = index;
                }
            }
        });
    });
    ui.ctx().data_mut(|data| data.insert_temp(category_id, category));
    ui.allocate_ui_at_rect(content, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(content));
        ui.set_width(content.width());
        let query = search.trim().to_lowercase();
        let eggs: Vec<_> = [Capsule::Meadow, Capsule::Neon].into_iter().filter(|egg| {
            let has_rarity = category == 0 || egg.drops().iter().any(|(pet, _)| usize::from(pet.evolution_tier().min(4)) == category);
            let matches = egg.name().to_lowercase().contains(&query) || egg.drops().iter().any(|(pet, _)| pet.name().to_lowercase().contains(&query));
            has_rarity && matches
        }).collect();
        egui::ScrollArea::vertical().id_salt("egg_cards").max_height((body.height() - 56.0).max(0.0)).show(ui, |ui| {
            if eggs.is_empty() { ui.label("Яйца не найдены"); }
            for egg in eggs {
                egg_card(ui, game, egg);
                ui.add_space(6.0);
            }
            if game.pets.len() >= 1000 { ui.label("Коллекция заполнена"); }
        });
    });
}

/// A dedicated scene: no sidebar, game field, windows or dialog background.
pub fn overlays(ctx:&egui::Context,game:&mut Game) {
    let Some(hatch)=game.hatch.clone() else {return;};
    let elapsed=hatch.elapsed as f32;let reveal=elapsed>=3.0;
    let Some(&pet)=game.pets.get(hatch.inventory_index) else {return;};
    egui::CentralPanel::default().frame(egui::Frame::none().fill(crate::theme_color(game.dark_mode, [166,207,224], [10,20,39]))).show(ctx,|ui| {
        let width=ui.available_width();let height=ui.available_height();
        if reveal && hatch.count > 1 {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(format!("Питомцы из яиц ×{}", hatch.count));
            });
            let results = &game.pets[hatch.inventory_index..hatch.inventory_index + hatch.count];
            egui::ScrollArea::vertical().max_height((height - 155.0).max(0.0)).show(ui, |ui| {
                for row in results.chunks(3) {
                    ui.columns(3, |columns| {
                        for (column, &pet) in columns.iter_mut().zip(row) {
                            pet_slot(column, pet, pet.rarity(), elapsed);
                        }
                    });
                    ui.add_space(5.0);
                }
            });
            ui.vertical_centered(|ui| {
                let controls = width.min(360.0) - 24.0;
                if ui.add_sized(vec2(controls,38.0), egui::Button::new("Надеть лучшего")).clicked() { game.finish_hatch(true); }
                if ui.add_sized(vec2(controls,34.0), egui::Button::new("В коллекцию")).clicked() { game.finish_hatch(false); }
            });
            return;
        }
        let model_size=(height-160.0).min(width*0.44).clamp(72.0,270.0);
        let content_height=model_size+if reveal {156.0} else {128.0};
        ui.add_space(((height-content_height)*0.5).max(0.0));
        ui.vertical_centered(|ui| {
            ui.label(T::new(if reveal {"НОВЫЙ ПИТОМЕЦ!"} else {"ОТКРЫТИЕ ЯЙЦА"}).size(19.0).strong());
            let title = if reveal { pet.name().to_owned() } else if hatch.count > 1 {
                format!("{} ×{}", hatch.capsule.name(), hatch.count)
            } else { hatch.capsule.name().to_owned() };
            ui.label(T::new(title).size(23.0).strong());
            let (r,_)=ui.allocate_exact_size(vec2(model_size,model_size),Sense::hover());
            if reveal {pet_render::pet(ui.painter(),r,pet,elapsed);} else {
                pet_render::egg_preview(ui.painter(),r,hatch.capsule==Capsule::Neon,elapsed,if elapsed<0.7 {0.0} else {((elapsed-0.7)*7.0).min(10.0)});
            }
            let controls=width.min(360.0)-24.0;
            if reveal {
                ui.label(T::new(format!("{} · доход ×{:.1}",pet.rarity(),pet.bonus())).size(16.0));
                ui.add_space(8.0);
                if ui.add_sized(vec2(controls,38.0),egui::Button::new(T::new("Применить").strong().color(C::WHITE)).fill(ui.visuals().selection.bg_fill).rounding(19.0)).clicked() {game.finish_hatch(true);}
                if ui.add_sized(vec2(controls,34.0),egui::Button::new("В коллекцию").rounding(17.0)).clicked() {game.finish_hatch(false);}
            } else {
                ui.label(T::new(if elapsed<0.7 {"Приготовься…"} else if elapsed<2.4 {"ТРЯСЁМ!"} else {"Кто же внутри?"}).size(20.0).strong());
                ui.add(egui::ProgressBar::new((elapsed/3.0).clamp(0.0,1.0)).desired_width(controls).fill(ui.visuals().selection.bg_fill).show_percentage());
            }
        });
    });
    ctx.request_repaint();
}

pub fn board_controls(ctx:&egui::Context,workspace:Rect,game:&mut Game) -> Rect {
    let width: f32 = 96.0;
    let size = vec2(width.min((workspace.width() - 16.0).max(0.0)), 50.0);
    egui::Area::new(egui::Id::new("canvas_page_controls")).order(egui::Order::Foreground)
        .fixed_pos(workspace.right_bottom() - size - vec2(8.0, 8.0)).show(ctx, |ui| {
        egui::Frame::none().fill(ctx.style().visuals.panel_fill).rounding(14.0).inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
            ui.set_width((size.x - 16.0).max(0.0));
            if ui.visuals().dark_mode {
                let rect = Rect::from_min_size(ui.cursor().min - vec2(7.0, 7.0), size - vec2(2.0, 2.0));
                crate::paint_dark_glass(ui.painter(), rect, 13.0, ui.visuals().panel_fill);
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                for (tab, label) in crate::FIELD_PAGES {
                    let active = game.tab == tab;
                    let (rect, response) = ui.allocate_exact_size(vec2(24.0, 34.0), Sense::click());
                    let ink = ui.visuals().text_color().gamma_multiply(if active { 1.0 } else { 0.42 });
                    ui.painter().circle_filled(rect.center(), if active { 4.0 } else { 3.0 }, ink);
                    if response.on_hover_text(label).clicked() { game.tab = tab; }
                }
            });
        });
    }).response.rect
}

pub fn boost_status(ctx: &egui::Context, workspace: Rect, controls: Rect, game: &Game) {
    if game.active_boosts.is_empty() { return; }
    let left = workspace.left() + 12.0;
    let right = controls.left() - 8.0;
    if right <= left { return; }
    let area = Rect::from_min_max(pos2(left, workspace.top()), pos2(right, workspace.bottom()));
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("boost_status")))
        .with_clip_rect(area);
    let active = game.active_boosts.iter().filter(|boost| boost.expires_at.is_finite() && boost.expires_at > game.played_seconds);
    for (row, boost) in active.enumerate() {
        let y = controls.center().y - 16.0 - row as f32 * 36.0;
        if y < area.top() { break; }
        let seconds = (boost.expires_at - game.played_seconds).ceil() as u64;
        let timer = format!("{}:{:02}", seconds / 60, seconds % 60);
        for (text, y, size) in [(boost.name.as_str(), y, 15.0), (timer.as_str(), y + 17.0, 12.0)] {
            let font = egui::FontId::proportional(size);
            let measured = painter.layout_no_wrap(text.to_owned(), font, C::WHITE);
            let size = size * (area.width() / measured.size().x.max(1.0)).min(1.0);
            let font = egui::FontId::proportional(size);
            let position = pos2(left, y);
            painter.text(position + vec2(1.0, 1.5), Align2::LEFT_TOP, text, font.clone(), C::from_black_alpha(235));
            painter.text(position, Align2::LEFT_TOP, text, font, C::WHITE);
        }
    }
}
