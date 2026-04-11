/**
 * Módulo Stats - Información y métricas
 */

export function getStats() {
    return [
        {
            value: '⚡',
            label: 'Instant HMR'
        },
        {
            value: '📦',
            label: 'ESM Native'
        },
        {
            value: '🚀',
            label: 'Production Ready'
        },
        {
            value: '✨',
            label: 'Modern Stack'
        }
    ];
}

export function getMetrics() {
    return {
        buildTime: '0.35s',
        bundleSize: '~150KB gzip',
        modules: 4,
        dependencies: 0,
        timestamp: new Date().toISOString()
    };
}

export function getEnvironment() {
    return {
        platform: navigator.platform,
        userAgent: navigator.userAgent,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        language: navigator.language,
        memory: navigator.deviceMemory || 'unknown'
    };
}
