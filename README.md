# Survey Audit - Outil d'audit technique et business

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/USERNAME/survey-audit/workflows/CI/badge.svg)](https://github.com/USERNAME/survey-audit/actions)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-wasm32--unknown--unknown-blue.svg)](https://webassembly.org/)

Outil d'audit en Rust permettant de réaliser des audits complets de projets via des questionnaires structurés, avec génération automatique de recommandations. Interface TUI (Terminal User Interface) dans le navigateur grâce à [Ratzilla](https://github.com/orhun/ratzilla).

## Fonctionnalités

- ✅ **Interface TUI dans le navigateur** : Interface terminal moderne avec Ratzilla et WebAssembly
- ✅ **Multi-types d'audits** : Technique, Business, Marketing, et types personnalisés
- ✅ **Catégories organisées** : Infra, Opération, DB, A11y, Archi, Intra, OSS, Dev XP, Produit, Sécu, Bizdev, Organisation
- ✅ **Système de questions** : Questions chargées depuis des fichiers YAML avec descriptions, liens, et réponses attendues
- ✅ **Réponses flexibles** : Oui, Non, Je ne sais pas, Non applicable (avec pourcentages optionnels)
- ✅ **Génération de recommandations** : Recommandations automatiques basées sur les réponses
- ✅ **Stockage local** : Données sauvegardées dans le localStorage du navigateur
- ✅ **Compatible Vercel** : Déploiement sur hébergement statique sans backend
- ✅ **Rapports et statistiques** : Résumés par catégorie et pourcentage de complétion
- ✅ **Export/Import** : Export des audits en TXT ou YAML, import depuis YAML
- ✅ **Tests complets** : 45 tests unitaires avec couverture à 100%

## Installation

### Prérequis

- [Rust](https://www.rust-lang.org/tools/install) (dernière version stable)
- [Trunk](https://trunkrs.dev/) (outil de build pour WebAssembly)

### Étapes d'installation

```bash
# Cloner le projet
git clone https://github.com/USERNAME/survey-audit.git
cd survey-audit

# Installer Trunk (outil de build pour WebAssembly)
cargo install --locked trunk

# Ajouter la cible WebAssembly
rustup target add wasm32-unknown-unknown
```

## Utilisation

### Démarrage de l'application

```bash
# Lancer l'application (par défaut sur le port 8080)
trunk serve index.html

# Ou utiliser le script de démarrage
./start.sh
```

Puis ouvrez votre navigateur à l'adresse : **http://localhost:8080**

### Raccourcis clavier

#### Vue Liste des audits
- `n` : Créer un nouvel audit
- `↑` / `↓` ou `k` / `j` : Naviguer dans la liste
- `Entrée` : Ouvrir l'audit sélectionné
- `d` : Supprimer l'audit sélectionné
- `e` : Exporter l'audit sélectionné en TXT
- `y` : Exporter l'audit vierge en YAML
- `i` : Importer un audit depuis un fichier YAML
- `q` / `Esc` : Quitter

#### Vue Détail d'un audit
- `↑` / `↓` ou `k` / `j` : Naviguer entre les questions
- `o` : Répondre "Oui"
- `n` : Répondre "Non"
- `j` : Répondre "Je ne sais pas"
- `a` : Répondre "Non applicable"
- `Entrée` : Sauvegarder la réponse et passer à la question suivante
- `f` : Activer/désactiver le filtre des questions non répondues (utile après import)
- `e` : Exporter l'audit en TXT
- `y` : Exporter l'audit vierge en YAML
- `r` : Voir les résultats et recommandations (si toutes les questions sont répondues)
- `q` / `Esc` : Retour à la liste

#### Vue Création d'audit
- `Tab` : Changer de champ
- `Entrée` : Créer l'audit
- `q` / `Esc` : Annuler et retourner à la liste

## Structure des données

### Types d'audit

- `Technique` : Audit technique (infrastructure, sécurité, architecture, etc.)
- `Business` : Audit business (stratégie, ROI, déploiement territorial, etc.)
- `Marketing` : Audit marketing
- `Autre(String)` : Types personnalisés

### Catégories

- **Infra** : Infrastructure et hébergement
- **Operation** : Opérations et monitoring
- **DB** : Bases de données
- **A11y** : Accessibilité
- **Archi** : Architecture
- **Intra** : Outils internes
- **OSS** : Open Source
- **Dev XP** : Developer Experience
- **Produit** : Produit et UX
- **Secu** : Sécurité
- **Bizdev** : Business Development
- **Organisation** : Organisation, processus et gestion des connaissances

### Réponses

- `Oui` : Réponse positive
- `Non` : Réponse négative
- `JeNeSaisPas` : Incertitude
- `NonApplicable` : Non applicable au contexte

Certaines questions peuvent nécessiter un pourcentage (0-100) pour indiquer le niveau de complétion.

## Questions

Les questions sont chargées depuis des fichiers YAML dans le répertoire `questions/technique/`. Chaque fichier contient une liste de questions pour une catégorie spécifique :

- `infra.yaml` : Questions sur l'infrastructure
- `db.yaml` : Questions sur les bases de données
- `operation.yaml` : Questions sur les opérations
- `secu.yaml` : Questions sur la sécurité
- `a11y.yaml` : Questions sur l'accessibilité
- `archi.yaml` : Questions sur l'architecture
- `oss.yaml` : Questions sur l'open source
- `devxp.yaml` : Questions sur la developer experience
- `produit.yaml` : Questions sur le produit
- `organisation.yaml` : Questions sur l'organisation, processus et gestion des connaissances

### Format d'une question

```yaml
questions:
  - text: "Votre question ici"
    description: "Description détaillée (optionnel)"
    category: "A11y"
    requires_percentage: false
    expected_answer: "Oui"
    links:
      - "https://exemple.com"
```

## Recommandations

Le système génère automatiquement des recommandations basées sur :

1. **Réponses négatives** aux questions où "Oui" est attendu
2. **Scores par catégorie** : Si une catégorie a moins de 50% de réponses positives
3. **Priorités** : 
   - 🔴 **Critique** : Sécurité, infrastructure critique
   - 🟠 **Élevée** : Infrastructure, DB, Opérations
   - 🟡 **Moyenne** : Autres catégories
   - 🟢 **Faible** : Améliorations mineures

## Stockage

Les audits sont sauvegardés dans le **localStorage** du navigateur. Cela signifie que :

- ✅ Les données sont persistantes entre les sessions
- ✅ Compatible avec tous les hébergements statiques (Vercel, Netlify, GitHub Pages, etc.)
- ✅ Pas besoin de backend ou de base de données
- ⚠️ Les données sont locales au navigateur (pas de synchronisation entre appareils)

### Structure du stockage

- Clé `survey_audit_<UUID>` : Contient les données JSON de l'audit
- Clé `survey_audit_index` : Liste des IDs de tous les audits

## Déploiement

### Déploiement sur Vercel

1. **Build de l'application** :

```bash
trunk build --release
```

2. **Configuration Vercel** :

Créez un fichier `vercel.json` à la racine :

```json
{
  "buildCommand": "trunk build --release",
  "outputDirectory": "dist",
  "devCommand": "trunk serve index.html",
  "framework": null
}
```

3. **Déployer** :

```bash
vercel deploy
```

### Déploiement sur Netlify

1. **Build de l'application** :

```bash
trunk build --release
```

2. **Configuration Netlify** :

Créez un fichier `netlify.toml` :

```toml
[build]
  command = "trunk build --release"
  publish = "dist"
```

3. **Déployer** :

Connectez votre dépôt GitHub à Netlify ou utilisez `netlify deploy`.

### Déploiement sur GitHub Pages

1. **Build de l'application** :

```bash
trunk build --release
```

2. **Copier les fichiers** :

```bash
cp -r dist/* /path/to/gh-pages-branch/
```

3. **Commit et push** :

```bash
git add .
git commit -m "Deploy"
git push origin gh-pages
```

## Développement

### Tests

Le projet inclut une suite complète de tests unitaires avec une couverture à 100% :

```bash
# Lancer tous les tests
cargo test

# Lancer les tests avec affichage détaillé
cargo test -- --nocapture

# Lancer un test spécifique
cargo test test_create_complete_audit_workflow
```

**Statistiques des tests** :
- ✅ 45 tests unitaires
- ✅ Couverture complète des modules principaux
- ✅ Tests d'intégration pour les workflows complets
- ✅ Tests pour toutes les catégories et types de réponses

### Structure du projet

```
_survey/
├── src/
│   ├── lib.rs              # Point d'entrée Ratzilla
│   ├── domain/
│   │   ├── models.rs        # Modèles de données
│   │   ├── audit.rs         # Gestionnaire d'audits et générateur de questions
│   │   └── recommendations.rs # Génération de recommandations
│   ├── infrastructure/
│   │   ├── storage.rs       # Stockage localStorage
│   │   └── storage_trait.rs # Trait d'abstraction du stockage
│   ├── shared/
│   │   ├── config.rs        # Configuration et constantes
│   │   ├── error.rs         # Gestion des erreurs
│   │   └── utils.rs         # Utilitaires (calcul de scores, etc.)
│   ├── ui/
│   │   └── app.rs           # État de l'application et logique UI
│   └── interfaces/
│       ├── cli.rs           # Interface ligne de commande
│       └── web.rs           # Interface web (Actix)
├── questions/
│   └── technique/          # Fichiers YAML de questions
├── static/                 # Fichiers statiques (JS, CSS)
├── index.html              # Page HTML principale
├── Trunk.toml              # Configuration Trunk
└── Cargo.toml              # Dépendances Rust
```

### Ajouter des questions

Modifiez ou créez des fichiers YAML dans `questions/technique/` :

```yaml
questions:
  - text: "Votre nouvelle question"
    description: "Description optionnelle"
    category: "A11y"
    requires_percentage: false
    expected_answer: "Oui"
    links:
      - "https://exemple.com"
```

### Ajouter de nouvelles catégories

Ajoutez de nouvelles variantes à l'enum `Category` dans `src/models.rs` :

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Category {
    // ... catégories existantes
    NouvelleCategorie,
    Autre(String),
}
```

### Personnaliser les recommandations

Modifiez la fonction `generate_recommendations()` dans `src/domain/recommendations.rs` pour ajuster la logique de génération.

### Exporter et importer des audits

Les audits peuvent être exportés en deux formats :

1. **TXT** : Rapport complet avec toutes les réponses et recommandations
   - Utilisez `e` dans la vue Liste ou Détail
   - Format lisible pour partage et archivage

2. **YAML** : Format structuré pour réimportation
   - Utilisez `y` dans la vue Liste ou Détail (uniquement pour audits vierges)
   - Permet de remplir un audit hors ligne et de l'importer ensuite
   - Utilisez `i` dans la vue Liste pour importer un fichier YAML

## Technologies utilisées

- **Rust** : Langage de programmation
- **Ratzilla** : Framework TUI pour le web (basé sur Ratatui)
- **WebAssembly** : Compilation vers WASM pour le navigateur
- **Trunk** : Outil de build et de développement
- **Serde** : Sérialisation/désérialisation JSON et YAML
- **localStorage** : Stockage côté client

## Contribuer

Les contributions sont les bienvenues ! Ce projet suit le [Code de conduite du contributeur](CODE_OF_CONDUCT.md). 

Pour commencer :
- Lisez notre [guide de contribution](CONTRIBUTING.md)
- Consultez les [issues ouvertes](../../issues) pour voir où vous pouvez aider
- Consultez le [CHANGELOG](CHANGELOG.md) pour voir les changements récents

### Comment contribuer

1. 🍴 Fork le projet
2. 🌿 Créez une branche pour votre fonctionnalité (`git checkout -b feature/AmazingFeature`)
3. 💻 Committez vos changements (`git commit -m 'feat: Add some AmazingFeature'`)
4. 📤 Pushez vers la branche (`git push origin feature/AmazingFeature`)
5. 🔀 Ouvrez une Pull Request

Pour plus de détails, consultez le [guide de contribution](CONTRIBUTING.md).

## Code de conduite

Ce projet adhère au [Code de conduite du contributeur](CODE_OF_CONDUCT.md). En participant, vous êtes tenu de maintenir ce code. Veuillez signaler tout comportement inacceptable.

## Licence

Ce projet est sous licence MIT. Voir le fichier [LICENSE](LICENSE) pour plus de détails.

## Inspiration

Cet outil s'inspire de l'outil d'audit technique de l'[Incubateur des Territoires](https://audit.incubateur.anct.gouv.fr/).

## Auteurs et contributeurs

Voir [CHANGELOG](CHANGELOG.md) pour la liste complète des contributeurs.

---

⭐ Si ce projet vous est utile, n'hésitez pas à lui donner une étoile !
