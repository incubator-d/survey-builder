# Réorganisation de l'historique Git

Ce document explique comment réorganiser l'historique des commits pour suivre les conventions Conventional Commits.

## État actuel

L'historique actuel contient quelques commits qui ne suivent pas parfaitement les conventions :
- `c0c2805` - "Sauvegarde initiale - Projet fonctionnel" → devrait être `chore: initial commit`
- `39e233c` - "Ajout de tests unitaires..." → devrait être `test: add unit tests`
- `a8c5c07` - "Nettoyage du code mort" → devrait être `refactor: remove dead code`
- `b94451d` - "Amélioration de l'affichage..." → devrait être `feat(ui): improve question display`
- `e603ad0` - "Amélioration de la couverture..." → devrait être `test: improve test coverage to 100%`

## Réorganisation recommandée

Pour réorganiser l'historique, vous pouvez utiliser un rebase interactif :

```bash
# Démarrer un rebase interactif depuis le premier commit
git rebase -i --root

# Dans l'éditeur, changer les commits à réécrire :
# - c0c2805 : changer "pick" en "reword" et modifier le message
# - 39e233c : changer "pick" en "reword" et modifier le message
# - etc.
```

## Alternative : Garder l'historique actuel

Étant donné que :
1. Les hooks Git sont maintenant en place pour garantir la qualité future
2. L'historique actuel est fonctionnel et compréhensible
3. Un rebase complet peut être complexe avec des conflits

Il est acceptable de garder l'historique actuel tel quel. Les futurs commits suivront automatiquement les conventions grâce aux hooks.

## Vérification des hooks

Les hooks sont automatiquement installés via :
```bash
git config core.hooksPath .githooks
```

Pour tester un hook manuellement :
```bash
# Tester le hook commit-msg
echo "feat: test commit" | .githooks/commit-msg /dev/stdin

# Tester le hook pre-commit
.githooks/pre-commit
```

