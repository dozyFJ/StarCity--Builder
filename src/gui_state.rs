use egui::{Context,Id};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum Panel {#[default] None,Profile,Pets}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum Modal {#[default] None,Upgrades,Settings,Shop}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum PetAction {Delete(usize),Craft(usize),DeleteMany}
#[derive(Clone,Default)]
pub struct GuiState {
    pub panel:Panel,pub modal:Modal,pub profile_stats:bool,
    pub pet_tab:usize,pub rarity:usize,pub selected_pet:Option<usize>,pub pending:Option<PetAction>,
    pub pet_multi_delete: bool,
    pub pet_marked: Vec<usize>,
    pub sidebar_scroll:f32,
    pub board_swipe: Option<(egui::TouchId, egui::Pos2)>,
}
impl GuiState {
    pub fn load(ctx:&Context)->Self {ctx.data_mut(|d|d.get_temp::<Self>(Id::new("game_gui"))).unwrap_or_default()}
    pub fn store(&self,ctx:&Context) {ctx.data_mut(|d|d.insert_temp(Id::new("game_gui"),self.clone()));}
    pub fn busy(&self)->bool {self.panel!=Panel::None||self.modal!=Modal::None||self.pending.is_some()}
    pub fn close(&mut self) {self.panel=Panel::None;self.modal=Modal::None;self.pending=None;self.pet_multi_delete=false;self.pet_marked.clear();}
    pub fn profile(&mut self,stats:bool) {self.close();self.panel=Panel::Profile;self.profile_stats=stats;}
    pub fn pets(&mut self) {self.close();self.panel=Panel::Pets;}
    pub fn modal(&mut self,modal:Modal) {self.close();self.modal=modal;}
}
