# Algorithme de Génération Procédurale (Simplex / Perlin Noise)

**Contexte :** Pour éviter de stocker en mémoire RAM une grille géante de 10km² contenant les obstacles, le serveur Rust et le client PixiJS utiliseront la même fonction mathématique continue (Bruit de Perlin ou Simplex Noise) pour savoir ce qui se trouve à une coordonnée (x, y) précise.

**Implémentation requise (Rust) :**

1. **La Graine (Seed) :**
   Générez un `seed` aléatoire au démarrage de l'application (ex: `u32` ou `u64`) et stockez-le dans une ressource globale `WorldSettings`.

2. **La Fonction d'Échantillonnage (Sampling) :**
   Utilisez une librairie comme la crate `noise` en Rust (ex: `OpenSimplex` ou `Fbm`).
   Créez une fonction pure : `fn get_terrain_type(x: f32, y: f32, seed: u32) -> TerrainType`.
   
   *Astuce : Multipliez `x` et `y` par une constante d'échelle (ex: `0.005`) avant de les passer à la fonction de bruit pour obtenir des "taches" organiques lisses et non du bruit blanc.*

3. **Seuils de Terrain (Thresholds) :**
   La fonction de bruit retourne une valeur entre -1.0 et 1.0. Définissez des tranches strictes :
   * De `-1.0` à `-0.3` : `WATER` (Obstacle infranchissable, les fourmis se noient ou contournent).
   * De `-0.3` à `0.6` : `DIRT` (Terrain praticable standard).
   * De `0.6` à `1.0` : `ROCK` (Obstacle infranchissable, bloque aussi la diffusion des phéromones).

4. **Synchronisation Frontend :**
   Le Frontend implémentera la même logique (via un package npm comme `simplex-noise`) en utilisant le même `seed` et les mêmes seuils pour peindre le fond de la carte sous les entités. Zéro données de terrain ne doivent transiter à 60 FPS, seul le `seed` est envoyé au début.

5. **Impact Météorologique (Pluie) :**
   Si l'état global de la météo passe à `RAIN` (Pluie), en plus d'accélérer l'évaporation des phéromones, l'algorithme peut virtuellement décaler le seuil de l'eau (ex: l'eau passe de `-0.3` à `-0.1`), créant une montée des eaux dynamique in-game bloquant certains chemins.