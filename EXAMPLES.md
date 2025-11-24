# Exemples d'utilisation

## Création d'un audit technique

```bash
# Créer un audit technique avec questions par défaut
cargo run -- create "Audit technique MonProjet" --type technique

# Sortie:
# ✅ Audit créé avec succès!
# ID: 550e8400-e29b-41d4-a716-446655440000
# Type: Technique
# Questions: 12
```

## Lister les audits

```bash
cargo run -- list

# Sortie:
# Audits disponibles:
#   550e8400-e29b-41d4-a716-446655440000 - Audit technique MonProjet (Technique) - 0.0% complété
```

## Mode interactif (recommandé)

Le mode interactif vous guide à travers toutes les questions :

```bash
cargo run -- interactive 550e8400-e29b-41d4-a716-446655440000

# Sortie:
# 🎯 Mode interactif pour l'audit: Audit technique MonProjet
#
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 📁 Catégorie: DB
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#
# ❓ Les bases de données sont-elles déployées en haute disponibilité ?
#    Redondance avec master/slave.
#
# Réponse (oui/non/je-ne-sais-pas/non-applicable): oui
# ✅ Réponse enregistrée!
```

## Répondre à une question spécifique

```bash
# D'abord, afficher l'audit pour voir les IDs des questions
cargo run -- show 550e8400-e29b-41d4-a716-446655440000

# Puis répondre à une question spécifique
cargo run -- answer \
  550e8400-e29b-41d4-a716-446655440000 \
  <QUESTION_ID> \
  oui \
  --percentage 80 \
  --comment "Presque complètement déployé"
```

## Générer les recommandations

```bash
cargo run -- recommend 550e8400-e29b-41d4-a716-446655440000

# Sortie:
# 📊 Recommandations (3):
#
# 🔴 [Critique] Améliorer: Les données sont-elles chiffrées au repos (at rest) ?
#    Cette question nécessite une attention particulière. Cela a pour but d'empêcher l'utilisation des données dans le cas d'une intrusion par exemple.
#    Catégorie: Sécu
#
# 🟠 [Élevée] Améliorer la catégorie Infra
#    La catégorie Infra a un score de 40.0%. Il est recommandé de revoir les pratiques dans ce domaine.
#    Catégorie: Infra
```

## Afficher le résumé

```bash
cargo run -- summary 550e8400-e29b-41d4-a716-446655440000

# Sortie:
# 📊 Résumé de l'audit
# Complétion: 75.0%
# Questions totales: 12
# Questions répondues: 9
#
# Par catégorie:
#
# DB
#   Questions: 2/2 (100.0% complété)
#   Réponses positives: 1 (50.0%)
#
# Infra
#   Questions: 3/5 (60.0% complété)
#   Réponses positives: 2 (66.7%)
```

## Workflow complet

```bash
# 1. Créer un audit technique
cargo run -- create "Audit MonProjet" --type technique
# Notez l'ID retourné

# 2. Répondre aux questions en mode interactif
cargo run -- interactive <AUDIT_ID>

# 3. Consulter le résumé
cargo run -- summary <AUDIT_ID>

# 4. Générer les recommandations
cargo run -- recommend <AUDIT_ID>

# 5. Afficher l'audit complet
cargo run -- show <AUDIT_ID>
```

## Créer un audit business

```bash
cargo run -- create "Audit business MonProjet" --type business
```

## Structure des fichiers

Les audits sont sauvegardés dans `./audits/` au format JSON :

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Audit technique MonProjet",
  "audit_type": "Technique",
  "questions": [...],
  "answers": [...],
  "recommendations": []
}
```


