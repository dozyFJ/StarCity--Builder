//! Clicker economy, capsule drops and persistent player collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Home, Capsules, Upgrades, Statistics, Development }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetKind { Cat, Dog, Bunny, Fox, Dragon, Unicorn }
impl PetKind {
    pub fn name(self) -> &'static str { match self { Self::Cat=>"Котик", Self::Dog=>"Щенок", Self::Bunny=>"Зайчик", Self::Fox=>"Лисёнок", Self::Dragon=>"Дракон", Self::Unicorn=>"Единорог" } }
    pub fn rarity(self) -> &'static str { match self { Self::Cat|Self::Dog=>"Обычный", Self::Bunny=>"Редкий", Self::Fox=>"Эпический", _=>"Легендарный" } }
    pub fn evolution_tier(self) -> u8 {
        match self { Self::Cat | Self::Dog => 1, Self::Bunny => 2, Self::Fox => 3, Self::Dragon => 4, Self::Unicorn => 5 }
    }
    pub fn bonus(self) -> f64 { match self { Self::Cat=>1.2, Self::Dog=>1.3, Self::Bunny=>1.6, Self::Fox=>2.0, Self::Dragon=>3.0, Self::Unicorn=>4.0 } }
    pub fn color(self) -> [u8;3] { match self { Self::Cat=>[124,178,255], Self::Dog=>[227,176,117], Self::Bunny=>[245,186,226], Self::Fox=>[255,139,69], Self::Dragon=>[135,102,242], Self::Unicorn=>[132,246,226] } }
    fn from_id(id: usize) -> Option<Self> { [Self::Cat,Self::Dog,Self::Bunny,Self::Fox,Self::Dragon,Self::Unicorn].get(id).copied() }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capsule { Meadow, Neon }
impl Capsule {
    pub fn name(self)-> &'static str { match self { Self::Meadow=>"Луговое яйцо",Self::Neon=>"Солнечное яйцо" } }
    pub fn price(self)->f64 { match self { Self::Meadow=>250.0,Self::Neon=>2500.0 } }
    pub fn drops(self)-> &'static [(PetKind,u8); 4] { match self {
        Self::Meadow=>&[(PetKind::Cat,45),(PetKind::Dog,35),(PetKind::Bunny,17),(PetKind::Fox,3)],
        Self::Neon=>&[(PetKind::Bunny,45),(PetKind::Fox,35),(PetKind::Dragon,17),(PetKind::Unicorn,3)] } }
    pub fn price_with_luck(self, luck: u8) -> f64 {
        self.price() + self.price() * f64::from(luck.min(5)) / 5.0
    }
    pub fn drops_with_luck(self, luck: u8) -> [(PetKind, u8); 4] {
        let luck = luck.min(5);
        let mut drops = *self.drops();
        drops[0].1 -= 2 * luck;
        drops[1].1 -= 2 * luck;
        drops[2].1 += 3 * luck;
        drops[3].1 += luck;
        drops
    }
    fn roll(self, roll: u64, luck: u8) -> PetKind {
        let mut threshold = 0;
        for (pet, weight) in self.drops_with_luck(luck) {
            threshold += u64::from(weight);
            if roll % 100 < threshold { return pet; }
        }
        self.drops()[0].0
    }
}
#[derive(Debug, Clone)]
pub struct Hatch { pub capsule:Capsule, pub inventory_index:usize, pub count:usize, pub elapsed:f64 }
#[derive(Debug, Clone)]
pub struct ActiveBoost {
    pub name: String,
    pub expires_at: f64,
}
pub const CITY_SIDE: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Building { Road, House, Shop, Park }

impl Building {
    pub const ALL: [Self; 4] = [Self::Road, Self::House, Self::Shop, Self::Park];

    pub fn name(self) -> &'static str {
        match self { Self::Road => "Дорога", Self::House => "Дом", Self::Shop => "Магазин", Self::Park => "Парк" }
    }

    pub fn cost(self) -> f64 {
        match self { Self::Road => 25.0, Self::House => 120.0, Self::Shop => 180.0, Self::Park => 75.0 }
    }

    pub fn income(self) -> f64 {
        match self { Self::Road => 0.0, Self::House => 2.0, Self::Shop => 5.0, Self::Park => 1.0 }
    }

    pub fn residents(self) -> u32 { if self == Self::House { 3 } else { 0 } }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub coins: f64,
    pub base_income: f64,
    pub upgrades: u32,
    pub tab: Tab,
    pub paused: bool,
    pub city_xp: u64,
    pub total_earned: f64,
    pub played_seconds: f64,
    pub rebirths: u32,
    pub pets: Vec<PetKind>,
    pub equipped_pet: Option<usize>,
    pub hatch: Option<Hatch>,
    pub pets_open: bool,
    pub equipped_pets: Vec<usize>,
    pub interface_open: bool,
    pub dark_mode: bool,
    pub active_boosts: Vec<ActiveBoost>,
    pub plots: [Option<Building>; CITY_SIDE * CITY_SIDE],
    pub selected_building: Building,
    pet_xp: std::collections::BTreeMap<usize, u64>,
    rng: u64,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            coins: 300.0, base_income: 1.0, upgrades: 0, tab: Tab::Home, paused: false,
            city_xp: 0, total_earned: 0.0, played_seconds: 0.0, rebirths: 0,
            pets: Vec::new(), equipped_pet: None, hatch: None, pets_open: false,
            equipped_pets: Vec::new(), interface_open: false, dark_mode: false,
            active_boosts: Vec::new(), plots: [None; CITY_SIDE * CITY_SIDE],
            selected_building: Building::House, pet_xp: Default::default(),
            rng: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |time| time.as_nanos() as u64).max(1),
        }
    }
}

