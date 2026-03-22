# Algorithme d'Optimisation : Spatial Hash Grid

**Contexte :** Pour éviter une complexité quadratique $O(N^2)$ lors de la recherche de voisins (collisions, détection de nourriture, interactions), l'espace 2D continu de la carte de 10km² doit être divisé en "seaux" (buckets) discrets.

**Implémentation requise (Rust) :**

1. **La Structure de Données :**
   Créez une `struct SpatialHashGrid`.
   * **Paramètre :** `CELL_SIZE` (ex: 50.0 unités). Ce doit être légèrement plus grand que le rayon de vision maximum d'une fourmi.
   * **Stockage :** Utilisez une `HashMap<(i32, i32), Vec<Entity>>` (ex: via `rustc-hash` pour des performances optimales sans cryptographie, le FNV hasher est plus rapide que le SipHash par défaut de Rust).

2. **Fonction de Hachage (Position vers Index) :**
   Implémentez une méthode `fn get_cell_coords(x: f32, y: f32) -> (i32, i32)` :
   * `cell_x = (x / CELL_SIZE).floor() as i32;`
   * `cell_y = (y / CELL_SIZE).floor() as i32;`

3. **Mise à jour à chaque Tick (Clear & Populate) :**
   Plutôt que de traquer les mouvements individuels (ce qui est lourd), il est souvent plus rapide en Rust de vider la `HashMap` via `.clear()` au début du tick (ce qui conserve la capacité mémoire allouée), puis d'itérer sur toutes les entités ayant une `Position` pour faire un `.push(entity)` dans la bonne cellule.

4. **Requête de Voisinage (Query Nearby) :**
   Implémentez une méthode `fn query_in_radius(&self, x: f32, y: f32, radius: f32) -> Vec<Entity>` :
   * Calculez la Bounding Box de la requête :
     `min_cell = get_cell_coords(x - radius, y - radius)`
     `max_cell = get_cell_coords(x + radius, y + radius)`
   * Itérez uniquement sur les cellules comprises entre `min_cell` et `max_cell` (soit 4 à 9 cellules maximum).
   * Récupérez les entités de ces cellules.
   * (Optionnel mais recommandé) : Filtrez ces entités avec un vrai calcul de distance au carré `(dx*dx + dy*dy <= radius*radius)` pour éliminer les faux positifs dans les coins des cellules.