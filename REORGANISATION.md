# Réorganisation de la Structure du Code

## Structure Actuelle (Plate)

```
src/
├── lib.rs              # UI principale (1232 lignes!)
├── models.rs           # Modèles de données
├── audit.rs            # Gestionnaire d'audits
├── recommendations.rs  # Génération de recommandations
├── storage.rs         # Implémentation localStorage
├── storage_trait.rs   # Trait Storage
├── config.rs          # Configuration
├── error.rs           # Gestion d'erreur
├── utils.rs           # Utilitaires
├── cli.rs             # Interface CLI
└── web.rs             # Interface web
```

## Structure Proposée (Hiérarchique)

```
src/
├── lib.rs              # Point d'entrée, réexporte les modules publics
├── domain/            # Logique métier (Domain Layer)
│   ├── mod.rs
│   ├── models.rs      # Entités et value objects
│   ├── audit.rs       # Gestionnaire d'audits
│   └── recommendations.rs # Génération de recommandations
├── infrastructure/    # Infrastructure (Infrastructure Layer)
│   ├── mod.rs
│   ├── storage.rs     # Implémentation localStorage
│   └── storage_trait.rs # Trait Storage
├── ui/                # Interface utilisateur (Presentation Layer)
│   ├── mod.rs
│   └── app.rs         # AppState et logique UI (extrait de lib.rs)
├── shared/            # Code partagé (Shared Kernel)
│   ├── mod.rs
│   ├── config.rs      # Configuration
│   ├── error.rs       # Gestion d'erreur
│   └── utils.rs       # Utilitaires
└── interfaces/       # Interfaces d'entrée (Interface Adapters)
    ├── mod.rs
    ├── cli.rs         # Interface CLI
    └── web.rs         # Interface web
```

## Avantages

1. **Séparation claire des responsabilités** : Domain, Infrastructure, UI, Shared
2. **Facilité de navigation** : Plus facile de trouver le code
3. **Scalabilité** : Facile d'ajouter de nouveaux modules
4. **Testabilité** : Chaque couche peut être testée indépendamment
5. **Architecture DDD** : Suit les principes Domain-Driven Design

## Migration

Les imports seront mis à jour automatiquement grâce au système de modules de Rust.

