// ============================================
// API Client for GCP IAM Search Backend
// ============================================

export type SearchMode = 'exact' | 'prefix' | 'fuzzy';

export interface RoleSummary {
    name: string;
    title: string;
    stage: string;
}

export interface Permission {
    name: string;
    service: string;
    resource: string;
    action: string;
    score: number;
    granted_by_roles: RoleSummary[];
}

export interface Role {
    name: string;
    title: string;
    description: string;
    stage: string;
    score: number;
    permission_count: number;
    sample_permissions: string[];
}

export interface SearchResults {
    permissions: Permission[];
    roles: Role[];
}

export class SearchClient {
    private baseUrl: string;
    private cache: Map<string, SearchResults> = new Map();
    private abortController: AbortController | null = null;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
    }

    /**
     * Search for permissions and roles
     */
    async search(query: string, mode: SearchMode = 'fuzzy'): Promise<SearchResults> {
        // Cancel previous request if any
        if (this.abortController) {
            this.abortController.abort();
        }

        const cacheKey = `${query}:${mode}`;
        const cached = this.cache.get(cacheKey);
        if (cached) {
            return cached;
        }

        this.abortController = new AbortController();

        try {
            const params = new URLSearchParams({
                q: query,
                mode,
                limit: '20',
            });

            const url = `${this.baseUrl}/search?${params}`;

            const response = await fetch(url, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                },
                signal: this.abortController.signal,
            });

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const json = await response.json();
            // Backend returns {success: true, data: {permissions: [...], roles: [...]}}
            const rawData = json.data || json;

            const results: SearchResults = {
                permissions: rawData.permissions || [],
                roles: rawData.roles || [],
            };

            // Cache results
            this.cache.set(cacheKey, results);

            return results;
        } catch (error) {
            // If it's an abort error, return empty results silently
            if (error instanceof Error && error.name === 'AbortError') {
                return { permissions: [], roles: [] };
            }

            throw error;
        }
    }

    /**
     * Clear cache
     */
    clearCache() {
        this.cache.clear();
    }

    /**
     * Get API health status
     */
    async getHealth(): Promise<boolean> {
        try {
            const response = await fetch(`${this.baseUrl}/health`, {
                method: 'GET',
            });
            return response.ok;
        } catch {
            return false;
        }
    }
}
