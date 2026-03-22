// Imports des structures générées par protobuf (via crate::pb)
use crate::pb::{Ant, AntRole, AntState, CommandRequest, CommandType, GameState, Resource, ResourceType, Nest, Room, RoomType};
// hecs est un framework ECS (Entity Component System) ultra-rapide utilisé pour gérer des milliers d'entités (les fourmis, la nourriture, etc.)
use hecs::{World, Entity};
// rand::Rng fournit les fonctionnalités de génération de nombres aléatoires
use rand::Rng;
// Collections standards pour stocker nos données
use std::collections::{HashMap, HashSet, VecDeque};
// Système de bruit pour générer la carte procédurale
use noise::{NoiseFn, OpenSimplex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Water,
    Dirt,
    Rock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherState {
    Clear,
    Rain,
}

pub fn get_terrain_type(x: f32, y: f32, seed: u32) -> TerrainType {
    let simplex = OpenSimplex::new(seed);
    // Multiply by scale 0.005
    let val = simplex.get([(x * 0.005) as f64, (y * 0.005) as f64]);
    
    if val < -0.3 {
        TerrainType::Water
    } else if val <= 0.6 {
        TerrainType::Dirt
    } else {
        TerrainType::Rock
    }
}

// --- CONSTANTES DU JEU ---
// Nombre de fois que le moteur de jeu se met à jour par seconde
pub const TICK_RATE: f32 = 100.0;
// Taille du terrain (3162x3162 correspond grossièrement à une aire de 10km²)
pub const MAP_SIZE: f32 = 3162.0;

// Biology constants
// Paramètres biologiques qui régissent la simulation de la colonie
pub const ANT_SIZE: f32 = 2.0;
// Vitesse d'une fourmi par seconde (ex: 6 unités / seconde)
pub const ANT_SPEED_PER_SEC: f32 = 3.0 * ANT_SIZE; 
// La vitesse recalculée par rapport au cycle de tick pour garantir la fluidité du déplacement
pub const ANT_SPEED_PER_TICK: f32 = ANT_SPEED_PER_SEC / TICK_RATE;

// Coût énergétique d'une reine par minute au repos et lors de la ponte
pub const QUEEN_METABOLISM_IDLE: u32 = 1; // per minute
pub const QUEEN_METABOLISM_LAYING: u32 = 10; // per minute
pub const MAX_TURN_ANGLE: f32 = 0.2; // Maximum turn angle per tick in radians
// L'espérance de vie maximale de la reine en Ticks (10 années in-game => 87600 heures de jeu ou 87600 secondes IRL)
pub const QUEEN_MAX_AGE_TICKS: u64 = 10 * 365 * 24 * 60 * 60 * 100; // 10 in-game years where 1 real second = 1 game hour. Wait, 1 second = 3600 seconds. 10 years = 87600 hours = 87600 real seconds.
// Intervalle de perte de protéines pour la larve/nymphe
pub const NYMPH_METABOLISM_INTERVAL: u64 = 90 * 100; 
// Protéines récupérables sur le cadavre d'une fourmi morte
pub const DEAD_ANT_PROTEINS: u16 = 20;

// Ressources : maximum par cellule de la grille spatiale et au niveau global de la carte
pub const RESOURCE_MAX_PER_CELL: u16 = 30;
pub const RESOURCE_MAX_GLOBAL: usize = 1000;

// --- COMPOSANTS ECS ---
// Ces "structs" sont des composants de la bibliothèque hecs attachés aux "Entities"
// Composant pour stocker la position 2D
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// Composant pour stocker le vecteur de vitesse 2D
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

// Composant contenant toutes les métadonnées de simulation pour une fourmi
pub struct AntData {
    pub id: u32,                  // Identifiant unique
    pub role: AntRole,            // Rôle (Ouvrière, Soldat, Reine...)
    pub state: AntState,          // Etat actuel (Exploration, Retour...)
    pub angle: f32,               // Angle de rotation actuel de la fourmi en radians
    pub faction_id: u32,          // Identifiant de la faction/colonie à laquelle elle appartient
    pub age: u64,                 // Son âge en ticks
}

// Composant pour une entité de type "Ressource"
pub struct ResourceData {
    pub quantity: u16,            // Quantité restante avant disparition
    pub res_type: ResourceType,   // Type (Animal ou Plante)
}

// --- OPTIMISATION SPATIALE ("Spatial Hashing") ---
// Map split into grid cells of size SPATIAL_CELL_SIZE for hash grid
// L'espace est divisé en carrés de taille SPATIAL_CELL_SIZE pour y répartir toutes les entités
pub const SPATIAL_CELL_SIZE: f32 = 50.0;
// Nombre de colonnes et de lignes formées par ces carrés
pub const SPATIAL_COLS: usize = (MAP_SIZE / SPATIAL_CELL_SIZE) as usize + 1;

// La grille spatiale ("Spatial Hash Grid"). Au lieu de comparer la position de chaque fourmi avec
// la position de toutes les autres pour trouver des voisins (Calcul en O(n²)), on insère
// leurs positions dans cette grille (Calcul en O(1)).
pub struct SpatialGrid {
    // Une HashMap dont la clé est un tuple (Colonne, Ligne) et la valeur est un tableau d'Entités hecs.
    pub cells: rustc_hash::FxHashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    // Constructeur d'une nouvelle grille spatiale
    pub fn new() -> Self {
        Self { cells: rustc_hash::FxHashMap::default() }
    }
    
    // Fonction appelée à chaque tick pour vider la grille de l'ancien état avant de tout réinsérer
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    
    // Fonction calculant la case (Colonne `cx`, Ligne `cy`) à partir des coordonnées réelles (x, y) de l'entité
    fn get_cell_coords(x: f32, y: f32) -> (i32, i32) {
        ((x / SPATIAL_CELL_SIZE).floor() as i32, (y / SPATIAL_CELL_SIZE).floor() as i32)
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        let coords = Self::get_cell_coords(x, y);
        // Ajout de l'entité dans le tableau correspondant dans le HashMap, en créant le tableau si absent.
        self.cells.entry(coords).or_insert_with(Vec::new).push(entity);
    }
    
    // Fonction cruciale : récupère toutes les entités se trouvant autour d'un point (x,y) donné dans un rayon défini.
    pub fn get_nearby(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        // Calcule la bounding-box des cellules à couvrir avec le rayon cible
        let min_cell = Self::get_cell_coords(x - radius, y - radius);
        let max_cell = Self::get_cell_coords(x + radius, y + radius);
        
        // Itère uniquement sur les cellules correspondantes et collecte les entités
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.cells.get(&(cx, cy)) {
                    result.extend(entities.iter().copied()); // Copie rapide car `Entity` est très léger
                }
            }
        }
        result
    }
}

