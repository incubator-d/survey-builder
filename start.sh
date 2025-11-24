#!/bin/bash
# Script de démarrage pour Survey Audit avec Ratzilla

echo "🚀 Démarrage de Survey Audit..."
echo "📦 Vérification des dépendances..."

# Vérifier que wasm32-unknown-unknown est installé
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "⚠️  Installation de wasm32-unknown-unknown..."
    rustup target add wasm32-unknown-unknown
fi

# Lancer trunk avec le fichier HTML explicite
echo "🌐 Lancement du serveur Trunk..."
trunk serve index.html

