# Algorithme de Suivi de Piste (Gradient Phéromonal par Capteurs)

**Contexte :** Pour qu'une fourmi en état `WANDER` suive une piste de manière organique (sans se téléporter ni bloquer dans des boucles infinies), elle ne doit pas regarder toutes les cases autour d'elle. Elle doit simuler 3 antennes (capteurs) placées *devant* elle.

**Implémentation requise (Rust / hecs) :**

1. **Géométrie des Capteurs :**
   Définissez 3 points de détection devant la fourmi, basés sur son `current_angle` et sa position $x, y$.
   * `SENSOR_ANGLE` = $\pm 45^\circ$ (environ 0.78 radians).
   * `SENSOR_DISTANCE` = L'équivalent d'une ou deux cases de la grille de phéromones.
   
   Calcul des 3 positions continues :
   * **Capteur Centre :** $x + \cos(\text{angle}) \times \text{distance}$, $y + \sin(\text{angle}) \times \text{distance}$
   * **Capteur Gauche :** $x + \cos(\text{angle} - \text{SENSOR\_ANGLE}) \times \text{distance}$, $y + \sin(\text{angle} - \text{SENSOR\_ANGLE}) \times \text{distance}$
   * **Capteur Droit :** $x + \cos(\text{angle} + \text{SENSOR\_ANGLE}) \times \text{distance}$, $y + \sin(\text{angle} + \text{SENSOR\_ANGLE}) \times \text{distance}$

2. **Échantillonnage de la Grille (Sampling) :**
   * Convertissez les coordonnées de ces 3 capteurs continus en indices discrets $(i, j)$ pour lire la valeur dans la `PheromoneGrid`.
   * Obtenez 3 valeurs : `val_gauche`, `val_centre`, `val_droite`.

3. **Logique de Décision (Steering) :**
   Appliquez les règles d'orientation suivantes. `TURN_SPEED` doit être défini pour tourner doucement (ex: 0.1 rad/tick).
   
   * Si `val_centre` > `val_gauche` ET `val_centre` > `val_droite` :
     La piste est droit devant. On ne change pas l'angle.
   * Sinon, si `val_gauche` > `val_droite` :
     La piste tourne à gauche. `current_angle -= TURN_SPEED`
   * Sinon, si `val_droite` > `val_gauche` :
     La piste tourne à droite. `current_angle += TURN_SPEED`
   * Si toutes les valeurs sont à 0 (ou sous un seuil critique de détection) :
     La fourmi a perdu la piste. Elle exécute l'algorithme classique du `WANDER` (Correlated Random Walk).

4. **Optimisation CPU :**
   Assurez-vous que cette vérification des 3 capteurs ne se fait pas si la fourmi est simplement en train de ramener de la nourriture au nid (état `RETURN_TO_NEST`), auquel cas elle calcule juste le vecteur mathématique direct vers les coordonnées de sa colonie de départ.