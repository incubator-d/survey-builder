# Survey Audit - Outil d'audit technique et business

Outil d'audit en Rust permettant de réaliser des audits complets de projets via des questionnaires structurés, avec génération automatique de recommandations. Interface TUI (Terminal User Interface) dans le navigateur grâce à [Ratzilla](https://github.com/orhun/ratzilla).

## Fonctionnalités

- ✅ **Interface TUI dans le navigateur** : Interface terminal moderne avec Ratzilla et WebAssembly
- ✅ **Multi-types d'audits** : Technique, Business, Marketing, et types personnalisés
- ✅ **Catégories organisées** : Infra, Opération, DB, A11y, Archi, Intra, OSS, Dev XP, Produit, Sécu, Bizdev
- ✅ **Système de questions** : Questions chargées depuis des fichiers YAML avec descriptions, liens, et réponses attendues
- ✅ **Réponses flexibles** : Oui, Non, Je ne sais pas, Non applicable (avec pourcentages optionnels)
- ✅ **Génération de recommandations** : Recommandations automatiques basées sur les réponses
- ✅ **Stockage local** : Données sauvegardées dans le localStorage du navigateur
- ✅ **Compatible Vercel** : Déploiement sur hébergement statique sans backend
- ✅ **Rapports et statistiques** : Résumés par catégorie et pourcentage de complétion

## Installation

### Prérequis

- [Rust](https://www.rust-lang.org/tools/install) (dernière version stable)
- [Trunk](https://trunkrs.dev/) (outil de build pour WebAssembly)

### Étapes d'installation

```bash
# Cloner le projet
git clone <url-du-repo>
cd _survey

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
- `q` / `Esc` : Quitter

#### Vue Détail d'un audit
- `↑` / `↓` ou `k` / `j` : Naviguer entre les questions
- `o` : Répondre "Oui"
- `n` : Répondre "Non"
- `jsp` : Répondre "Je ne sais pas"
- `na` : Répondre "Non applicable"
- `Entrée` : Sauvegarder la réponse et passer à la question suivante
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

### Structure du projet

```
_survey/
├── src/
│   ├── lib.rs          # Point d'entrée Ratzilla
│   ├── models.rs       # Modèles de données
│   ├── audit.rs        # Gestionnaire d'audits et générateur de questions
│   ├── storage.rs      # Stockage localStorage
│   └── recommendations.rs # Génération de recommandations
├── questions/
│   └── technique/      # Fichiers YAML de questions
├── index.html          # Page HTML principale
├── Trunk.toml          # Configuration Trunk
└── Cargo.toml          # Dépendances Rust
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

Modifiez la fonction `generate_recommendations()` dans `src/recommendations.rs` pour ajuster la logique de génération.

## Technologies utilisées

- **Rust** : Langage de programmation
- **Ratzilla** : Framework TUI pour le web (basé sur Ratatui)
- **WebAssembly** : Compilation vers WASM pour le navigateur
- **Trunk** : Outil de build et de développement
- **Serde** : Sérialisation/désérialisation JSON et YAML
- **localStorage** : Stockage côté client

## Licence

MIT

## Inspiration

Cet outil s'inspire de l'outil d'audit technique de l'[Incubateur des Territoires](https://audit.incubateur.anct.gouv.fr/).
