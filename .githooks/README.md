# Git Hooks

Ce répertoire contient les hooks Git pour garantir la qualité des commits.

## Hooks disponibles

### `commit-msg`
Valide que les messages de commit suivent le format [Conventional Commits](https://www.conventionalcommits.org/).

Format requis : `<type>(<scope>): <subject>`

Types valides :
- `feat`: Nouvelle fonctionnalité
- `fix`: Correction de bug
- `docs`: Documentation
- `style`: Formatage
- `refactor`: Refactorisation
- `test`: Tests
- `chore`: Maintenance
- `perf`: Performance
- `ci`: CI/CD
- `build`: Build
- `revert`: Annulation

### `pre-commit`
Vérifie automatiquement avant chaque commit :
- Formatage du code (`cargo fmt`)
- Lints (`cargo clippy`)

## Installation

Les hooks sont automatiquement installés via :
```bash
git config core.hooksPath .githooks
```

Cette commande est déjà exécutée dans le projet.

## Désactiver temporairement

Pour désactiver les hooks temporairement (non recommandé) :
```bash
git commit --no-verify -m "message"
```

