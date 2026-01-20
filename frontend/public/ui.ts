// ============================================
// UI Management - DOM Manipulation and Rendering
// ============================================

import { SearchResults, Permission, Role } from './api';

export class SearchUI {
    private selectedIndex: number = -1;
    private currentResults: SearchResults = { permissions: [], roles: [] };

    constructor() {
        this.initializeElements();
    }

    private initializeElements() {
        // Verify all required elements exist
        const requiredIds = [
            'loadingState',
            'emptyState',
            'errorState',
            'resultsContainer',
            'permissionsSection',
            'rolesSection',
            'permissionsList',
            'rolesList',
            'permissionCount',
            'roleCount',
        ];

        for (const id of requiredIds) {
            if (!document.getElementById(id)) {
                console.warn(`Element with id "${id}" not found`);
            }
        }
    }

    /**
     * Show loading state
     */
    showLoading() {
        this.hideAllStates();
        const loadingState = document.getElementById('loadingState');
        if (loadingState) {
            loadingState.style.display = 'block';
        }
        this.selectedIndex = -1;
    }

    /**
     * Show empty state
     */
    showEmptyState() {
        this.hideAllStates();
        const emptyState = document.getElementById('emptyState');
        if (emptyState) {
            emptyState.style.display = 'block';
        }
        this.selectedIndex = -1;
    }

    /**
     * Show error state
     */
    showError() {
        this.hideAllStates();
        const errorState = document.getElementById('errorState');
        if (errorState) {
            errorState.style.display = 'block';
        }
        this.selectedIndex = -1;
    }

    /**
     * Display search results
     */
    displayResults(results: SearchResults) {
        this.currentResults = results;
        this.hideAllStates();
        this.selectedIndex = -1;

        const resultsContainer = document.getElementById('resultsContainer');
        if (!resultsContainer) return;

        // Hide both sections initially
        const permSection = document.getElementById('permissionsSection');
        const rolesSection = document.getElementById('rolesSection');
        if (permSection) permSection.style.display = 'none';
        if (rolesSection) rolesSection.style.display = 'none';

        // Display permissions
        if (results.permissions.length > 0) {
            this.displayPermissions(results.permissions);
        }

        // Display roles
        if (results.roles.length > 0) {
            this.displayRoles(results.roles);
        }

        resultsContainer.style.display = 'block';
    }

    /**
     * Display permissions results with associated roles
     */
    private displayPermissions(permissions: Permission[]) {
        const section = document.getElementById('permissionsSection');
        const list = document.getElementById('permissionsList');
        const count = document.getElementById('permissionCount');

        if (!section || !list || !count) return;

        list.innerHTML = '';

        permissions.forEach((perm, idx) => {
            const item = this.createPermissionItem(perm, idx);
            list.appendChild(item);
        });

        count.textContent = `(${permissions.length})`;
        section.style.display = 'block';
    }

    /**
     * Display roles results with their permissions
     */
    private displayRoles(roles: Role[]) {
        const section = document.getElementById('rolesSection');
        const list = document.getElementById('rolesList');
        const count = document.getElementById('roleCount');

        if (!section || !list || !count) return;

        list.innerHTML = '';

        roles.forEach((role, idx) => {
            const item = this.createRoleItem(role, idx);
            list.appendChild(item);
        });

        count.textContent = `(${roles.length})`;
        section.style.display = 'block';
    }

    /**
     * Create permission result item with associated roles
     */
    private createPermissionItem(perm: Permission, idx: number) {
        const div = document.createElement('div');
        div.className = 'result-item';
        div.dataset.index = String(idx);
        div.dataset.type = 'permission';

        const rolesHtml = perm.granted_by_roles && perm.granted_by_roles.length > 0
            ? `<div class="associated-roles">
                <span class="roles-label">Granted by:</span>
                ${perm.granted_by_roles.map(r =>
                    `<span class="role-chip" title="${this.escapeHtml(r.name)}">${this.escapeHtml(r.title)}</span>`
                ).join('')}
               </div>`
            : '<div class="no-roles">No roles grant this permission directly</div>';

        div.innerHTML = `
            <div class="result-name">${this.escapeHtml(perm.name)}</div>
            <div class="result-meta">
                <span class="result-badge service">${this.escapeHtml(perm.service)}</span>
                <span class="result-badge resource">${this.escapeHtml(perm.resource)}</span>
                <span class="result-badge action">${this.escapeHtml(perm.action)}</span>
                <span class="result-score">Match: ${(perm.score * 100).toFixed(0)}%</span>
            </div>
            ${rolesHtml}
        `;

        div.addEventListener('click', () => {
            this.selectItem(idx);
        });

        return div;
    }

