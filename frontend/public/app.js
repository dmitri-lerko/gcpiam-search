"use strict";
(() => {
  var __defProp = Object.defineProperty;
  var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
  var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);

  // public/api.ts
  var SearchClient = class {
    constructor(baseUrl) {
      __publicField(this, "baseUrl");
      __publicField(this, "cache", /* @__PURE__ */ new Map());
      __publicField(this, "abortController", null);
      this.baseUrl = baseUrl.replace(/\/$/, "");
    }
    /**
     * Search for permissions and roles
     */
    async search(query, mode = "fuzzy") {
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
          limit: "20"
        });
        const url = `${this.baseUrl}/search?${params}`;
        const response = await fetch(url, {
          method: "GET",
          headers: {
            "Content-Type": "application/json"
          },
          signal: this.abortController.signal
        });
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        const json = await response.json();
        const rawData = json.data || json;
        const results = {
          permissions: rawData.permissions || [],
          roles: rawData.roles || []
        };
        this.cache.set(cacheKey, results);
        return results;
      } catch (error) {
        if (error instanceof Error && error.name === "AbortError") {
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
    async getHealth() {
      try {
        const response = await fetch(`${this.baseUrl}/health`, {
          method: "GET"
        });
        return response.ok;
      } catch {
        return false;
      }
    }
  };

  // public/ui.ts
  var SearchUI = class {
    constructor() {
      __publicField(this, "selectedIndex", -1);
      __publicField(this, "currentResults", { permissions: [], roles: [] });
      this.initializeElements();
    }
    initializeElements() {
      const requiredIds = [
        "loadingState",
        "emptyState",
        "errorState",
        "resultsContainer",
        "permissionsSection",
        "rolesSection",
        "permissionsList",
        "rolesList",
        "permissionCount",
        "roleCount"
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
      const loadingState = document.getElementById("loadingState");
      if (loadingState) {
        loadingState.style.display = "block";
      }
      this.selectedIndex = -1;
    }
    /**
     * Show empty state
     */
    showEmptyState() {
      this.hideAllStates();
      const emptyState = document.getElementById("emptyState");
      if (emptyState) {
        emptyState.style.display = "block";
      }
      this.selectedIndex = -1;
    }
    /**
     * Show error state
     */
    showError() {
      this.hideAllStates();
      const errorState = document.getElementById("errorState");
      if (errorState) {
        errorState.style.display = "block";
      }
      this.selectedIndex = -1;
    }
    /**
     * Display search results
     */
    displayResults(results) {
      this.currentResults = results;
      this.hideAllStates();
      this.selectedIndex = -1;
      const resultsContainer = document.getElementById("resultsContainer");
      if (!resultsContainer) return;
      const permSection = document.getElementById("permissionsSection");
      const rolesSection = document.getElementById("rolesSection");
      if (permSection) permSection.style.display = "none";
      if (rolesSection) rolesSection.style.display = "none";
      if (results.permissions.length > 0) {
        this.displayPermissions(results.permissions);
      }
      if (results.roles.length > 0) {
        this.displayRoles(results.roles);
      }
      resultsContainer.style.display = "block";
    }
    /**
     * Display permissions results with associated roles
     */
    displayPermissions(permissions) {
      const section = document.getElementById("permissionsSection");
      const list = document.getElementById("permissionsList");
      const count = document.getElementById("permissionCount");
      if (!section || !list || !count) return;
      list.innerHTML = "";
      permissions.forEach((perm, idx) => {
        const item = this.createPermissionItem(perm, idx);
        list.appendChild(item);
      });
      count.textContent = `(${permissions.length})`;
      section.style.display = "block";
    }
    /**
     * Display roles results with their permissions
     */
    displayRoles(roles) {
      const section = document.getElementById("rolesSection");
      const list = document.getElementById("rolesList");
      const count = document.getElementById("roleCount");
      if (!section || !list || !count) return;
      list.innerHTML = "";
      roles.forEach((role, idx) => {
        const item = this.createRoleItem(role, idx);
        list.appendChild(item);
      });
      count.textContent = `(${roles.length})`;
      section.style.display = "block";
    }
    /**
     * Create permission result item with associated roles
     */
    createPermissionItem(perm, idx) {
      const div = document.createElement("div");
      div.className = "result-item";
      div.dataset.index = String(idx);
      div.dataset.type = "permission";
      const rolesHtml = perm.granted_by_roles && perm.granted_by_roles.length > 0 ? `<div class="associated-roles">
                <span class="roles-label">Granted by:</span>
                ${perm.granted_by_roles.map(
        (r) => `<span class="role-chip" title="${this.escapeHtml(r.name)}">${this.escapeHtml(r.title)}</span>`
      ).join("")}
               </div>` : '<div class="no-roles">No roles grant this permission directly</div>';
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
      div.addEventListener("click", () => {
        this.selectItem(idx);
      });
      return div;
    }
    /**
     * Create role result item with sample permissions
     */
    createRoleItem(role, idx) {
      const div = document.createElement("div");
      div.className = "result-item role-item";
      div.dataset.index = String(idx);
      div.dataset.type = "role";
      const stageColor = role.stage === "GA" ? "#4CAF50" : role.stage === "BETA" ? "#FF9800" : role.stage === "ALPHA" ? "#2196F3" : "#F44336";
      const permissionsHtml = role.sample_permissions && role.sample_permissions.length > 0 ? `<div class="sample-permissions">
                <span class="perms-label">Includes:</span>
                ${role.sample_permissions.map(
        (p) => `<span class="perm-chip">${this.escapeHtml(p)}</span>`
      ).join("")}
                ${role.permission_count > 5 ? `<span class="more-perms">+${role.permission_count - 5} more</span>` : ""}
               </div>` : "";
      div.innerHTML = `
            <div class="result-name">${this.escapeHtml(role.name)}</div>
            <div class="role-title">${this.escapeHtml(role.title)}</div>
            <div class="role-description">${this.escapeHtml(role.description)}</div>
            <div class="result-meta">
                <span class="result-badge stage" style="background-color: ${stageColor}; color: white;">
                    ${this.escapeHtml(role.stage)}
                </span>
                <span class="result-badge">
                    ${role.permission_count} permission${role.permission_count !== 1 ? "s" : ""}
                </span>
                <span class="result-score">Match: ${(role.score * 100).toFixed(0)}%</span>
            </div>
            ${permissionsHtml}
        `;
      div.addEventListener("click", () => {
        this.selectItem(idx);
      });
      return div;
    }
    /**
     * Select a result item
     */
    selectItem(index) {
      const items = document.querySelectorAll(".result-item");
      items.forEach((item2) => item2.classList.remove("selected"));
      const item = document.querySelector(`.result-item[data-index="${index}"]`);
      if (item) {
        item.classList.add("selected");
        item.scrollIntoView({ behavior: "smooth", block: "nearest" });
      }
      this.selectedIndex = index;
    }
    /**
     * Select next item
     */
    selectNext() {
      const items = document.querySelectorAll(".result-item");
      if (items.length === 0) return;
      const next = Math.min(this.selectedIndex + 1, items.length - 1);
      this.selectItem(next);
    }
    /**
     * Select previous item
     */
    selectPrevious() {
      const items = document.querySelectorAll(".result-item");
      if (items.length === 0) return;
      const prev = Math.max(this.selectedIndex - 1, 0);
      this.selectItem(prev);
    }
    /**
     * Get selected result name
     */
    getSelectedResult() {
      if (this.selectedIndex === -1) return null;
      const item = document.querySelector(`.result-item[data-index="${this.selectedIndex}"]`);
      if (!item) return null;
      const nameEl = item.querySelector(".result-name");
      return nameEl ? nameEl.textContent : null;
    }
    /**
     * Hide all state indicators
     */
    hideAllStates() {
      const states = ["loadingState", "emptyState", "errorState", "resultsContainer"];
      states.forEach((id) => {
        const el = document.getElementById(id);
        if (el) {
          el.style.display = "none";
        }
      });
    }
    /**
     * Escape HTML special characters
     */
    escapeHtml(text) {
      const map = {
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#039;"
      };
      return text.replace(/[&<>"']/g, (char) => map[char]);
    }
  };

  // public/search.ts
  var SearchManager = class {
    constructor(config = {}) {
      __publicField(this, "currentMode", "fuzzy");
      __publicField(this, "debounceMs", 150);
      __publicField(this, "resultLimit", 20);
      __publicField(this, "fuzzyThreshold", 0.5);
      __publicField(this, "debounceTimer", null);
      __publicField(this, "lastQuery", "");
      this.debounceMs = config.debounceMs ?? 150;
      this.resultLimit = config.resultLimit ?? 20;
      this.fuzzyThreshold = config.fuzzyThreshold ?? 0.5;
    }
    /**
     * Set the current search mode
     */
    setMode(mode) {
      this.currentMode = mode;
    }
    /**
     * Get the current search mode
     */
    getCurrentMode() {
      return this.currentMode;
    }
    /**
     * Set debounce delay
     */
    setDebounceMs(ms) {
      this.debounceMs = ms;
    }
    /**
     * Set result limit
     */
    setResultLimit(limit) {
      this.resultLimit = limit;
    }
    /**
     * Set fuzzy threshold (0-1)
     */
    setFuzzyThreshold(threshold) {
      this.fuzzyThreshold = Math.max(0, Math.min(1, threshold));
    }
    /**
     * Debounce a search operation
     */
    debounceSearch(callback, query) {
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
    cancelDebounce() {
      if (this.debounceTimer !== null) {
        clearTimeout(this.debounceTimer);
        this.debounceTimer = null;
      }
    }
    /**
     * Get last query that was searched
     */
    getLastQuery() {
      return this.lastQuery;
    }
    /**
     * Validate and normalize a search query
     */
    validateQuery(query) {
      return query.trim().toLowerCase().slice(0, 100);
    }
    /**
     * Get search configuration for API call
     */
    getSearchConfig() {
      return {
        mode: this.currentMode,
        limit: this.resultLimit,
        fuzzyThreshold: this.fuzzyThreshold
      };
    }
  };

  // public/app.ts
  var CONFIG = {
    API_BASE_URL: "http://127.0.0.1:8000/api/v1",
    SEARCH_DEBOUNCE_MS: 150,
    RESULT_LIMIT: 20,
    FUZZY_THRESHOLD: 0.5
  };
  async function initializeApp() {
    try {
      const apiClient = new SearchClient(CONFIG.API_BASE_URL);
      const ui = new SearchUI();
      const searchManager = new SearchManager({
        debounceMs: CONFIG.SEARCH_DEBOUNCE_MS,
        resultLimit: CONFIG.RESULT_LIMIT,
        fuzzyThreshold: CONFIG.FUZZY_THRESHOLD
      });
      const searchInput = document.getElementById("searchInput");
      const clearBtn = document.getElementById("clearBtn");
      const modeButtons = document.querySelectorAll(".mode-btn");
      searchInput.addEventListener("input", async (e) => {
        const query = e.target.value.trim();
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
          console.error("Search error:", error);
          ui.showError();
        }
      });
      clearBtn.addEventListener("click", () => {
        searchInput.value = "";
        searchInput.focus();
        ui.showEmptyState();
      });
      modeButtons.forEach((btn) => {
        btn.addEventListener("click", () => {
          modeButtons.forEach((b) => b.classList.remove("active"));
          btn.classList.add("active");
          const mode = btn.getAttribute("data-mode");
          searchManager.setMode(mode);
          if (searchInput.value.trim()) {
            searchInput.dispatchEvent(new Event("input"));
          }
        });
      });
      document.addEventListener("keydown", (e) => {
        if (e.key === "/" && document.activeElement !== searchInput) {
          e.preventDefault();
          searchInput.focus();
        }
        if (e.key === "ArrowDown" && document.activeElement === searchInput) {
          e.preventDefault();
          ui.selectNext();
        }
        if (e.key === "ArrowUp" && document.activeElement === searchInput) {
          e.preventDefault();
          ui.selectPrevious();
        }
        if (e.key === "Enter" && document.activeElement === searchInput) {
          const selected = ui.getSelectedResult();
          if (selected) {
            e.preventDefault();
            copyToClipboard(selected);
          }
        }
        if (e.key === "Escape" && document.activeElement === searchInput) {
          clearBtn.click();
        }
      });
      searchInput.focus();
      console.log("\u2713 GCP IAM Search initialized");
    } catch (error) {
      console.error("Failed to initialize app:", error);
      document.body.innerHTML = '<div style="padding: 2rem; color: red;"><h1>Failed to initialize application</h1><p>Check console for details.</p></div>';
    }
  }
  function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(
      () => {
        const msg = document.createElement("div");
        msg.textContent = "\u2713 Copied!";
        msg.style.cssText = "position:fixed;top:20px;right:20px;background:#4CAF50;color:white;padding:10px 16px;border-radius:6px;z-index:9999;font-size:14px;";
        document.body.appendChild(msg);
        setTimeout(() => msg.remove(), 2e3);
      },
      () => {
        console.error("Failed to copy to clipboard");
      }
    );
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initializeApp);
  } else {
    initializeApp();
  }
})();
//# sourceMappingURL=app.js.map
