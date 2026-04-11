/**
 * Módulo Counter - Componente interactivo
 * Demuestra el estado y los event listeners
 */

export class Counter {
    constructor(initialValue = 0) {
        this.value = initialValue;
        this.history = [initialValue];
    }

    increment() {
        this.value++;
        this.history.push(this.value);
        console.log(`📈 Counter incremented: ${this.value}`);
        return this.value;
    }

    decrement() {
        this.value--;
        this.history.push(this.value);
        console.log(`📉 Counter decremented: ${this.value}`);
        return this.value;
    }

    reset() {
        const oldValue = this.value;
        this.value = 0;
        this.history = [0];
        console.log(`🔄 Counter reset: ${oldValue} → 0`);
        return this.value;
    }

    getHistory() {
        return [...this.history];
    }

    render() {
        return `
            <div style="background: white; padding: 0;">
                <div class="counter-label">Prueba el contador interactivo:</div>
                <div class="counter">${this.value}</div>
                <div class="buttons">
                    <button class="btn-secondary" data-action="decrement">➖ Decrementar</button>
                    <button class="btn-primary" data-action="increment">➕ Incrementar</button>
                    <button class="btn-secondary" data-action="reset">🔄 Reset</button>
                </div>
                ${this.history.length > 1 ? `
                    <p style="margin-top: 15px; color: #999; font-size: 0.9em;">
                        Historial: ${this.history.join(' → ')}
                    </p>
                ` : ''}
            </div>
        `;
    }
}