// Statistiques associées à une Faction (Colonie) de fourmis
pub struct FactionStats {
    pub proteins: u32,          // Quantité de nourriture globale disponible dans toute la colonie
    pub has_queen: bool,        // Indique si la reine est toujours en vie
}

// Composant central ("La Machine à états") qui gère le monde entier
pub struct SimulationEngine {
    world: World,                             // Le monde ECS `hecs` qui contient toutes les entités
    tick: u64,                                // Compteur de boucles de jeu ("Ticks")
    spatial_grid: SpatialGrid,                // Notre grille d'optimisation (mises à jour à chaque cycle)
    nests: Vec<Nest>,                         // La liste des nids et de leurs pièces (coordonnées)
    factions: HashMap<u32, FactionStats>,     // Le dictionnaire associant l'ID Faction à ses statistiques
    next_entity_id: u32,                      // Le compteur pour attribuer des IDs uniques aux nouvelles entités
    pheromones_exploration: HashMap<(i32, i32), f32>, // Grille de phéromones d'exploration
    pheromones_food: HashMap<(i32, i32), f32>, // Grille de phéromones de nourriture
    map_seed: u32,                            // Graine unique pour la génération procédurale
    pub weather: WeatherState,                // Etat global de la météo
}

impl SimulationEngine {
    // Constructeur : Initialise le moteur et démarre le monde
    pub fn new() -> Self {
        let mut engine = Self {
            world: World::new(),
            tick: 0,
            spatial_grid: SpatialGrid::new(),
            nests: Vec::new(),
            factions: HashMap::new(),
            next_entity_id: 1, // On commence à attribuer les ID depuis 1
            pheromones_exploration: HashMap::new(),
            pheromones_food: HashMap::new(),
            map_seed: rand::thread_rng().gen(),
            weather: WeatherState::Clear,
        };
        
        // Spawn 3 Factions (Multi-Colony)
        // Construit le monde par défaut au démarrage
        engine.setup_initial_world();
        engine // Retourne l'instance du moteur configurée
    }
    
    // Générateur d'ID unique de façon monotone
    fn generate_id(&mut self) -> u32 {
        self.next_entity_id += 1;
        self.next_entity_id
    }

