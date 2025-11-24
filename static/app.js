const API_BASE = '/api';

let currentAuditId = null;
let currentAudit = null;
let currentQuestionIndex = 0;
let allQuestions = [];
let radarChart = null;

// Gestion des vues
function showView(viewId) {
    document.querySelectorAll('.view').forEach(v => v.classList.add('hidden'));
    document.getElementById(viewId).classList.remove('hidden');
    
    // Détruire le graphique radar si on quitte la vue radar
    if (viewId !== 'radar-view' && radarChart) {
        radarChart.destroy();
        radarChart = null;
    }
}

function showListView() {
    showView('list-view');
    loadAudits();
}

function showCreateForm() {
    showView('create-view');
    document.getElementById('create-form').reset();
}

function showDetailView() {
    if (currentAuditId) {
        showView('detail-view');
        loadAudit(currentAuditId);
    }
}

function showRecommendations() {
    if (currentAuditId) {
        showView('recommendations-view');
        loadRecommendations(currentAuditId);
    }
}

function showRadarChart() {
    if (currentAuditId) {
        showView('radar-view');
        loadRadarChart(currentAuditId);
    }
}

// API calls
async function apiCall(endpoint, options = {}) {
    try {
        const response = await fetch(`${API_BASE}${endpoint}`, {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers,
            },
            ...options,
        });
        const data = await response.json();
        if (!data.success) {
            throw new Error(data.error || 'Erreur inconnue');
        }
        return data.data;
    } catch (error) {
        console.error('API Error:', error);
        throw error;
    }
}