    /**
     * Create role result item with sample permissions
     */
    private createRoleItem(role: Role, idx: number) {
        const div = document.createElement('div');
        div.className = 'result-item role-item';
        div.dataset.index = String(idx);
        div.dataset.type = 'role';

        const stageColor =
            role.stage === 'GA'
                ? '#4CAF50'
                : role.stage === 'BETA'
                  ? '#FF9800'
                  : role.stage === 'ALPHA'
                    ? '#2196F3'
                    : '#F44336';

        const permissionsHtml = role.sample_permissions && role.sample_permissions.length > 0
            ? `<div class="sample-permissions">
                <span class="perms-label">Includes:</span>
                ${role.sample_permissions.map(p =>
                    `<span class="perm-chip">${this.escapeHtml(p)}</span>`
                ).join('')}
                ${role.permission_count > 5 ? `<span class="more-perms">+${role.permission_count - 5} more</span>` : ''}
               </div>`
            : '';

        div.innerHTML = `
            <div class="result-name">${this.escapeHtml(role.name)}</div>
            <div class="role-title">${this.escapeHtml(role.title)}</div>
            <div class="role-description">${this.escapeHtml(role.description)}</div>
            <div class="result-meta">
                <span class="result-badge stage" style="background-color: ${stageColor}; color: white;">
                    ${this.escapeHtml(role.stage)}
                </span>
                <span class="result-badge">
                    ${role.permission_count} permission${role.permission_count !== 1 ? 's' : ''}
                </span>
                <span class="result-score">Match: ${(role.score * 100).toFixed(0)}%</span>
            </div>
            ${permissionsHtml}
        `;

        div.addEventListener('click', () => {
            this.selectItem(idx);
        });

        return div;
    }

    /**
     * Select a result item
     */
    private selectItem(index: number) {
        const items = document.querySelectorAll('.result-item');
        items.forEach((item) => item.classList.remove('selected'));

        const item = document.querySelector(`.result-item[data-index="${index}"]`);
        if (item) {
            item.classList.add('selected');
            item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }

        this.selectedIndex = index;
    }

    /**
     * Select next item
     */
    selectNext() {
        const items = document.querySelectorAll('.result-item');
        if (items.length === 0) return;

        const next = Math.min(this.selectedIndex + 1, items.length - 1);
        this.selectItem(next);
    }

    /**
     * Select previous item
     */
    selectPrevious() {
        const items = document.querySelectorAll('.result-item');
        if (items.length === 0) return;

        const prev = Math.max(this.selectedIndex - 1, 0);
        this.selectItem(prev);
    }

    /**
     * Get selected result name
     */
    getSelectedResult(): string | null {
        if (this.selectedIndex === -1) return null;

        const item = document.querySelector(`.result-item[data-index="${this.selectedIndex}"]`);
        if (!item) return null;

        const nameEl = item.querySelector('.result-name');
        return nameEl ? nameEl.textContent : null;
    }

    /**
     * Hide all state indicators
     */
    private hideAllStates() {
        const states = ['loadingState', 'emptyState', 'errorState', 'resultsContainer'];
        states.forEach((id) => {
            const el = document.getElementById(id);
            if (el) {
                el.style.display = 'none';
            }
        });
    }

    /**
     * Escape HTML special characters
     */
    private escapeHtml(text: string): string {
        const map: Record<string, string> = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#039;',
        };
        return text.replace(/[&<>"']/g, (char) => map[char]);
    }
}