    // Procédure de la génération du monde procédural initial
    fn setup_initial_world(&mut self) {
        // `thread_rng()` récupère une instance locale au thread du générateur de nombres aléatoires
        let mut rng = rand::thread_rng();
        
        // Spawn 3 colonies far apart
        for faction_id in 1..=3 {
            // Tire des coordonnées aléatoires pour le Nid (sans que cela soit collé au bord de carte)
            let nx = rng.gen_range(500.0..MAP_SIZE - 500.0);
            let ny = rng.gen_range(500.0..MAP_SIZE - 500.0);
            
            // Generate Rooms
            // Initialisation de la géométrie du nid (uniquement conceptuelle au niveau données)
            let mut rooms = Vec::new();
            // Pièce Royale (Chambre de la Reine)
            rooms.push(Room {
                id: faction_id * 100 + 1,
                room_type: RoomType::Royal as i32,
                x: nx,
                y: ny,
                radius: 30.0,
            });
            // Le grenier où la nourriture récoltée doit être stockée
            rooms.push(Room {
                id: faction_id * 100 + 2,
                room_type: RoomType::Granary as i32,
                x: nx + 60.0,
                y: ny + 20.0,
                radius: 20.0,
            });
            // La nurserie (là où les œufs apparaissent conceptuellement)
            rooms.push(Room {
                id: faction_id * 100 + 3,
                room_type: RoomType::Nursery as i32,
                x: nx - 30.0,
                y: ny + 60.0,
                radius: 40.0,
            });
            
            // Sauvegarde de l'instance du nid associé à la faction nouvellement créée
            self.nests.push(Nest {
                faction_id,
                rooms,
            });
            
            // Chaque colonie débute symboliquement avec 200 unités de protéines et sa Reine vivante
            self.factions.insert(faction_id, FactionStats { proteins: 200, has_queen: true });
            
            // Spawn Queen
            // Génère l'ID et invoque (spawn) l'Entity (La Reine) dans l'ECS principal `hecs`
            let q_id = self.generate_id();
            self.world.spawn((
                Position { x: nx, y: ny }, // Composant 1 : Position
                AntData {                  // Composant 2 : Metadonnées AntData
                    id: q_id,
                    role: AntRole::Queen,
                    state: AntState::Idle, // La Reine reste oisive et immobile
                    // TAU (constante = 2 * PI) représente un cercle complet de 360° en radians.
                    angle: rng.gen_range(0.0..std::f32::consts::TAU),
                    faction_id,
                    age: 0,
                },
            )); // Note: Le tuple `(Position, AntData)` sera attribué à l'Entity
            
            // Spawn initial Workers
            // Génère et invoque 10 ouvrières qui commenceront directement à explorer
            for _ in 0..10 {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let w_id = self.generate_id();
                self.world.spawn((
                    Position { x: nx, y: ny },
                    Velocity {
                        vx: angle.cos(),   // Mathématiques de trigo : Cosinus pour propulser en X
                        vy: angle.sin(),   // Mathématiques de trigo : Sinus pour propulser en Y
                    },
                    AntData {
                        id: w_id,
                        role: AntRole::Worker,
                        state: AntState::Exploring, // Directement actives dans la boucle comportementale
                        angle,
                        faction_id,
                        age: 0,
                    },
                ));
            }

            // Spawn initial Soldiers
            // Génère et invoque 5 soldats avec base comportementale identique aux ouvrières
            for _ in 0..5 {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let s_id = self.generate_id();
                self.world.spawn((
                    Position { x: nx, y: ny },
                    Velocity {
                        vx: angle.cos(),
                        vy: angle.sin(),
                    },
                    AntData {
                        id: s_id,
                        role: AntRole::Soldier,
                        state: AntState::Exploring,
                        angle,
                        faction_id,
                        age: 0,
                    },
                ));
            }
        }
    }
    
