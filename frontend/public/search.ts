// ============================================
// Search Manager - State and Mode Management
// ============================================

export type SearchMode = 'exact' | 'prefix' | 'fuzzy';

export interface SearchManagerConfig {
    debounceMs?: number;
    resultLimit?: number;
    fuzzyThreshold?: number;
}

/**
 * Manages search state, modes, and debouncing
 */
export class SearchManager {
    private currentMode: SearchMode = 'fuzzy';
    private debounceMs: number = 150;
    private resultLimit: number = 20;
    private fuzzyThreshold: number = 0.5;
    private debounceTimer: NodeJS.Timeout | null = null;
    private lastQuery: string = '';

    constructor(config: SearchManagerConfig = {}) {
        this.debounceMs = config.debounceMs ?? 150;
        this.resultLimit = config.resultLimit ?? 20;
        this.fuzzyThreshold = config.fuzzyThreshold ?? 0.5;
    }

    /**
     * Set the current search mode
     */
    setMode(mode: SearchMode) {
        this.currentMode = mode;
    }

    /**
     * Get the current search mode
     */
    getCurrentMode(): SearchMode {
        return this.currentMode;
    }

    /**
     * Set debounce delay
     */
    setDebounceMs(ms: number) {
        this.debounceMs = ms;
    }

    /**
     * Set result limit
     */
    setResultLimit(limit: number) {
        this.resultLimit = limit;
    }

    /**
     * Set fuzzy threshold (0-1)
     */
    setFuzzyThreshold(threshold: number) {
        this.fuzzyThreshold = Math.max(0, Math.min(1, threshold));
    }

    /**
     * Debounce a search operation
     */
    debounceSearch(callback: (query: string) => void, query: string): void {
        if (this.debounceTimer !== null) {
            clearTimeout(this.debounceTimer);
        }

        this.lastQuery = query;

        this.debounceTimer = setTimeout(() => {
            callback(query);
            this.debounceTimer = null;
        }, this.debounceMs);
    }

    /**
     * Cancel pending debounced search
     */
    cancelDebounce(): void {
        if (this.debounceTimer !== null) {
            clearTimeout(this.debounceTimer);
            this.debounceTimer = null;
        }
    }

    /**
     * Get last query that was searched
     */
    getLastQuery(): string {
        return this.lastQuery;
    }

    /**
     * Validate and normalize a search query
     */
    validateQuery(query: string): string {
        return query
            .trim()
            .toLowerCase()
            .slice(0, 100); // Max 100 chars
    }

    /**
     * Get search configuration for API call
     */
    getSearchConfig() {
        return {
            mode: this.currentMode,
            limit: this.resultLimit,
            fuzzyThreshold: this.fuzzyThreshold,
        };
    }
}

/**
 * Simple local search implementation for client-side filtering
 */
export class LocalSearchEngine {
    private permissions: Array<{ name: string; service: string; score?: number }> = [];
    private roles: Array<{ name: string; title: string; score?: number }> = [];

    /**
     * Index permissions
     */
    indexPermissions(perms: Array<{ name: string; service: string }>) {
        this.permissions = perms;
    }

    /**
     * Index roles
     */
    indexRoles(roles: Array<{ name: string; title: string }>) {
        this.roles = roles;
    }

    /**
     * Exact search
     */
    searchExact(query: string) {
        const q = query.toLowerCase();
        const permissions = this.permissions.filter((p) => p.name.toLowerCase() === q);
        const roles = this.roles.filter(
            (r) =>
                r.name.toLowerCase() === q ||
                r.title.toLowerCase() === q
        );
        return { permissions, roles };
    }

    /**
     * Prefix search
     */
    searchPrefix(query: string) {
        const q = query.toLowerCase();
        const permissions = this.permissions
            .filter((p) => p.name.toLowerCase().startsWith(q))
            .map((p) => ({
                ...p,
                score: 0.9,
            }));
        const roles = this.roles
            .filter(
                (r) =>
                    r.name.toLowerCase().startsWith(q) ||
                    r.title.toLowerCase().startsWith(q)
            )
            .map((r) => ({
                ...r,
                score: 0.8,
            }));
        return { permissions, roles };
    }

    /**
     * Fuzzy search using simple N-gram matching
     */
    searchFuzzy(query: string, threshold: number = 0.5) {
        const q = query.toLowerCase();
        const ngrams = this.extractNgrams(q, 3);

        const permissions = this.permissions
            .map((p) => ({
                ...p,
                score: this.calculateSimilarity(
                    ngrams,
                    this.extractNgrams(p.name.toLowerCase(), 3)
                ),
            }))
            .filter((p) => p.score >= threshold);

        const roles = this.roles
            .map((r) => ({
                ...r,
                score: Math.max(
                    this.calculateSimilarity(
                        ngrams,
                        this.extractNgrams(r.name.toLowerCase(), 3)
                    ),
                    this.calculateSimilarity(
                        ngrams,
                        this.extractNgrams(r.title.toLowerCase(), 3)
                    )
                ),
            }))
            .filter((r) => r.score >= threshold);

        return { permissions, roles };
    }

    /**
     * Full-text search
     */
    searchFulltext(query: string) {
        const tokens = query.toLowerCase().split(/\s+/);

        const permissions = this.permissions
            .map((p) => ({
                ...p,
                matchCount: tokens.filter((t) =>
                    p.name.toLowerCase().includes(t)
                ).length,
            }))
            .filter((p) => p.matchCount > 0)
            .map(({ matchCount, ...p }) => ({
                ...p,
                score: matchCount / tokens.length,
            }));

        const roles = this.roles
            .map((r) => ({
                ...r,
                matchCount: tokens.filter(
                    (t) =>
                        r.name.toLowerCase().includes(t) ||
                        r.title.toLowerCase().includes(t)
                ).length,
            }))
            .filter((r) => r.matchCount > 0)
            .map(({ matchCount, ...r }) => ({
                ...r,
                score: matchCount / tokens.length,
            }));

        return { permissions, roles };
    }

    /**
     * Extract n-grams from text
     */
    private extractNgrams(text: string, n: number): string[] {
        if (text.length < n) return [text];
        const ngrams: string[] = [];
        for (let i = 0; i <= text.length - n; i++) {
            ngrams.push(text.slice(i, i + n));
        }
        return ngrams;
    }

    /**
     * Calculate Jaccard similarity
     */
    private calculateSimilarity(set1: string[], set2: string[]): number {
        if (set1.length === 0 && set2.length === 0) return 1.0;

        const intersection = set1.filter((item) => set2.includes(item)).length;
        const union = new Set([...set1, ...set2]).size;

        return union === 0 ? 0 : intersection / union;
    }
}
