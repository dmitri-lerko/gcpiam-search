// ============================================
// GCP IAM Search - Vanilla Frontend
// ============================================

import { SearchClient } from './api';
import { SearchUI } from './ui';
import { SearchManager } from './search';

// Configuration
const CONFIG = {
    API_BASE_URL: 'https://gcpiam.com/api/v1',
    SEARCH_DEBOUNCE_MS: 150,
    RESULT_LIMIT: 20,
    FUZZY_THRESHOLD: 0.5,
};

// Initialize application
async function initializeApp() {
    try {
        // Create components
        const apiClient = new SearchClient(CONFIG.API_BASE_URL);
        const ui = new SearchUI();
        const searchManager = new SearchManager({
            debounceMs: CONFIG.SEARCH_DEBOUNCE_MS,
            resultLimit: CONFIG.RESULT_LIMIT,
            fuzzyThreshold: CONFIG.FUZZY_THRESHOLD,
        });

        // Get DOM elements
        const searchInput = document.getElementById('searchInput') as HTMLInputElement;
        const clearBtn = document.getElementById('clearBtn') as HTMLButtonElement;
        const modeButtons = document.querySelectorAll('.mode-btn');

        // Handle search input
        searchInput.addEventListener('input', async (e) => {
            const query = (e.target as HTMLInputElement).value.trim();

            if (!query) {
                ui.showEmptyState();
                return;
            }

            ui.showLoading();

            try {
                const mode = searchManager.getCurrentMode();
                const results = await apiClient.search(query, mode);

                if (results.permissions.length === 0 && results.roles.length === 0) {
                    ui.showEmptyState();
                } else {
                    ui.displayResults(results);
                }
            } catch (error) {
                console.error('Search error:', error);
                ui.showError();
            }
        });

        // Handle clear button
        clearBtn.addEventListener('click', () => {
            searchInput.value = '';
            searchInput.focus();
            ui.showEmptyState();
        });

        // Handle mode selection
        modeButtons.forEach((btn) => {
            btn.addEventListener('click', () => {
                modeButtons.forEach((b) => b.classList.remove('active'));
                btn.classList.add('active');
                const mode = btn.getAttribute('data-mode') as 'exact' | 'prefix' | 'fuzzy';
                searchManager.setMode(mode);

                // Re-search if there's a query
                if (searchInput.value.trim()) {
                    searchInput.dispatchEvent(new Event('input'));
                }
            });
        });

        // Handle keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            // Focus search box with /
            if (e.key === '/' && document.activeElement !== searchInput) {
                e.preventDefault();
                searchInput.focus();
            }

            // Arrow keys for navigation
            if (e.key === 'ArrowDown' && document.activeElement === searchInput) {
                e.preventDefault();
                ui.selectNext();
            }

            if (e.key === 'ArrowUp' && document.activeElement === searchInput) {
                e.preventDefault();
                ui.selectPrevious();
            }

            // Enter to copy selected result
            if (e.key === 'Enter' && document.activeElement === searchInput) {
                const selected = ui.getSelectedResult();
                if (selected) {
                    e.preventDefault();
                    copyToClipboard(selected);
                }
            }

            // Escape to clear
            if (e.key === 'Escape' && document.activeElement === searchInput) {
                clearBtn.click();
            }
        });

        // Set initial focus
        searchInput.focus();

        console.log('✓ GCP IAM Search initialized');
    } catch (error) {
        console.error('Failed to initialize app:', error);
        document.body.innerHTML =
            '<div style="padding: 2rem; color: red;"><h1>Failed to initialize application</h1><p>Check console for details.</p></div>';
    }
}

// Utility: Copy text to clipboard
function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(
        () => {
            // Show temporary success message
            const msg = document.createElement('div');
            msg.textContent = '✓ Copied!';
            msg.style.cssText =
                'position:fixed;top:20px;right:20px;background:#4CAF50;color:white;padding:10px 16px;border-radius:6px;z-index:9999;font-size:14px;';
            document.body.appendChild(msg);
            setTimeout(() => msg.remove(), 2000);
        },
        () => {
            console.error('Failed to copy to clipboard');
        }
    );
}

// Start app when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    initializeApp();
}