    // Flood fill algorithm for resources (max 30 per cell)
    // Algorithme "Flood Fill" (Remplissage par diffusion) pour les apparitions de ressources (max 30 éléments par case).
    fn spawn_food_cluster(&mut self, start_x: f32, start_y: f32, mut amount: u16, res_type: ResourceType) {
        // Utilisation d'une file d'attente (Queue) et d'un Set `visited` classiques pour l'algorithme BFS/Flood Fill.
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        
        // Convertit les coordonnées du monde (x,y) actuelles en index de grille (cx,cy).
        let cx = (start_x / SPATIAL_CELL_SIZE).max(0.0) as i32;
        let cy = (start_y / SPATIAL_CELL_SIZE).max(0.0) as i32;
        
        queue.push_back((cx, cy));
        visited.insert((cx, cy));
        
        // Tant qu'il reste des cellules à vérifier dans la file
        while let Some((x, y)) = queue.pop_front() {
            // Si on a posé toute la quantité demandée, on stoppe l'algorithme ("early return").
            if amount == 0 {
                break;
            }
            
            // On vérifie combien de nourriture est DEJA présente dans la cellule
            let mut current_cell_amount = 0;
            let cx_idx = x as i32;
            let cy_idx = y as i32;
            // On consulte la structure spatiale
            if let Some(ents) = self.spatial_grid.cells.get(&(cx_idx, cy_idx)) {
                for &e in ents {
                    // Pour chaque entité, on vérifie si elle dispose du composant ResourceData
                    if let Ok(res) = self.world.get::<&ResourceData>(e) {
                         current_cell_amount += res.quantity;
                    }
                }
            }

            // `saturating_sub` empêche une tentative mathématique de descendre sous 0. 
            // On ne peut avoir plus de RESOURCE_MAX_PER_CELL.
            let available_space = RESOURCE_MAX_PER_CELL.saturating_sub(current_cell_amount);
            
            if available_space > 0 {
                // On dépose le strict minimum entre "ce qu'il reste à déposer" et "la place disponible"
                let place_amount = amount.min(available_space);
                // On soustrait ce qu'il reste à déposer
                amount -= place_amount;
                
                // On recalcule le point central (physique, en float) de la cellule de grille concernée.
                let pos_x = (x as f32) * SPATIAL_CELL_SIZE + (SPATIAL_CELL_SIZE / 2.0);
                let pos_y = (y as f32) * SPATIAL_CELL_SIZE + (SPATIAL_CELL_SIZE / 2.0);
                
                // On 'spawn' (crée) alors l'entité Nourriture (Plant/Animal) dans le monde `hecs`
                self.world.spawn((
                    Position { x: pos_x, y: pos_y },
                    ResourceData {
                        quantity: place_amount,
                        res_type,
                    }
                ));
            }
            
            // S'il reste de la quantité à poser, on ajoute toutes les cases voisines dans la file d'attente.
            if amount > 0 {
                let neighbors = [
                    (x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)
                ];
                for &n in &neighbors {
                    // Vérifie que les voisins générés sont encore à l'intérieur de la carte géographique
                    if n.0 >= 0 && n.0 < SPATIAL_COLS as i32 && n.1 >= 0 && n.1 < SPATIAL_COLS as i32 {
                        // S'ils ne sont pas déjà visités, on les ajoute
                        if !visited.contains(&n) {
                            visited.insert(n);
                            queue.push_back(n);
                        }
                    }
                }
            }
        }
    }

    // Point d'entrée pour le réseau (depuis grpc.rs). Traite asynchronement chaque commande externe reçue.
    pub fn process_command(&mut self, cmd: CommandRequest) {
        let mut rng = rand::thread_rng();
        log::info!("Engine: Processing command type {:?}", cmd.command());
        
        // "Pattern Matching" global (comme un super-switch) sur le type de commande
        match cmd.command() {
            // Un utilisateur a cliqué pour générer de la nourriture (Plantes)
            CommandType::SpawnFood | CommandType::SpawnPlant => {
                // `unwrap_or_else` garantit une position par défaut si le front n'a pas transmis ses coordonnées (ex: null).
                let x = cmd.x.unwrap_or_else(|| rng.gen_range(100.0..MAP_SIZE - 100.0));
                let y = cmd.y.unwrap_or_else(|| rng.gen_range(100.0..MAP_SIZE - 100.0));
                let amount = cmd.amount.unwrap_or(100);
                log::info!("Engine: Spawning {} Plant at ({}, {})", amount, x, y);
                // Le moteur utilise notre propre module (BFS ci-dessus) de Spawn.
                self.spawn_food_cluster(x, y, amount as u16, ResourceType::Plant);
            }
            // Commande pour spawner un prédateur mort ou animal
            CommandType::SpawnAnimal => {
                let x = cmd.x.unwrap_or_else(|| rng.gen_range(100.0..MAP_SIZE - 100.0));
                let y = cmd.y.unwrap_or_else(|| rng.gen_range(100.0..MAP_SIZE - 100.0));
                let amount = cmd.amount.unwrap_or(100);
                log::info!("Engine: Spawning {} Animal at ({}, {})", amount, x, y);
                self.spawn_food_cluster(x, y, amount as u16, ResourceType::Animal);
            }
            // Remise à zéro totale du jeu ("Reset State")
            CommandType::Reset => {
                log::info!("Engine: Resetting world");
                // On réassigne une nouvelle instance de SimulationEngine sur self, le "Garbage Collector" fera le reste.
                *self = Self::new();
            }
            // Commande par défaut ignorée (si un nouveau Type existe via Protobuf et n'est pas encore implémenté en Rust)
            _ => {
                log::info!("Engine: Unhandled/Ignored command");
            }
        }
    }
    
    // Système ECS pour mettre à jour la grille spatiale
    fn update_spatial_grid_system(&mut self) {
        // A chaque tour, on vide la grille d'optimisation...
        self.spatial_grid.clear();
        // ... et on demande à `hecs` de nous trouver toutes les entités qui possèdent une composante `Position` 
        // pour les réinsérer dans la grille aux nouvelles coordonnées.
        for (entity, pos) in self.world.query::<&Position>().iter() {
            self.spatial_grid.insert(entity, pos.x, pos.y);
        }
    }