impl Game {
    pub fn build(&mut self, index: usize, building: Building) -> bool {
        if self.paused || self.hatch.is_some() || self.interface_open || self.pets_open
            || self.coins < building.cost()
        {
            return false;
        }
        let Some(plot) = self.plots.get_mut(index) else { return false; };
        if plot.is_some() { return false; }
        *plot = Some(building);
        self.coins -= building.cost();
        self.city_xp = self.city_xp.saturating_add(if building == Building::Road { 2 } else { 10 });
        for &pet in &self.equipped_pets {
            let xp = self.pet_xp.entry(pet).or_default();
            *xp = xp.saturating_add(1);
        }
        true
    }

    pub fn population(&self) -> u32 { self.plots.iter().flatten().map(|building| building.residents()).sum() }
    pub fn building_count(&self) -> usize { self.plots.iter().filter(|plot| plot.is_some()).count() }

    pub fn tick(&mut self, seconds: f64) {
        if self.paused || !seconds.is_finite() { return; }
        let dt = seconds.max(0.0);
        let income = self.coins_per_second() * dt;
        self.coins += income;
        self.total_earned += income;
        self.played_seconds += dt;
        if let Some(hatch) = &mut self.hatch { hatch.elapsed += dt; }
    }

    pub fn level(&self)->u64 { self.city_xp/100+1 }
    pub fn level_progress(&self)->f32 { (self.city_xp%100) as f32/100.0 }
    pub fn rank(&self)-> &'static str { match self.level() { 1..=4=>"НОВИЧОК",5..=14=>"ИСКАТЕЛЬ",15..=29=>"МАСТЕР",_=>"ЛЕГЕНДА" } }
    pub fn rebirth_bonus(&self)->f64 { 2.0_f64.powi(self.rebirths.min(30) as i32) }
    pub fn active_pet(&self)->Option<PetKind> { self.equipped_pet.and_then(|i|self.pets.get(i)).copied() }
    pub fn pet_bonus(&self)->f64 {
        1.0+self.equipped_pets.iter().filter_map(|&i|self.pets.get(i)).map(|p|p.bonus()-1.0).sum::<f64>()
    }
    pub fn coins_per_second(&self) -> f64 {
        let buildings: f64 = self.plots.iter().flatten().map(|building| building.income()).sum();
        (self.base_income + buildings) * self.rebirth_bonus() * self.pet_bonus()
    }
    pub fn upgrade_price(&self)->f64 { 50.0*2.0_f64.powi(self.upgrades.min(30) as i32) }
    pub fn upgrade(&mut self)->bool {
        let cost=self.upgrade_price();
        if self.paused||self.hatch.is_some()||self.coins<cost { return false; }
        self.coins-=cost; self.upgrades=self.upgrades.saturating_add(1); self.base_income+=1.0; true
    }
    pub fn rebirth_price_at(rebirths: u32) -> f64 { 10_000.0 * 10.0_f64.powi(rebirths.min(30) as i32) }
    pub fn rebirth_price(&self)->f64 { Self::rebirth_price_at(self.rebirths) }
    pub fn rebirth(&mut self)->bool {
        if self.paused||self.hatch.is_some()||self.coins<self.rebirth_price() { return false; }
        self.coins = 300.0;
        self.base_income = 1.0;
        self.upgrades = 0;
        self.plots.fill(None);
        self.rebirths = self.rebirths.saturating_add(1);
        true
    }
    pub fn buy_capsule(&mut self, capsule: Capsule) -> bool {
        self.buy_eggs(capsule, 1, 0)
    }
    pub fn buy_eggs(&mut self, egg: Capsule, count: usize, luck: u8) -> bool {
        if self.paused || self.hatch.is_some() || !(1..=10).contains(&count) || luck > 5 || count > 1000usize.saturating_sub(self.pets.len()) {
            return false;
        }
        let price = egg.price_with_luck(luck) * count as f64;
        if self.coins < price { return false; }
        let index = self.pets.len();
        for _ in 0..count {
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            self.pets.push(egg.roll(self.rng, luck));
        }
        self.coins -= price;
        self.hatch = Some(Hatch { capsule: egg, inventory_index: index, count, elapsed: 0.0 });
        true
    }
    pub fn equip_pet(&mut self,index:usize)->bool {
        if self.paused||index>=self.pets.len() { return false; }
        if !self.equipped_pets.contains(&index) {
            if self.equipped_pets.len()==3 { self.equipped_pets.pop(); }
            self.equipped_pets.push(index);
        }
        self.equipped_pet=self.equipped_pets.first().copied(); true
    }
    pub fn unequip_pet(&mut self,index:usize)->bool {
        if self.paused||self.hatch.is_some() { return false; }
        let Some(slot)=self.equipped_pets.iter().position(|&i|i==index) else { return false; };
        self.equipped_pets.remove(slot);self.equipped_pet=self.equipped_pets.first().copied();
        true
    }
    pub fn pet_xp(&self, index: usize) -> u64 {
        self.pet_xp.get(&index).copied().unwrap_or(0)
    }
    pub fn equip_best_pet(&mut self) -> bool {
        let best = (0..self.pets.len()).filter(|index| !self.equipped_pets.contains(index))
            .max_by_key(|&index| (self.pets[index] as u8, self.pet_xp(index), std::cmp::Reverse(index)));
        best.is_some_and(|index| self.equip_pet(index))
    }
    pub fn unequip_all_pets(&mut self) {
        if self.paused || self.hatch.is_some() { return; }
        self.equipped_pets.clear();
        self.equipped_pet = None;
    }
    pub fn delete_pets(&mut self, mut indices: Vec<usize>) -> bool {
        if self.paused || self.hatch.is_some() || indices.is_empty()
            || indices.iter().any(|&index| index >= self.pets.len()) { return false; }
        indices.sort_unstable();
        indices.dedup();
        for index in indices.into_iter().rev() { self.delete_pet(index); }
        true
    }
    pub fn delete_pet(&mut self,index:usize)->bool {
        if self.paused||self.hatch.is_some()||index>=self.pets.len() {return false;}
        self.pets.remove(index);
        let mut later_xp = self.pet_xp.split_off(&index);
        later_xp.remove(&index);
        self.pet_xp.extend(later_xp.into_iter().map(|(slot, xp)| (slot - 1, xp)));
        self.equipped_pets.retain(|&i|i!=index);
        for i in &mut self.equipped_pets {if *i>index {*i-=1;}}
        self.equipped_pet=self.equipped_pets.first().copied();
        true
    }
    pub fn craft_result(kind:PetKind)->Option<PetKind> {
        match kind {PetKind::Cat|PetKind::Dog=>Some(PetKind::Bunny),PetKind::Bunny=>Some(PetKind::Fox),PetKind::Fox=>Some(PetKind::Dragon),PetKind::Dragon=>Some(PetKind::Unicorn),PetKind::Unicorn=>None}
    }
    pub fn craft_materials(&self,index:usize)->Option<Vec<usize>> {
        let kind=*self.pets.get(index)?;Self::craft_result(kind)?;
        let mut materials=vec![index];
        for (i,&pet) in self.pets.iter().enumerate() {if i!=index&&pet==kind {materials.push(i);if materials.len()==3 {return Some(materials);}}}
        None
    }
    pub fn craft_pet(&mut self,index:usize)->Option<usize> {
        if self.paused||self.hatch.is_some() {return None;}
        let result=Self::craft_result(*self.pets.get(index)?)?;
        let mut materials=self.craft_materials(index)?;
        let equipped=materials.iter().any(|i|self.equipped_pets.contains(i));
        materials.sort_unstable_by(|a,b|b.cmp(a));
        for i in materials {self.delete_pet(i);}
        let index=self.pets.len();self.pets.push(result);
        if equipped {self.equip_pet(index);}
        Some(index)
    }
    pub fn finish_hatch(&mut self,equip:bool)->bool {
        let Some(h)=&self.hatch else { return false; };
        if self.paused||h.elapsed<3.0 { return false; }
        let first = h.inventory_index;
        let last = first + h.count;
        self.hatch = None;
        if equip {
            if let Some(index) = (first..last).max_by_key(|&index| self.pets[index] as u8) {
                self.equip_pet(index);
            }
        }
        true
    }
    pub fn save_text(&self) -> String {
        let pets = self.pets.iter().map(|pet| (*pet as u8).to_string()).collect::<Vec<_>>().join(",");
        let team = self.equipped_pets.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
        let plots: String = self.plots.iter().map(|plot| match plot {
            None => '.', Some(Building::Road) => 'r', Some(Building::House) => 'h',
            Some(Building::Shop) => 's', Some(Building::Park) => 'p',
        }).collect();
        let xp = self.pet_xp.iter().map(|(index, xp)| format!("{index}:{xp}")).collect::<Vec<_>>().join(",");
        format!("STAR_CITY_1\n{} {} {} {} {} {} {} {}\n{plots}\n{pets}\n{team}\n{xp}\n",
            self.coins, self.base_income, self.upgrades, self.city_xp, self.total_earned,
            self.played_seconds, self.rebirths, self.dark_mode)
    }

