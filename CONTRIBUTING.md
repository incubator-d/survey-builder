# Guide de contribution

Merci de votre intérêt pour contribuer à Survey Audit ! Ce document fournit des directives pour contribuer au projet.

## Comment contribuer

### Signaler un bug

Si vous trouvez un bug, veuillez créer une [issue](../../issues) avec :
- Une description claire du problème
- Les étapes pour reproduire le bug
- Le comportement attendu vs le comportement observé
- Votre environnement (OS, version de Rust, navigateur, etc.)

### Proposer une fonctionnalité

Les suggestions de fonctionnalités sont les bienvenues ! Créez une [issue](../../issues) avec :
- Une description détaillée de la fonctionnalité
- Le cas d'usage et la valeur ajoutée
- Des exemples d'utilisation si possible

### Contribuer du code

1. **Fork le projet** et clonez votre fork
2. **Créez une branche** pour votre fonctionnalité ou correction :
   ```bash
   git checkout -b feature/ma-fonctionnalite
   # ou
   git checkout -b fix/mon-bug
   ```
3. **Faites vos modifications** en suivant les conventions du projet
4. **Ajoutez des tests** si vous ajoutez une nouvelle fonctionnalité
5. **Vérifiez que les tests passent** :
   ```bash
   cargo test
   ```
6. **Vérifiez le formatage** :
   ```bash
   cargo fmt --check
   ```
7. **Vérifiez les lints** :
   ```bash
   cargo clippy -- -D warnings
   ```
8. **Commitez vos changements** avec des messages clairs :
   ```bash
   git commit -m "feat: ajouter une nouvelle fonctionnalité"
   # ou
   git commit -m "fix: corriger un bug"
   ```
9. **Pushez vers votre fork** :
   ```bash
   git push origin feature/ma-fonctionnalite
   ```
10. **Ouvrez une Pull Request** sur GitHub

## Conventions de code

### Formatage

Le projet utilise `rustfmt`. Assurez-vous que votre code est formaté :
```bash
cargo fmt
```

### Linting

Le projet utilise `clippy`. Vérifiez qu'il n'y a pas d'avertissements :
```bash
cargo clippy -- -D warnings
```

### Tests

- Ajoutez des tests pour toute nouvelle fonctionnalité
- Assurez-vous que tous les tests passent avant de soumettre une PR
- Les tests doivent être clairs et bien documentés

### Messages de commit

Nous utilisons des conventions de commit pour faciliter la maintenance :
- `feat:` : Nouvelle fonctionnalité
- `fix:` : Correction de bug
- `docs:` : Documentation uniquement
- `style:` : Changements de formatage (pas de changement de code)
- `refactor:` : Refactorisation du code
- `test:` : Ajout ou modification de tests
- `chore:` : Tâches de maintenance

Exemples :
```
feat: ajouter l'export en format JSON
fix: corriger le calcul des scores par catégorie
docs: mettre à jour la documentation sur les questions
```

## Structure du projet

```
_survey/
├── src/
│   ├── domain/          # Logique métier
│   ├── infrastructure/  # Implémentations techniques (stockage, etc.)
│   ├── shared/          # Code partagé (config, erreurs, utils)
│   ├── ui/              # Interface utilisateur
│   └── interfaces/      # Interfaces (CLI, web)
├── questions/           # Fichiers YAML de questions
└── static/             # Fichiers statiques
```

## Ajouter des questions

Pour ajouter des questions à un audit :

1. Modifiez ou créez un fichier YAML dans `questions/technique/`
2. Suivez le format existant :
   ```yaml
   questions:
     - text: "Votre question"
       description: "Description optionnelle"
       category: "A11y"
       requires_percentage: false
       expected_answer: "Oui"
       links:
         - "https://exemple.com"
   ```
3. Testez que les questions se chargent correctement

## Ajouter une nouvelle catégorie

1. Ajoutez la catégorie à l'enum `Category` dans `src/domain/models.rs`
2. Mettez à jour la documentation dans le README
3. Ajoutez des questions pour cette catégorie dans les fichiers YAML appropriés

## Processus de review

- Les PR seront examinées par les mainteneurs
- Des commentaires peuvent être demandés pour améliorer le code
- Les PR doivent passer tous les tests et vérifications avant d'être mergées

## Questions ?

Si vous avez des questions, n'hésitez pas à ouvrir une issue ou à contacter les mainteneurs.

Merci de contribuer à Survey Audit ! 🎉

