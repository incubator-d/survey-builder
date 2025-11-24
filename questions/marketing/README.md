# Questions d'audit Marketing

Ce répertoire contient les questions pour les audits de type **Marketing**.

## Structure

Les questions sont organisées par thème dans différents fichiers YAML :

- `strategie.yaml` : Questions sur la stratégie marketing, positionnement, USP
- `communication.yaml` : Questions sur la communication, identité visuelle, présence digitale
- `acquisition.yaml` : Questions sur l'acquisition de leads, SEO, publicité, marketing automation
- `retention.yaml` : Questions sur la rétention client, fidélisation, satisfaction

## Catégories utilisées

Les questions marketing utilisent principalement les catégories suivantes :
- **Bizdev** : Business development, stratégie marketing
- **Organisation** : Processus marketing, gestion des leads

## Format

Chaque fichier YAML suit le format standard :

```yaml
questions:
  - text: "Votre question ici"
    description: "Description détaillée (optionnel)"
    category: Bizdev
    requires_percentage: false
    expected_answer: Oui
    links:
      - "https://exemple.com"
```

## Ajouter des questions

Pour ajouter de nouvelles questions marketing :

1. Identifiez le fichier YAML approprié selon le thème
2. Ajoutez votre question en respectant le format ci-dessus
3. Utilisez la catégorie appropriée (Bizdev ou Organisation)
4. Indiquez si la question nécessite un pourcentage avec `requires_percentage: true`

