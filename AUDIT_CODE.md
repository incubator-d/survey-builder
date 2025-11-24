# Audit du Code - Liste des Problèmes par Priorité

## 🔴 PRIORITÉ CRITIQUE (Bloquant / Impact majeur)

### 1. Violation du Principe de Responsabilité Unique (SRP)
**Fichier**: `src/lib.rs` (1184 lignes)
- **Problème**: `AppState` concentre trop de responsabilités :
  - Gestion d'état (state management)
  - Logique métier (création d'audit, sauvegarde)
  - Rendu UI (toutes les fonctions `render_*`)
  - Gestion des événements clavier (toutes les fonctions `handle_*`)
  - Navigation entre vues
- **Impact**: Code difficile à maintenir, tester et faire évoluer
- **Solution**: Séparer en modules distincts :
  - `AppState`: uniquement l'état
  - `EventHandler`: gestion des événements
  - `ViewRenderer`: rendu des vues
  - `AuditService`: logique métier

### 2. Code Mort et Attributs Inutilisés
**Fichiers**: `src/lib.rs`, `src/recommendations.rs`, `src/audit.rs`
- **Problème**: 
  - `__error_message` dans `AppState` jamais utilisé (ligne 186, 261, 394, 589)
  - `InputMode::Answer`, `Percentage`, `Comment` marqués `#[allow(dead_code)]`
  - `View::Recommendations` marqué `#[allow(dead_code)]` mais utilisé
  - Fonction `AppState::new()` jamais utilisée (utilise `Default`)
- **Impact**: Confusion, maintenance difficile, code inutile
- **Solution**: Supprimer le code mort ou l'utiliser correctement

### 3. Gestion d'Erreur Inconsistante
**Fichiers**: `src/lib.rs`, `src/storage.rs`
- **Problème**:
  - Erreurs silencieusement ignorées dans `load_audits()` (ligne 245-247)
  - `unwrap_or()` masque les erreurs dans `storage.rs` (lignes 62, 65, 101, 104)
  - `__error_message` stocké mais jamais affiché à l'utilisateur
- **Impact**: Erreurs non visibles, debugging difficile
- **Solution**: Implémenter un système d'erreur cohérent avec affichage utilisateur

### 4. Clonage Inutile de Données
**Fichier**: `src/lib.rs` (ligne 577)
- **Problème**: `let mut audit = audit.clone();` dans `save_answer()`
- **Impact**: Performance dégradée, consommation mémoire inutile
- **Solution**: Utiliser `Rc<RefCell<Audit>>` ou passer une référence mutable

---

## 🟠 PRIORITÉ ÉLEVÉE (Impact significatif sur la qualité)

### 5. Duplication de Code
**Fichier**: `src/lib.rs`
- **Problème**:
  - Calcul du score global dupliqué dans `render_list()` (lignes 637-652) et potentiellement ailleurs
  - Logique de navigation répétée dans plusieurs `handle_*`
  - Mapping de catégories en dur dans `audit.rs` (lignes 98-110)
- **Impact**: Maintenance difficile, risque d'incohérences
- **Solution**: Extraire en fonctions utilitaires ou constantes

### 6. Hardcoding et Manque de Configuration
**Fichiers**: `src/audit.rs`, `src/recommendations.rs`
- **Problème**:
  - Chemins de fichiers YAML hardcodés (lignes 142-152)
  - Seuils de recommandation hardcodés (50%, 25%, 40% dans `recommendations.rs`)
  - Mapping catégorie → priorité hardcodé (lignes 82-90)
  - Questions business hardcodées au lieu d'utiliser YAML
- **Impact**: Impossible de personnaliser sans modifier le code
- **Solution**: Utiliser un fichier de configuration ou des constantes configurables

### 7. Violation du Principe Ouvert/Fermé (OCP)
**Fichier**: `src/audit.rs`
- **Problème**: 
  - `QuestionGenerator::load_questions_from_yaml()` avec `match` sur les chemins (lignes 66-77)
  - Impossible d'ajouter de nouveaux types de questions sans modifier le code
- **Impact**: Extension difficile, violation OCP
- **Solution**: Utiliser un système de plugins ou un registre de chargeurs

### 8. Couplage Fort avec WebStorage
**Fichier**: `src/audit.rs`
- **Problème**: `AuditManager` est juste un wrapper autour de `WebStorage`
- **Impact**: Impossible de changer de backend de stockage facilement
- **Solution**: Introduire un trait `Storage` avec implémentations multiples

### 9. Manque de Validation des Données
**Fichiers**: `src/models.rs`, `src/lib.rs`
- **Problème**:
  - Pas de validation sur `Audit::new()` (nom vide, etc.)
  - Pas de validation sur `answer_percentage` (doit être 0-100)
  - Parsing de `answer_value` avec `return` silencieux si invalide (ligne 562)
- **Impact**: Données invalides possibles, comportement imprévisible
- **Solution**: Ajouter des validations avec types `Result` ou `Option`

---

## 🟡 PRIORITÉ MOYENNE (Amélioration de la qualité)

### 10. Manque d'Abstraction pour les Types d'Audit
**Fichier**: `src/lib.rs` (lignes 520-536)
- **Problème**: `match` sur `AuditType` avec logique métier dans la couche UI
- **Impact**: Logique métier dans la présentation
- **Solution**: Utiliser le pattern Strategy ou Factory pour la génération de questions

### 11. Code de Rendu Trop Long
**Fichier**: `src/lib.rs`
- **Problème**: 
  - `render_recommendations()` fait 237 lignes (lignes 919-1156)
  - `render_list()` fait 113 lignes (lignes 611-724)
  - Logique de calcul mélangée avec le rendu
- **Impact**: Difficulté à maintenir et tester
- **Solution**: Extraire la logique de calcul dans des fonctions séparées

### 12. Magic Numbers et Strings
**Fichiers**: `src/lib.rs`, `src/recommendations.rs`
- **Problème**:
  - Valeurs magiques: `75.0`, `50.0`, `25.0` pour les scores (lignes 669-672, 1001-1009)
  - Strings magiques: `"oui"`, `"non"`, `"jsp"` (lignes 558-561)
  - `3` pour le scroll (lignes 327, 373)
- **Impact**: Code difficile à comprendre et maintenir
- **Solution**: Définir des constantes nommées

### 13. Manque de Documentation
**Fichiers**: Tous
- **Problème**: 
  - Peu de documentation Rust (`///`)
  - Pas de documentation sur les algorithmes complexes
  - Pas de README technique
- **Impact**: Onboarding difficile, maintenance compliquée
- **Solution**: Ajouter de la documentation Rust et un guide d'architecture

### 14. Tests Insuffisants pour la Couche UI
**Fichier**: `src/lib.rs`
- **Problème**: Aucun test pour `AppState` et les fonctions de rendu
- **Impact**: Risque de régression dans l'UI
- **Solution**: Ajouter des tests d'intégration pour les interactions utilisateur

### 15. Gestion d'État Fragile
**Fichier**: `src/lib.rs`
- **Problème**:
  - Index de sélection peut être invalide (lignes 256-258, 397-399)
  - Pas de validation de cohérence d'état
- **Impact**: Bugs potentiels (panics, comportement étrange)
- **Solution**: Ajouter des invariants et des validations d'état

---

## 🟢 PRIORITÉ FAIBLE (Améliorations mineures)

### 16. Noms de Variables Ambiguës
**Fichier**: `src/lib.rs`
- **Problème**: 
  - `c` pour caractère (ligne 52)
  - `a`, `b` dans les comparaisons (ligne 250)
  - `f` pour Frame (ligne 602)
- **Impact**: Lisibilité réduite
- **Solution**: Utiliser des noms plus descriptifs

### 17. Manque de Builder Pattern
**Fichier**: `src/models.rs`
- **Problème**: `Audit::new()` avec beaucoup de champs optionnels
- **Impact**: Création d'objets verbeuse
- **Solution**: Implémenter un builder pattern

### 18. Pas de Trait pour les Types Affichables
**Fichier**: `src/models.rs`
- **Problème**: Beaucoup de `impl Display` répétitifs
- **Impact**: Code répétitif
- **Solution**: Créer un macro ou un trait générique

### 19. Performance: Calculs Redondants
**Fichier**: `src/lib.rs`
- **Problème**: 
  - `completion_percentage()` appelé plusieurs fois pour le même audit
  - `generate_recommendations()` et `generate_summary()` appelés à chaque frame
- **Impact**: Performance dégradée
- **Solution**: Mettre en cache les résultats ou recalculer seulement quand nécessaire

### 20. Manque de Types pour les Identifiants
**Fichier**: Partout
- **Problème**: Utilisation directe de `Uuid` partout
- **Impact**: Risque de confusion entre différents types d'IDs
- **Solution**: Créer des types wrapper (`AuditId`, `QuestionId`)

---

## 📊 Résumé par Catégorie

### Principes SOLID Violés
- ❌ **SRP**: `AppState` a trop de responsabilités
- ❌ **OCP**: Impossible d'étendre sans modifier (QuestionGenerator)
- ⚠️ **DIP**: Couplage fort avec WebStorage

### Code Smells
- 🔴 God Object (`AppState`)
- 🔴 Code Duplication
- 🔴 Magic Numbers/Strings
- 🟠 Long Methods
- 🟠 Dead Code

### Qualité
- 🔴 Gestion d'erreur inconsistante
- 🟠 Manque de validation
- 🟠 Manque de documentation
- 🟡 Tests UI manquants

### Performance
- 🟠 Clonage inutile
- 🟡 Calculs redondants

---

## 🎯 Plan d'Action Recommandé

1. **Phase 1 (Critique)**: Refactoriser `AppState` en modules séparés
2. **Phase 2 (Élevée)**: Implémenter un système de configuration et un trait Storage
3. **Phase 3 (Moyenne)**: Améliorer la documentation et ajouter des tests UI
4. **Phase 4 (Faible)**: Optimisations et améliorations mineures

