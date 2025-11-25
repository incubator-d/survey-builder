# Questions d'audit Business

Ce répertoire contient les questions pour les audits de type **Business**.

## Structure

Les questions sont organisées par thème dans différents fichiers YAML :

- `strategie.yaml` : Questions sur la stratégie business, modèle économique, ROI, croissance
- `communication.yaml` : Questions sur la communication et le marketing
- `organisation.yaml` : Questions sur l'organisation commerciale, processus, CRM, support client

## Catégories utilisées

Les questions business utilisent principalement les catégories suivantes :
- **Bizdev** : Business development, stratégie commerciale
- **Organisation** : Processus, organisation de l'équipe commerciale

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

Pour ajouter de nouvelles questions business :

1. Identifiez le fichier YAML approprié selon le thème
2. Ajoutez votre question en respectant le format ci-dessus
3. Utilisez la catégorie appropriée (Bizdev ou Organisation)
4. Indiquez si la question nécessite un pourcentage avec `requires_percentage: true`

