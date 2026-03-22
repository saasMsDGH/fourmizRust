# Algorithme de Mouvement : Correlated Random Walk (Wander)

**Contexte :** Afin d'éviter le "jitter" (tremblement) des entités dû à un aléatoire pur à chaque tick (Mouvement Brownien), le système de déplacement des ouvrières en état `WANDER` doit utiliser une persistance directionnelle.

**Implémentation requise (Rust / hecs) :**

1. **Composant `Velocity` ou `Transform` :**
   Assurez-vous que l'entité possède un champ `current_angle` (en radians, f32) et une `speed` (f32).

2. **Paramètres de l'algorithme :**
   Définissez une constante `MAX_TURN_ANGLE` (ex: 0.2 radians, soit environ 11 degrés). C'est la variation maximale autorisée par tick.

3. **Logique dans la boucle système (à 100 TPS) :**
   Pour chaque entité en état `WANDER` :
   
   * Étape A : Générer un flottant aléatoire entre `-MAX_TURN_ANGLE` et `+MAX_TURN_ANGLE`.
   * Étape B : Ajouter cette valeur à `current_angle`.
   * Étape C : Calculer le vecteur de déplacement :
     `dx = current_angle.cos() * speed`
     `dy = current_angle.sin() * speed`
   * Étape D : Mettre à jour la position (`x += dx`, `y += dy`).

4. **Gestion des bordures de carte (Bouncing) :**
   Si la position `x` ou `y` calculée sort des limites de la carte (ex: 0 à 10000) :
   * Inverser l'angle de réflexion ou ajouter `std::f32::consts::PI` à `current_angle` pour faire faire un demi-tour fluide à la fourmi.
   * Borner les coordonnées pour éviter qu'elle ne s'échappe.