// Charger la liste des audits
async function loadAudits() {
    const container = document.getElementById('audits-list');
    container.innerHTML = '<div class="loading">Chargement...</div>';

    try {
        const audits = await apiCall('/audits');
        
        if (audits.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <h3>Aucun audit</h3>
                    <p>Créez votre premier audit pour commencer</p>
                </div>
            `;
            return;
        }

        // Charger les summaries pour chaque audit
        const auditsWithScores = await Promise.all(
            audits.map(async (audit) => {
                try {
                    const summary = await apiCall(`/audits/${audit.id}/summary`);
                    const globalScore = calculateGlobalScore(summary);
                    return { audit, summary, globalScore };
                } catch (error) {
                    console.error(`Erreur lors du chargement du summary pour ${audit.id}:`, error);
                    return { audit, summary: null, globalScore: null };
                }
            })
        );

        // Créer le tableau
        const tableHTML = `
            <table class="audits-table">
                <thead>
                    <tr>
                        <th>Nom</th>
                        <th>Type</th>
                        <th>Questions</th>
                        <th>Complétion</th>
                        <th>Résultat global</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    ${auditsWithScores.map(({ audit, summary, globalScore }) => {
                        const completion = calculateCompletion(audit);
                        const scoreDisplay = globalScore !== null 
                            ? `<span class="score-badge score-${getScoreClass(globalScore)}">${globalScore.toFixed(1)}%</span>`
                            : '<span class="score-badge score-na">N/A</span>';
                        return `
                            <tr onclick="openAudit('${audit.id}')" class="audit-row">
                                <td class="audit-name">${escapeHtml(audit.name)}</td>
                                <td><span class="type-badge">${escapeHtml(audit.audit_type)}</span></td>
                                <td>${audit.questions.length}</td>
                                <td>${completion.toFixed(1)}%</td>
                                <td>${scoreDisplay}</td>
                                <td>
                                    <button class="btn-delete-small" onclick="event.stopPropagation(); deleteAudit('${audit.id}')" title="Supprimer l'audit">
                                        ×
                                    </button>
                                </td>
                            </tr>
                        `;
                    }).join('')}
                </tbody>
            </table>
        `;
        
        console.log('Génération du tableau avec', auditsWithScores.length, 'audits');
        container.innerHTML = tableHTML;
    } catch (error) {
        container.innerHTML = `<div class="error">Erreur: ${error.message}</div>`;
    }
}

// Calculer le score global à partir du summary
function calculateGlobalScore(summary) {
    if (!summary || !summary.category_stats || summary.category_stats.length === 0) {
        return null;
    }
    
    let totalPositive = 0;
    let totalAnswered = 0;
    
    summary.category_stats.forEach(stat => {
        totalPositive += stat.positive_answers;
        totalAnswered += stat.answered;
    });
    
    if (totalAnswered === 0) {
        return null;
    }
    
    return (totalPositive / totalAnswered) * 100;
}

// Obtenir la classe CSS pour le score
function getScoreClass(score) {
    if (score >= 80) return 'excellent';
    if (score >= 60) return 'good';
    if (score >= 40) return 'average';
    return 'poor';
}

function calculateCompletion(audit) {
    if (audit.questions.length === 0) return 0;
    return (audit.answers.length / audit.questions.length) * 100;
}

// Créer un audit
document.getElementById('create-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const formData = {
        name: document.getElementById('audit-name').value,
        audit_type: document.getElementById('audit-type').value,
        project_name: document.getElementById('project-name').value || null,
        product_name: document.getElementById('product-name').value || null,
    };

    try {
        const audit = await apiCall('/audits', {
            method: 'POST',
            body: JSON.stringify(formData),
        });
        
        currentAuditId = audit.id;
        currentAudit = audit;
        currentQuestionIndex = 0;
        showDetailView();
    } catch (error) {
        alert(`Erreur: ${error.message}`);
    }
});

// Ouvrir un audit
async function openAudit(auditId) {
    currentAuditId = auditId;
    currentQuestionIndex = 0;
    showDetailView();
}

// Supprimer un audit
async function deleteAudit(auditId) {
    if (!confirm('Êtes-vous sûr de vouloir supprimer cet audit ? Cette action est irréversible.')) {
        return;
    }

    try {
        await apiCall(`/audits/${auditId}`, {
            method: 'DELETE',
        });
        
        // Si l'audit supprimé était celui en cours, retourner à la liste
        if (currentAuditId === auditId) {
            currentAuditId = null;
            currentAudit = null;
            showListView();
        } else {
            // Recharger la liste
            loadAudits();
        }
    } catch (error) {
        alert(`Erreur lors de la suppression: ${error.message}`);
    }
}

// Supprimer un audit depuis la vue détail
async function deleteAuditFromDetail() {
    if (!currentAuditId) return;
    await deleteAudit(currentAuditId);
}

// Charger un audit
async function loadAudit(auditId) {
    const container = document.getElementById('questions-container');
    container.innerHTML = '<div class="loading">Chargement...</div>';

    try {
        const audit = await apiCall(`/audits/${auditId}`);
        currentAudit = audit;

        document.getElementById('audit-title').textContent = audit.name;
        document.getElementById('audit-type-display').textContent = audit.audit_type;
        
        const completion = calculateCompletion(audit);
        document.getElementById('completion').textContent = `${completion.toFixed(1)}%`;

        // Trier toutes les questions par catégorie puis par ordre
        allQuestions = audit.questions.sort((a, b) => {
            if (a.category !== b.category) {
                return a.category.localeCompare(b.category);
            }
            return 0;
        });

        // Réinitialiser l'index si nécessaire
        if (currentQuestionIndex >= allQuestions.length) {
            currentQuestionIndex = 0;
        }

        // Mettre à jour la catégorie et la progression
        updateCategory();
        updateProgress();

        // Afficher la question courante
        displayCurrentQuestion();
    } catch (error) {
        container.innerHTML = `<div class="error">Erreur: ${error.message}</div>`;
    }
}

// Mettre à jour la catégorie
function updateCategory() {
    const categoryEl = document.getElementById('category-display');
    
    if (!categoryEl) {
        return; // L'élément n'existe pas (on n'est pas dans la vue détail)
    }
    
    if (allQuestions.length === 0 || currentQuestionIndex >= allQuestions.length) {
        categoryEl.textContent = '-';
        return;
    }

    const question = allQuestions[currentQuestionIndex];
    categoryEl.textContent = question.category;
}

// Mettre à jour la progression
function updateProgress() {
    const progressTextEl = document.getElementById('progress-text');
    const progressFillEl = document.getElementById('progress-fill');
    
    if (!progressTextEl || !progressFillEl) {
        return; // Les éléments n'existent pas (on n'est pas dans la vue détail)
    }
    
    if (allQuestions.length === 0) {
        progressTextEl.textContent = 'Aucune question';
        progressFillEl.style.width = '0%';
        return;
    }

    const progress = `${currentQuestionIndex + 1} / ${allQuestions.length}`;
    const progressPercent = ((currentQuestionIndex + 1) / allQuestions.length) * 100;
    
    progressTextEl.textContent = `Question ${progress}`;
    progressFillEl.style.width = `${progressPercent}%`;
}

// Afficher la question courante
function displayCurrentQuestion() {
    const container = document.getElementById('questions-container');
    
    if (allQuestions.length === 0) {
        container.innerHTML = '<div class="empty-state">Aucune question disponible</div>';
        return;
    }

    if (currentQuestionIndex >= allQuestions.length) {
        // Toutes les questions sont répondues
        container.innerHTML = `
            <div class="empty-state">
                <h3>✓ Audit terminé !</h3>
                <p>Toutes les questions ont été répondues.</p>
                <button class="btn btn-primary" onclick="showRecommendations()" style="margin-top: 1rem;">
                    Voir les recommandations
                </button>
            </div>
        `;
        return;
    }

    const question = allQuestions[currentQuestionIndex];
    
    // Mettre à jour la catégorie dans le cadre
    updateCategory();
    
    container.innerHTML = `
        <div class="category-section">
            ${renderQuestion(question, currentAudit)}
        </div>
    `;
}

// Rendre une question
function renderQuestion(question, audit) {
    const answer = audit.answers.find(a => a.question_id === question.id);
    const isAnswered = !!answer;
    
    return `
        <div class="question-card ${isAnswered ? 'answered' : ''}">
            <div class="question-text">${escapeHtml(question.text)}</div>
            ${question.description ? `<div class="question-description">${escapeHtml(question.description)}</div>` : ''}
            ${question.links && question.links.length > 0 ? `
                <div class="question-links">
                    ${question.links.map(link => `<a href="${link}" target="_blank">→ ${escapeHtml(link)}</a>`).join('')}
                </div>
            ` : ''}
            <div class="answer-section">
                ${isAnswered ? `
                    <div class="current-answer">
                        <strong>Réponse actuelle:</strong> ${answer.answer}
                        ${answer.percentage !== null ? ` (${answer.percentage}%)` : ''}
                        ${answer.comment ? `<br><em>${escapeHtml(answer.comment)}</em>` : ''}
                    </div>
                ` : ''}
                <div class="answer-buttons">
                    <button class="answer-btn ${answer?.answer === 'Oui' ? 'selected oui' : ''}" 
                            onclick="answerQuestion('${question.id}', 'oui', ${question.requires_percentage})">
                        Oui
                    </button>
                    <button class="answer-btn ${answer?.answer === 'Non' ? 'selected non' : ''}" 
                            onclick="answerQuestion('${question.id}', 'non', ${question.requires_percentage})">
                        Non
                    </button>
                    <button class="answer-btn ${answer?.answer === 'JeNeSaisPas' ? 'selected' : ''}" 
                            onclick="answerQuestion('${question.id}', 'je-ne-sais-pas', ${question.requires_percentage})">
                        Je ne sais pas
                    </button>
                    <button class="answer-btn ${answer?.answer === 'NonApplicable' ? 'selected' : ''}" 
                            onclick="answerQuestion('${question.id}', 'non-applicable', ${question.requires_percentage})">
                        Non applicable
                    </button>
                </div>
                ${question.requires_percentage ? `
                    <div style="margin-top: 0.5rem;">
                        <label style="display: block; margin-bottom: 0.25rem; font-size: 0.875rem;">
                            Pourcentage (0-100):
                        </label>
                        <input type="number" id="percentage-${question.id}" 
                               class="percentage-input" 
                               min="0" max="100" 
                               value="${answer?.percentage || ''}"
                               placeholder="0-100">
                    </div>
                ` : ''}
                <div style="margin-top: 0.5rem;">
                    <label style="display: block; margin-bottom: 0.25rem; font-size: 0.875rem;">
                        Commentaire (optionnel):
                    </label>
                    <textarea id="comment-${question.id}" 
                              class="comment-input" 
                              placeholder="Ajoutez un commentaire...">${answer?.comment || ''}</textarea>
                </div>
                <button class="btn btn-primary" style="margin-top: 0.5rem;" 
                        onclick="saveAnswer('${question.id}', ${question.requires_percentage})">
                    ${isAnswered ? 'Mettre à jour' : 'Enregistrer'}
                </button>
            </div>
        </div>
    `;
}

// Répondre à une question
function answerQuestion(questionId, answer, requiresPercentage) {
    // Mettre à jour visuellement les boutons
    const card = document.querySelector(`[id*="${questionId}"]`).closest('.question-card');
    card.querySelectorAll('.answer-btn').forEach(btn => btn.classList.remove('selected', 'oui', 'non'));
    
    const answerMap = {
        'oui': 'Oui',
        'non': 'Non',
        'je-ne-sais-pas': 'JeNeSaisPas',
        'non-applicable': 'NonApplicable'
    };
    
    const buttons = card.querySelectorAll('.answer-btn');
    buttons.forEach(btn => {
        if (btn.textContent.trim() === answerMap[answer] || 
            (answer === 'oui' && btn.textContent.trim() === 'Oui') ||
            (answer === 'non' && btn.textContent.trim() === 'Non') ||
            (answer === 'je-ne-sais-pas' && btn.textContent.trim() === 'Je ne sais pas') ||
            (answer === 'non-applicable' && btn.textContent.trim() === 'Non applicable')) {
            btn.classList.add('selected');
            if (answer === 'oui') btn.classList.add('oui');
            if (answer === 'non') btn.classList.add('non');
        }
    });
}

// Sauvegarder une réponse
async function saveAnswer(questionId, requiresPercentage) {
    const card = document.querySelector(`[id*="${questionId}"]`).closest('.question-card');
    const selectedBtn = card.querySelector('.answer-btn.selected');
    
    if (!selectedBtn) {
        alert('Veuillez sélectionner une réponse');
        return;
    }

    const answerText = selectedBtn.textContent.trim();
    const answerMap = {
        'Oui': 'oui',
        'Non': 'non',
        'Je ne sais pas': 'je-ne-sais-pas',
        'Non applicable': 'non-applicable'
    };

    const answer = answerMap[answerText];
    const percentage = requiresPercentage ? 
        parseInt(document.getElementById(`percentage-${questionId}`).value) || null : null;
    const comment = document.getElementById(`comment-${questionId}`).value.trim() || null;

    try {
        const updatedAudit = await apiCall(`/audits/${currentAuditId}/questions/${questionId}/answer`, {
            method: 'POST',
            body: JSON.stringify({
                answer,
                percentage,
                comment,
            }),
        });

        currentAudit = updatedAudit;
        
        // Mettre à jour les réponses dans allQuestions
        const questionIndex = allQuestions.findIndex(q => q.id === questionId);
        if (questionIndex !== -1) {
            // Mettre à jour la réponse dans l'audit
            const answerIndex = updatedAudit.answers.findIndex(a => a.question_id === questionId);
            if (answerIndex !== -1) {
                // La réponse existe déjà, on la met à jour
                currentAudit.answers[answerIndex] = updatedAudit.answers[answerIndex];
            } else {
                // Nouvelle réponse
                currentAudit.answers.push(updatedAudit.answers.find(a => a.question_id === questionId));
            }
        }

        // Mettre à jour le pourcentage de complétion
        const completion = calculateCompletion(updatedAudit);
        document.getElementById('completion').textContent = `${completion.toFixed(1)}%`;

        // Passer à la question suivante
        currentQuestionIndex++;
        updateCategory();
        updateProgress();
        displayCurrentQuestion();
    } catch (error) {
        alert(`Erreur: ${error.message}`);
    }
}

// Charger et afficher le graphique radar
async function loadRadarChart(auditId) {
    const container = document.querySelector('.radar-container');
    try {
        const summary = await apiCall(`/audits/${auditId}/summary`);
        
        // Vérifier que les données existent
        if (!summary || !summary.category_stats || summary.category_stats.length === 0) {
            if (container) {
                container.innerHTML = `<div class="error">Aucune donnée disponible pour cet audit</div>`;
            }
            return;
        }
        
        // Préparer les données pour le graphique radar
        const labels = [];
        const scores = [];
        const colors = [];
        
        // Fonction pour obtenir le nom de la catégorie
        function getCategoryName(category) {
            // Si c'est déjà une string, la retourner
            if (typeof category === 'string') {
                return category;
            }
            // Si c'est un objet (enum Rust sérialisé comme objet)
            if (typeof category === 'object' && category !== null) {
                const keys = Object.keys(category);
                if (keys.length > 0) {
                    // Gérer le cas où l'enum a une valeur (ex: {"Autre": "valeur"})
                    const firstKey = keys[0];
                    if (category[firstKey] && typeof category[firstKey] === 'string') {
                        return category[firstKey];
                    }
                    return firstKey;
                }
            }
            // Fallback
            return String(category);
        }
        
        // Mapper les noms de catégories pour l'affichage
        function getDisplayName(categoryName) {
            const categoryMap = {
                'Infra': 'Infra',
                'Operation': 'Opération',
                'DB': 'DB',
                'Secu': 'Sécu',
                'A11y': 'A11y',
                'Archi': 'Archi',
                'OSS': 'OSS',
                'DevXP': 'Dev XP',
                'Produit': 'Produit',
                'Bizdev': 'Bizdev',
                'Intra': 'Intra'
            };
            return categoryMap[categoryName] || categoryName;
        }
        
        // Trier les catégories pour un affichage cohérent
        const categoryOrder = ['Infra', 'Operation', 'DB', 'Secu', 'A11y', 'Archi', 'OSS', 'DevXP', 'Produit', 'Bizdev', 'Intra'];
        
        // Créer une copie pour ne pas modifier l'original
        const sortedStats = [...summary.category_stats].sort((a, b) => {
            const aName = getCategoryName(a.category);
            const bName = getCategoryName(b.category);
            const aIndex = categoryOrder.findIndex(cat => aName === cat || aName.includes(cat) || cat.includes(aName));
            const bIndex = categoryOrder.findIndex(cat => bName === cat || bName.includes(cat) || cat.includes(bName));
            const aPos = aIndex !== -1 ? aIndex : 999;
            const bPos = bIndex !== -1 ? bIndex : 999;
            if (aPos !== bPos) return aPos - bPos;
            return aName.localeCompare(bName);
        });
        
        sortedStats.forEach(stat => {
            const categoryName = getCategoryName(stat.category);
            const displayName = getDisplayName(categoryName);
            labels.push(displayName);
            
            // Calculer le score : pourcentage de réponses positives parmi les questions répondues
            let score = 0;
            if (stat.answered > 0) {
                score = (stat.positive_answers / stat.answered) * 100;
            }
            scores.push(score);
            
            // Couleur basée sur le score
            if (score >= 80) {
                colors.push('rgba(34, 197, 94, 0.6)'); // Vert
            } else if (score >= 60) {
                colors.push('rgba(234, 179, 8, 0.6)'); // Jaune
            } else if (score >= 40) {
                colors.push('rgba(249, 115, 22, 0.6)'); // Orange
            } else {
                colors.push('rgba(239, 68, 68, 0.6)'); // Rouge
            }
        });
        
        // Vérifier qu'on a des données à afficher
        if (labels.length === 0) {
            if (container) {
                container.innerHTML = `<div class="error">Aucune catégorie à afficher</div>`;
            }
            return;
        }
        
        // Détruire le graphique existant s'il existe
        if (radarChart) {
            radarChart.destroy();
            radarChart = null;
        }
        
        // Créer le graphique radar
        const ctx = document.getElementById('radar-chart');
        if (!ctx) {
            console.error('Canvas radar-chart introuvable');
            return;
        }
        
        radarChart = new Chart(ctx, {
            type: 'radar',
            data: {
                labels: labels,
                datasets: [{
                    label: 'Score par catégorie (%)',
                    data: scores,
                    backgroundColor: 'rgba(37, 99, 235, 0.2)',
                    borderColor: 'rgba(37, 99, 235, 1)',
                    borderWidth: 2,
                    pointBackgroundColor: colors,
                    pointBorderColor: '#fff',
                    pointHoverBackgroundColor: '#fff',
                    pointHoverBorderColor: 'rgba(37, 99, 235, 1)',
                    pointRadius: 5,
                    pointHoverRadius: 7
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: true,
                scales: {
                    r: {
                        beginAtZero: true,
                        max: 100,
                        ticks: {
                            stepSize: 20,
                            font: {
                                size: 12
                            }
                        },
                        pointLabels: {
                            font: {
                                size: 13,
                                weight: 'bold'
                            }
                        },
                        grid: {
                            color: 'rgba(0, 0, 0, 0.1)'
                        }
                    }
                },
                plugins: {
                    legend: {
                        display: true,
                        position: 'bottom',
                        labels: {
                            font: {
                                size: 14
                            },
                            padding: 15
                        }
                    },
                    tooltip: {
                        callbacks: {
                            label: function(context) {
                                const label = context.dataset.label || '';
                                const value = context.parsed.r.toFixed(1);
                                const stat = sortedStats[context.dataIndex];
                                return [
                                    `${label}: ${value}%`,
                                    `Questions: ${stat.answered}/${stat.total_questions}`,
                                    `Positives: ${stat.positive_answers}`
                                ];
                            }
                        }
                    }
                }
            }
        });
    } catch (error) {
        console.error('Erreur lors du chargement du graphique:', error);
        if (container) {
            container.innerHTML = `<div class="error">Erreur: ${error.message}</div>`;
        }
    }
}

// Charger les recommandations
async function loadRecommendations(auditId) {
    const container = document.getElementById('recommendations-list');
    container.innerHTML = '<div class="loading">Chargement...</div>';

    try {
        const recommendations = await apiCall(`/audits/${auditId}/recommendations`);
        
        if (recommendations.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <h3>✓ Aucune recommandation</h3>
                    <p>L'audit semble bon !</p>
                </div>
            `;
            return;
        }

        // Trier par priorité
        const priorityOrder = { 'Critique': 0, 'Elevee': 1, 'Moyenne': 2, 'Faible': 3 };
        recommendations.sort((a, b) => priorityOrder[a.priority] - priorityOrder[b.priority]);

        container.innerHTML = recommendations.map(rec => {
            const priorityClass = rec.priority.toLowerCase();
            return `
                <div class="recommendation-card priority-${priorityClass}">
                    <div class="recommendation-header">
                        <span class="priority-badge ${priorityClass}">${escapeHtml(rec.priority)}</span>
                        <div class="recommendation-title">${escapeHtml(rec.title)}</div>
                    </div>
                    <div class="recommendation-description">${escapeHtml(rec.description)}</div>
                    <div style="margin-top: 0.75rem; font-size: 0.875rem; color: var(--text-secondary);">
                        Catégorie: ${escapeHtml(rec.category)}
                    </div>
                </div>
            `;
        }).join('');
    } catch (error) {
        container.innerHTML = `<div class="error">Erreur: ${error.message}</div>`;
    }
}

// Utilitaires
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Initialisation
document.addEventListener('DOMContentLoaded', () => {
    showListView();
});

