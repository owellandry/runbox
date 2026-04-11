/**
 * Módulo principal de la aplicación
 * Renderiza la estructura HTML principal
 */

export function createApp(config) {
    const { title, subtitle, stats, features } = config;

    const statsHtml = stats
        .map(stat => `
            <div class="stat-card">
                <div class="stat-value">${stat.value}</div>
                <div class="stat-label">${stat.label}</div>
            </div>
        `)
        .join('');

    const featuresHtml = features
        .map(feature => `
            <div class="feature-item">
                <span class="feature-icon">${feature.icon}</span>
                <span class="feature-text">${feature.text}</span>
            </div>
        `)
        .join('');

    return `
        <h1>${title}</h1>
        <p class="subtitle">${subtitle}</p>

        <div class="stats">
            ${statsHtml}
        </div>

        <div class="features">
            <h2>✨ Características de Vite</h2>
            <div class="feature-list">
                ${featuresHtml}
            </div>
        </div>

        <div class="counter-section">
            <div class="counter-label">Prueba el contador interactivo:</div>
            <div class="counter">0</div>
            <div class="buttons">
                <button class="btn-secondary" data-action="decrement">➖ Decrementar</button>
                <button class="btn-primary" data-action="increment">➕ Incrementar</button>
                <button class="btn-secondary" data-action="reset">🔄 Reset</button>
            </div>
        </div>

        <div class="footer">
            <p>💡 Rendered by <strong>RunBox WebAssembly</strong></p>
            <p>Built with <strong>Vite</strong> modules</p>
            <p>
                <small>
                    Timestamp: ${new Date().toLocaleString()}
                </small>
            </p>
        </div>
    `;
}