    pub fn from_save(text: &str) -> Option<Self> {
        let mut lines = text.lines();
        if lines.next()? != "STAR_CITY_1" { return None; }
        let mut fields = lines.next()?.split_whitespace();
        let mut game = Self::default();
        game.coins = fields.next()?.parse().ok()?;
        game.base_income = fields.next()?.parse().ok()?;
        game.upgrades = fields.next()?.parse().ok()?;
        game.city_xp = fields.next()?.parse().ok()?;
        game.total_earned = fields.next()?.parse().ok()?;
        game.played_seconds = fields.next()?.parse().ok()?;
        game.rebirths = fields.next()?.parse().ok()?;
        game.dark_mode = fields.next()?.parse().ok()?;
        if ![game.coins, game.base_income, game.total_earned, game.played_seconds]
            .into_iter().all(|value| value.is_finite() && value >= 0.0)
        { return None; }

        let plots = lines.next()?.as_bytes();
        if plots.len() != game.plots.len() { return None; }
        for (plot, &kind) in game.plots.iter_mut().zip(plots) {
            *plot = match kind {
                b'.' => None, b'r' => Some(Building::Road), b'h' => Some(Building::House),
                b's' => Some(Building::Shop), b'p' => Some(Building::Park), _ => return None,
            };
        }
        for id in lines.next()?.split(',').filter(|id| !id.is_empty()).take(1001) {
            game.pets.push(PetKind::from_id(id.parse().ok()?)?);
        }
        if game.pets.len() > 1000 { return None; }
        for index in lines.next()?.split(',').filter(|id| !id.is_empty()) {
            let index = index.parse().ok()?;
            if index >= game.pets.len() || game.equipped_pets.contains(&index) || game.equipped_pets.len() >= 3 {
                return None;
            }
            game.equipped_pets.push(index);
        }
        game.equipped_pet = game.equipped_pets.first().copied();
        for entry in lines.next()?.split(',').filter(|entry| !entry.is_empty()) {
            let (index, xp) = entry.split_once(':')?;
            let index = index.parse().ok()?;
            if index >= game.pets.len() { return None; }
            game.pet_xp.insert(index, xp.parse().ok()?);
        }
        Some(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buildings_cost_money_and_change_income_and_population() {
        let mut game = Game::default();
        assert!(game.build(0, Building::House));
        assert!(!game.build(0, Building::Shop));
        assert!(!game.build(999, Building::House));
        assert_eq!(game.coins, 180.0);
        assert_eq!(game.population(), 3);
        assert_eq!(game.coins_per_second(), 3.0);
        game.tick(2.0);
        assert_eq!(game.coins, 186.0);
        assert!(game.build(1, Building::Shop));
        assert!(!game.build(2, Building::Road));
    }

    #[test]
    fn city_saves_plots_and_pets_and_keeps_collection_after_restart() {
        let mut game = Game::default();
        game.pets.push(PetKind::Cat);
        game.equip_pet(0);
        game.dark_mode = true;
        assert!(game.build(5, Building::House));
        let mut game = Game::from_save(&game.save_text()).unwrap();
        assert_eq!(game.plots[5], Some(Building::House));
        assert_eq!(game.equipped_pets, vec![0]);
        assert_eq!(game.pet_xp(0), 1);
        assert!(game.dark_mode);
        game.coins = game.rebirth_price();
        assert!(game.rebirth());
        assert_eq!(game.building_count(), 0);
        assert_eq!(game.pets, vec![PetKind::Cat]);
        assert_eq!(game.rebirth_bonus(), 2.0);
    }

    #[test]
    fn egg_batches_and_crafting_keep_the_pet_economy() {
        let mut game = Game::default();
        game.coins = 1_500.0;
        assert!(game.buy_eggs(Capsule::Meadow, 3, 5));
        assert_eq!(game.coins, 0.0);
        assert_eq!(game.pets.len(), 3);
        assert!(!game.buy_capsule(Capsule::Meadow));
        game.tick(3.0);
        assert!(game.finish_hatch(true));
        game.pets = vec![PetKind::Cat; 3];
        assert_eq!(game.craft_pet(0), Some(0));
        assert_eq!(game.pets, vec![PetKind::Bunny]);
    }
}
