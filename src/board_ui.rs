//! Building palette and a touch-friendly city map.
use egui::{Align2, Color32 as C, FontId, Rect, RichText, Sense, Shape, Stroke, pos2, vec2};
use crate::{amount, game::{Building, Game, CITY_SIDE}};

pub fn show(ui: &mut egui::Ui, game: &mut Game) {
    let dark = game.dark_mode;
    let grass = crate::theme_color(dark, [169,196,128], [28,59,68]);
    let occupied = crate::theme_color(dark, [181,202,145], [37,72,81]);

    ui.label(RichText::new("Город").size(19.0).strong());
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("Жители {}", game.population()));
        ui.label(format!("Доход {}/сек", amount(game.coins_per_second())));
        ui.label(format!("Занято {}/{}", game.building_count(), game.plots.len()));
    });
    ui.separator();
    egui::ScrollArea::horizontal().id_salt("city_tools").show(ui, |ui| {
        ui.horizontal(|ui| {
            for building in Building::ALL {
                let label = format!("{} · {}", building.name(), amount(building.cost()));
                ui.selectable_value(&mut game.selected_building, building, label);
            }
        });
    });
    ui.label(RichText::new("Выбери постройку и нажми на свободный участок").size(11.0));

    egui::ScrollArea::vertical().id_salt("city_map").max_height((ui.available_height() - 60.0).max(0.0)).show(ui, |ui| {
        let side = ((ui.available_width() - 22.0) / CITY_SIDE as f32).clamp(16.0, 64.0);
        let gap = 2.0;
        for row in 0..CITY_SIDE {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for column in 0..CITY_SIDE {
                    let index = row * CITY_SIDE + column;
                    let (rect, resp) = ui.allocate_exact_size(vec2(side, side), Sense::click());
                    let building = game.plots[index];
                    let p = ui.painter();
                    p.rect_filled(rect, 3.0, if building.is_some() { occupied } else { grass });
                    p.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, C::from_black_alpha(65)));
                    if let Some(building) = building {
                        draw_building(p, rect.shrink(3.0), building);
                        resp.on_hover_text(building.name());
                    } else {
                        if resp.hovered() {
                            p.rect_stroke(rect, 3.0, Stroke::new(2.0_f32, ui.visuals().selection.bg_fill));
                        }
                        if resp.clicked() { game.build(index, game.selected_building); }
                    }
                }
            });
        }
    });
}

fn draw_building(p: &egui::Painter, rect: Rect, building: Building) {
    let center = rect.center();
    match building {
        Building::Road => {
            p.rect_filled(rect, 2.0, C::from_rgb(58,69,82));
            let inset = rect.height() * 0.2;
            p.line_segment([pos2(center.x, rect.top() + inset), pos2(center.x, rect.bottom() - inset)],
                Stroke::new(2.0_f32, C::from_rgb(240,215,145)));
        }
        Building::Park => {
            p.rect_filled(rect, 3.0, C::from_rgb(90,158,90));
            for offset in [-0.20, 0.20] {
                let tree = center + vec2(rect.width() * offset, rect.height() * offset);
                p.circle_filled(tree + vec2(1.0, 2.0), rect.width() * 0.23, C::from_rgb(39,100,73));
                p.circle_filled(tree, rect.width() * 0.19, C::from_rgb(148,214,121));
            }
        }
        Building::House | Building::Shop => {
            let house = building == Building::House;
            let wall = if house { C::from_rgb(243,210,160) } else { C::from_rgb(188,215,240) };
            let roof = if house { C::from_rgb(198,84,69) } else { C::from_rgb(70,126,196) };
            let base = Rect::from_center_size(center + vec2(0.0, rect.height() * 0.12), rect.size() * 0.63);
            p.rect_filled(base.translate(vec2(2.0, 2.0)), 2.0, C::from_black_alpha(55));
            p.rect_filled(base, 2.0, wall);
            p.add(Shape::convex_polygon(vec![base.left_top(), pos2(center.x, rect.top()), base.right_top(),
                base.right_top() + vec2(0.0, 2.0), base.left_top() + vec2(0.0, 2.0)], roof, Stroke::NONE));
            p.rect_filled(Rect::from_center_size(base.center(), base.size() * 0.30), 1.0, roof);
            if rect.width() > 37.0 {
                p.text(base.center_bottom() - vec2(0.0, 2.0), Align2::CENTER_BOTTOM,
                    if house { "+3" } else { "+5" }, FontId::proportional(8.0), C::from_rgb(26,42,59));
            }
        }
    }
}