    // La boucle principale de la simulation. Cette fonction est appelée 100 fois par seconde (TICK_RATE).
    pub fn tick(&mut self) {
        self.tick += 1;
        let mut rng = rand::thread_rng();
        
        // 1. Rebuild Spatial Hash Grid
        self.update_spatial_grid_system();

        // Evaporation des phéromones (tous les TICK_RATE cycles = 1 seconde)
        if self.tick % (TICK_RATE as u64) == 0 {
            let (exp_decay, food_decay) = if self.weather == WeatherState::Rain {
                (0.50, 0.90) // Rain: evaporation x10 (perd 50% et 10%)
            } else {
                (0.95, 0.99) // Clear: normal (perd 5% et 1%)
            };

            // Exploration
            self.pheromones_exploration.retain(|_, v| {
                *v *= exp_decay;
                *v > 0.1
            });
            // Nourriture
            self.pheromones_food.retain(|_, v| {
                *v *= food_decay;
                *v > 0.1
            });
        }

        // 2. Background Resource Spawning
        // Evénement régulier (toutes les 60 secondes virtuelles)
        if self.tick % (60 * (TICK_RATE as u64)) == 0 {
            // Weather change chance
            if rng.gen_bool(0.2) { // 20% chance to change weather
                self.weather = if self.weather == WeatherState::Clear { WeatherState::Rain } else { WeatherState::Clear };
                log::info!("Engine: Weather changed to {:?}", self.weather);
            }

            // Background plant/animal logic (limited by RESOURCE_MAX_GLOBAL)
            let mut plant_qty: u32 = 0;
            let mut animal_qty: u32 = 0;
            // On compte le nombre de ressources actuellement présentes sur le terrain
            for (_, res) in self.world.query_mut::<&ResourceData>() {
                match res.res_type {
                    ResourceType::Plant => plant_qty += res.quantity as u32,
                    ResourceType::Animal => animal_qty += res.quantity as u32,
                }
            }
            // S'il manque des plantes max par rapport à notre plafond global, la nature en régénère une grappe
            if plant_qty < RESOURCE_MAX_GLOBAL as u32 {
                let x = rng.gen_range(100.0..MAP_SIZE - 100.0);
                let y = rng.gen_range(100.0..MAP_SIZE - 100.0);
                self.spawn_food_cluster(x, y, 100, ResourceType::Plant);
            }
            // S'il manque des animaux morts globalement, la nature en génère
            if animal_qty < RESOURCE_MAX_GLOBAL as u32 {
                let x = rng.gen_range(100.0..MAP_SIZE - 100.0);
                let y = rng.gen_range(100.0..MAP_SIZE - 100.0);
                self.spawn_food_cluster(x, y, 100, ResourceType::Animal);
            }
        }

        // Nymph Metabolism
        // Toutes les ~90 secondes, la colonie consomme une protéine automatiquement pour simuler le métabolisme de croissance des larves
        if self.tick % NYMPH_METABOLISM_INTERVAL == 0 {
            for (_, stats) in self.factions.iter_mut() {
                stats.proteins = stats.proteins.saturating_sub(1);
            }
        }

        // 3. Biological Systems (Aging, Metabolism)
        let mut dead_ants = Vec::new();
        let mut repromoted_queens = Vec::new(); // Inutilisé pour le moment (pouvant servir à élever de futures princesses)
        let mut new_ants_to_spawn = Vec::new();
        
        // On effectue une requête `hecs` afin de boucler sur TOUTES les entités ayant Position ET AntData avec un droit de modification (mut)
        for (entity, (pos, ant)) in self.world.query_mut::<(&Position, &mut AntData)>() {
            // Vieillissement chronologique à chaque tick
            ant.age += 1;
            
            if ant.role == AntRole::Queen {
                // Death by age (if > 10 in-game years)
                // Le composant "meurt" (est consigné) si la reine dépasse l'âge maximum biologique.
                if ant.age > QUEEN_MAX_AGE_TICKS {
                    dead_ants.push((entity, pos.x, pos.y));
                    if let Some(f) = self.factions.get_mut(&ant.faction_id) {
                        f.has_queen = false;
                        repromoted_queens.push(ant.faction_id);
                    }
                    // Termine silencieusement d'évaluer le code de la boucle pour cette fourmi désignée comme morte
                    continue;
                }
                
                // Queen Metabolism
                // Toutes les 60 secondes virtuelles, la reine pond ou mange selon son alimentation.
                if self.tick % (60 * (TICK_RATE as u64)) == 0 {
                    if let Some(stats) = self.factions.get_mut(&ant.faction_id) {
                        if stats.proteins >= QUEEN_METABOLISM_LAYING * 10 { 
                            // Elle consomme ses dizaines de vitamines nécessaires
                            stats.proteins -= QUEEN_METABOLISM_LAYING * 10;
                            // Spawn 10 logic representing 100 eggs
                            // ... Et prépare "conceptuellement" une nouvelle ponte d'œufs.
                            for _ in 0..10 { 
                                new_ants_to_spawn.push((pos.x, pos.y, ant.faction_id));
                            }
                        } else {
                            // Sinon, elle pioche dans les réserves uniquement pour l'entretien indispensable de sa survie
                            stats.proteins = stats.proteins.saturating_sub(QUEEN_METABOLISM_IDLE);
                        }
                    }
                }
            } else if ant.role == AntRole::Worker {
                // Mutation if Queen is dead
                // Si la reine est morte, il y a une probabilité infime mais présente qu'une ouvrière mute en nouvelle reine potentielle (Ponte Parthénogenèse Thélytoque).
                if let Some(f) = self.factions.get(&ant.faction_id) {
                    if !f.has_queen {
                        if rng.gen_range(0..10_000) < 5 { // slightly rare per tick
                            ant.role = AntRole::Queen;
                            ant.age = 0; 
                            if let Some(f2) = self.factions.get_mut(&ant.faction_id) {
                                f2.has_queen = true;
                            }
                        }
                    }
                }
            }
        }
        
        // Process dead
        // ECS: On détruit proprement l'entité de la fourmi passée (vieillie/morte)
        for e in &dead_ants {
            let _ = self.world.despawn(e.0); // Libère la RAM en supprimant l'Entity et tous ses composants
            // Transforme le cadavre en source de protéines récoltables sur le point exact du décès
            self.spawn_food_cluster(e.1, e.2, DEAD_ANT_PROTEINS, ResourceType::Animal);
        }

        // On boucle sur l'ensemble des nouveaux bébés (œufs traités) préparés lors de la ponte
        for (nx, ny, faction_id) in new_ants_to_spawn {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            // Probabilité génétique de naissance définie : 15% Soldat, 85% Ouvrière
            let role = if rng.gen_range(0..100) < 15 { AntRole::Soldier } else { AntRole::Worker };
            let a_id = self.generate_id();
            // Demande au moteur de création d'injecter la nouvelle fourmi à son point d'origine
            self.world.spawn((
                Position { x: nx, y: ny },
                Velocity { vx: angle.cos(), vy: angle.sin() },
                AntData {
                    id: a_id,
                    role,
                    state: AntState::Exploring,
                    angle,
                    faction_id,
                    age: 0,
                },
            ));
        }

        // 4. Activity: Foraging and returning food
        // Système de forage/récolte : la fourmi scrute activement pour du ravitaillement.
        let mut forage_attempts = Vec::new();
        for (ant_id, (pos, ant_data)) in self.world.query::<(&Position, &AntData)>().iter() {
            // Seules les ouvrières en recherche active d'aliments effectuent cette scrutation
            if ant_data.state == AntState::Exploring && ant_data.role == AntRole::Worker {
                // EXTRÊMEMENT RAPIDE : Grâce au Spatial Hash, on scanne les environs (Rayon de 10 unités) sans itérer sur l'ensemble de la map.
                for res_ent in self.spatial_grid.get_nearby(pos.x, pos.y, 10.0) {
                    // On enregistre une "tentative" de morsure
                    forage_attempts.push((ant_id, res_ent));
                }
            }
        }
        
        // On traite les tentatives de récolte collisées
        let mut ants_who_ate = Vec::new();
        for (ant_id, res_id) in forage_attempts {
            if let Ok(mut res) = self.world.get::<&mut ResourceData>(res_id) {
                // S'il reste de la nourriture matérielle disponible à l'instant T sur ce point...
                if res.quantity > 0 {
                    res.quantity -= 1; // La ressource au sol diminue d'un "morceau"
                    ants_who_ate.push(ant_id); // La fourmi est taggée "satisfaite" (elle porte la charge)
                }
            }
        }
        
        // On recherche et on marque toutes les entités de ressources totalement épuisées (= 0 morceaux restants)
        let mut resources_to_remove = Vec::new();
        for (res_id, res) in self.world.query::<&ResourceData>().iter() {
            if res.quantity == 0 {
                resources_to_remove.push(res_id);
            }
        }
        
        // Et on libère ces miettes périmées de la mémoire.
        for e in resources_to_remove {
            let _ = self.world.despawn(e);
        }
        
        // Pour toutes les ouvrières qui viennent de se saisir d'un morceau de ravitaillement...
        for ant_id in ants_who_ate {
            if let Ok(mut ant) = self.world.get::<&mut AntData>(ant_id) {
                // On passe brusquement leur cerveau en 'Retour à la base de rattachement'.
                ant.state = AntState::ReturningWithFood;
                // On fait mécaniquement faire un demi-tour à la fourmi (Angle actuel + Angle Plat PI radians, soit +180°)
                ant.angle += std::f32::consts::PI;
                // On s'assure que modulo 360° l'angle reste strict et cohérent au niveau trigonométrique.
                if ant.angle > std::f32::consts::TAU { ant.angle -= std::f32::consts::TAU; }
            }
        }

        // Move & Steering
        // Application de la physique basique et du "Steering Behavior" (Mouvement comportemental)
        for (_id, (pos, vel, ant)) in self.world.query_mut::<(&mut Position, &mut Velocity, &mut AntData)>() {
            // Seules les unités mobiles (qui ne sont pas des reines inertes) bougent
            if ant.role == AntRole::Worker || ant.role == AntRole::Soldier {
                // Point de ralliement par défaut au centre de la carte
                let mut hx = MAP_SIZE / 2.0;
                let mut hy = MAP_SIZE / 2.0;
                
                // On cherche les coordonnées du nid (précisément le grenier) associé à la faction de la fourmi
                if let Some(nest) = self.nests.iter().find(|n| n.faction_id == ant.faction_id) {
                    if let Some(granary) = nest.rooms.iter().find(|r| r.room_type == RoomType::Granary as i32) {
                        hx = granary.x;
                        hy = granary.y;
                    }
                }

                if ant.state == AntState::Exploring {
                    // Phéromone d'Exploration
                    let cx = (pos.x / SPATIAL_CELL_SIZE).floor() as i32;
                    let cy = (pos.y / SPATIAL_CELL_SIZE).floor() as i32;
                    let pheromone = self.pheromones_exploration.entry((cx, cy)).or_insert(0.0);
                    *pheromone = (*pheromone + 1.0).min(100.0);

                    // Détection de gradient phéromonal (3 capteurs quant à la nourriture)
                    let sensor_angle = 0.78; // ~45 degrés
                    let sensor_distance = 15.0; // Poussée du capteur (distance physique)

                    let mut get_ph_val = |angle: f32| -> f32 {
                        let sx = pos.x + angle.cos() * sensor_distance;
                        let sy = pos.y + angle.sin() * sensor_distance;
                        let scx = (sx / SPATIAL_CELL_SIZE).floor() as i32;
                        let scy = (sy / SPATIAL_CELL_SIZE).floor() as i32;
                        *self.pheromones_food.get(&(scx, scy)).unwrap_or(&0.0)
                    };

                    let val_center = get_ph_val(ant.angle);
                    let val_left = get_ph_val(ant.angle - sensor_angle);
                    let val_right = get_ph_val(ant.angle + sensor_angle);

                    // Seuil minimal pour considérer une piste valide (au-dessus du bruit de fond)
                    let threshold = 0.1;
                    
                    if val_center > threshold || val_left > threshold || val_right > threshold {
                        let turn_speed = 0.1; // Radian de pivot constant
                        if val_center > val_left && val_center > val_right {
                            // On va tout droit
                        } else if val_left > val_right {
                            ant.angle -= turn_speed; // Tourne à GAUCHE
                        } else if val_right > val_left {
                            ant.angle += turn_speed; // Tourne à DROITE
                        }
                    } else {
                        // Perdu la piste ou pas de piste au départ : Mouvement libre erratique ("Wander")
                        let wander_angle = rng.gen_range(-MAX_TURN_ANGLE..MAX_TURN_ANGLE);
                        ant.angle += wander_angle;
                    }

                    vel.vx = ant.angle.cos();
                    vel.vy = ant.angle.sin();
                } else if ant.state == AntState::ReturningWithFood {
                    // Phéromone de Nourriture (intensité +10.0, max 100.0)
                    let cx = (pos.x / SPATIAL_CELL_SIZE).floor() as i32;
                    let cy = (pos.y / SPATIAL_CELL_SIZE).floor() as i32;
                    let pheromone = self.pheromones_food.entry((cx, cy)).or_insert(0.0);
                    *pheromone = (*pheromone + 10.0).min(100.0);

                    // Calcul Vectoriel (Distance) : Vecteur du Point de Destination (Nid) - Vecteur de notre position actuelle
                    let dx = hx - pos.x;
                    let dy = hy - pos.y;
                    let distsq = dx * dx + dy * dy;

                    // Si on est à moins de ~40 unités du nid (sqrt 1600 = 40)
                    if distsq < 1600.0 { 
                        // On "dépose" magiquement la nourriture, la fourmi redevient exploratrice
                        ant.state = AntState::Exploring;
                        // On augmente directement le score global de la colonie
                        if let Some(stats) = self.factions.get_mut(&ant.faction_id) {
                            stats.proteins += 1;
                        }
                        // La fourmi fait demi-tour instantanément pour retourner chercher à manger
                        ant.angle += std::f32::consts::PI;
                        vel.vx = ant.angle.cos();
                        vel.vy = ant.angle.sin();
                    } else {
                        // Sinon, l'angle cible est de se diriger en ligne droite vers la base
                        let target_angle = dy.atan2(dx);
                        vel.vx = target_angle.cos();
                        vel.vy = target_angle.sin();
                    }
                }
                
                // Normalisation : On s'assure que le vecteur vitesse a toujours une base uniforme de "1" de magnitude.
                let len = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
                if len > 0.0001 {
                    // On multiplie alors ce vecteur unitaire de 1 par notre vitesse calculée sur ce tick.
                    vel.vx = (vel.vx / len) * ANT_SPEED_PER_TICK;
                    vel.vy = (vel.vy / len) * ANT_SPEED_PER_TICK;
                }
                
                // Application de la vélocité sur la configuration spatiale avec vérification du terrain
                let next_x = pos.x + vel.vx;
                let next_y = pos.y + vel.vy;

                let terrain = get_terrain_type(next_x, next_y, self.map_seed);
                if terrain == TerrainType::Water || terrain == TerrainType::Rock {
                    // Rebond naturel sur l'obstacle
                    vel.vx *= -1.0;
                    vel.vy *= -1.0;
                    ant.angle += std::f32::consts::PI;
                } else {
                    pos.x = next_x;
                    pos.y = next_y;
                }

                // Si la fourmi atteint le bord gauche (0) ou droit (MAP_SIZE)
                if pos.x <= 0.0 || pos.x >= MAP_SIZE {
                    // Rebord (Bounce) inversé sur X
                    vel.vx *= -1.0;
                    pos.x = pos.x.clamp(0.0, MAP_SIZE);
                }
                // Si la fourmi atteint le bord haut (0) ou bas (MAP_SIZE)
                if pos.y <= 0.0 || pos.y >= MAP_SIZE {
                    // Rebord (Bounce) inversé sur Y
                    vel.vy *= -1.0;
                    pos.y = pos.y.clamp(0.0, MAP_SIZE);
                }

                // On écrase la donnée "angle physique" de la fourmi avec le résultat final (pour l'affichage orienté)
                ant.angle = vel.vy.atan2(vel.vx);
            }
        }
    }

