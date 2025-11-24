# Questions d'audit technique

Ce dossier contient les questions d'audit technique organisées par catégorie dans des fichiers YAML.

## Structure

Chaque fichier YAML contient une liste de questions pour une catégorie spécifique :

- `infra.yaml` - Infrastructure et scaling
- `db.yaml` - Base de données et performance
- `operation.yaml` - Opérations et observabilité
- `secu.yaml` - Sécurité
- `a11y.yaml` - Accessibilité
- `archi.yaml` - Architecture
- `oss.yaml` - Open Source
- `devxp.yaml` - Developer Experience
- `produit.yaml` - Produit et expérience utilisateur
- `organisation.yaml` - Organisation, processus et gestion des connaissances

## Format des questions

Chaque question doit suivre ce format :

```yaml
questions:
  - text: "Texte de la question"
    description: "Description détaillée (optionnel)"
    category: Infra  # Infra, Operation, DB, Secu, A11y, Archi, OSS, DevXP, Produit
    requires_percentage: true  # ou false
    expected_answer: Oui  # Oui, Non, JeNeSaisPas, NonApplicable
    links:  # optionnel
      - "https://exemple.com"
```

## Ajouter ou modifier des questions

1. Éditez le fichier YAML correspondant à la catégorie
2. Redémarrez l'application pour charger les nouvelles questions
3. Les questions sont chargées automatiquement au démarrage

## Notes

- Les questions sont orientées scaling et passage au niveau supérieur pour les startups
- Chaque catégorie couvre les aspects critiques pour le scaling
- Les questions peuvent être modifiées sans recompiler le code Rust