    // Fonction génératrice de l'État du Jeu utilisé par l'API réseau gRPC
    // Exécutée à chaque demande d'interface ou dans les boucles de flux Server Streaming
    pub fn get_game_state(&self) -> GameState {
        // Collecte toutes les fourmis
        let mut out_ants = Vec::new();
        // Une requête ECS (sans mut car pas de modification pure)
        for (_, (pos, ant_data)) in self.world.query::<(&Position, &AntData)>().iter() {
            out_ants.push(Ant {
                id: ant_data.id,
                // On cast l'enum Rust `AntRole` en Entier i32 pour le flux Protobuf (Standard)
                role: ant_data.role as i32,
                x: pos.x,
                y: pos.y,
                // Idem, on cast `AntState` de l'enum Rust
                state: ant_data.state as i32,
                angle: ant_data.angle,
                faction_id: ant_data.faction_id,
            });
        }

        // Collecte toutes les ressources à l'instant T
        let mut out_res = Vec::new();
        // Le `res_id` est arbitraire ici car une ressource disparaît rapidement et n'a pas besoin de persistance stricte
        let mut res_id = 1;
        for (_, (pos, res)) in self.world.query::<(&Position, &ResourceData)>().iter() {
            out_res.push(Resource {
                id: res_id,
                x: pos.x,
                y: pos.y,
                quantity: res.quantity as u32,
                r#type: res.res_type as i32, // `r#type` car `type` est un mot clé réservé en Rust
            });
            res_id += 1;
        }

        // Renvoie l'objet final GameState formatté pour GRPC
        GameState {
            tick: self.tick,                             // Temps serveur
            ants: out_ants,                              // Tableau consolidé de toutes les fourmis
            resources: out_res,                          // Tableau de sources de ressources
            nests: self.nests.clone(),                   // Clône rapide du vecteur contenant la configuration géométrique
            encoded_pheromones: vec![], // Omit for bandwidth (optimisation réseau, on les omet pour le moment)
            map_seed: self.map_seed as u64,              // Graine de génération de la map procédurale
        }
    }
}
