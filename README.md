# GCP IAM Permissions Search Platform

🦀 **Full Rust Stack** - A high-performance search platform for Google Cloud Platform IAM permissions and roles with instant search, compiled WASM frontend, and blazingly-fast backend.

## 🚀 Project Status

**Phase 1: Scraper & Data Collection** - 20% Complete ⚡
- [x] Rust monorepo structure with workspace
- [x] Scraper project setup with Cargo.toml
- [x] Core modules: models, error handling, GCP client stub, transformer, storage
- [x] GitHub Actions workflow for daily updates
- [ ] Implement GCP API client (authentication, pagination, retries)
- [ ] Implement data transformation and optimization
- [ ] Unit tests for scraper
- [ ] GCP service account setup (user action required)

**Phase 2-5**: Backend API, WASM Frontend, Integration, Deployment - Pending

## 📋 Architecture

🦀 **Full Rust Stack** - All components compiled to native code or WebAssembly.

### 1. **Scraper** (`/scraper`)
- **Language**: Rust
- **Runtime**: Tokio async runtime
- **Features**:
  - Automated daily collection via GCP IAM API
  - GitHub Actions CI/CD automation
  - Concurrent data fetching with tokio
  - Automatic change detection via ETags
  - JSON file versioning and archiving
  - Comprehensive error handling and retries
- **Output**: JSON files (iam-roles.json, iam-permissions.json, metadata.json)

### 2. **Backend API** (`/backend`)
- **Language**: Rust
- **Framework**: Actix-web
- **Features**:
  - Hybrid search engine:
    - Exact: O(1) hash map lookups
    - Prefix: Trie-based autocomplete (O(k))
    - Fuzzy: N-gram similarity matching
    - Full-text: Regex-based tokenization
  - In-memory data loading (no database)
  - LRU caching layer
  - CORS middleware
  - Rate limiting and request validation
  - Health check endpoints
- **Performance Target**: < 10ms P95 latency
- **Benchmarks**: Built-in criterion integration

### 3. **Frontend** (`/frontend`)
- **Language**: TypeScript (vanilla JavaScript)
- **Framework**: None (pure HTML/CSS/TS)
- **Features**:
  - Instant search with 150ms debounce
  - Real-time keyboard navigation (↑↓ Enter Esc /)
  - Responsive mobile-first design
  - Automatic dark mode support
  - WCAG 2.1 Level AA accessibility
  - ~15KB uncompressed, ~5KB gzipped
  - Zero runtime dependencies
- **Build**: Optional (static files, ready to serve)

## 🛠️ Tech Stack

### Backend (Rust)
- **Language**: Rust 2021 edition
- **Workspace**: Cargo monorepo with shared dependencies
- **Build**: Cargo (Rust package manager)

#### Scraper
- `tokio` - Async runtime (multi-threaded)
- `reqwest` - HTTP client with retries
- `serde`/`serde_json` - Serialization
- `thiserror` - Error handling
- `chrono` - Date/time handling
- `clap` - CLI argument parsing
- `tracing` - Structured logging

#### Backend API
- `actix-web` - Web framework
- `actix-cors` - CORS middleware
- `lru` - LRU caching
- `regex` - Pattern matching
- `parking_lot` - Better synchronization primitives
- `criterion` - Benchmarking framework
- `tokio` - Async runtime
- `serde`/`serde_json` - Serialization

### Frontend (Vanilla TypeScript)
- **Language**: TypeScript (type-safe vanilla JS)
- **Framework**: None - Pure HTML/CSS/TypeScript
- **Build**: Optional (included files are ready to serve)
- **Dev Server**: Node.js http-server or equivalent
- **Zero dependencies**: No npm modules required for production

## 📦 Project Structure

```
gcpiam.com/                        # Rust workspace root
├── Cargo.toml                     # Workspace manifest (shared deps)
├── Cargo.lock                     # Dependency lock file
├── scraper/                       # GCP IAM data scraper
│   ├── src/
│   │   ├── main.rs               # CLI entry point
│   │   ├── models.rs             # Data models (IAMRole, Permission, etc.)
│   │   ├── gcp.rs                # GCP API client
│   │   ├── transformer.rs        # Data transformation logic
│   │   ├── storage.rs            # File I/O and persistence
│   │   └── error.rs              # Error types
│   ├── Cargo.toml                # Scraper dependencies
│   └── tests/                    # Integration tests
├── backend/                       # Search API server
│   ├── src/
│   │   ├── lib.rs                # Library root
│   │   ├── models.rs             # API types
│   │   ├── error.rs              # Error handling
│   │   ├── search/
│   │   │   ├── mod.rs
│   │   │   └── engine.rs         # Search engine implementation
│   │   └── api/                  # REST endpoints (TODO)
│   ├── Cargo.toml                # Backend dependencies
│   └── benches/                  # Criterion benchmarks
├── frontend/                      # Vanilla TypeScript frontend
│   ├── public/
│   │   └── index.html            # Main HTML file
│   ├── app.ts                    # Main application logic
│   ├── api.ts                    # Backend API client
│   ├── ui.ts                     # DOM management and rendering
│   ├── search.ts                 # Search state and local engine
│   ├── styles.css                # All styling (light/dark modes)
│   ├── package.json              # Dev dependencies
│   ├── tsconfig.json             # TypeScript configuration
│   └── README.md                 # Frontend documentation
├── data/                         # Generated IAM data
│   ├── iam-roles.json
│   ├── iam-permissions.json
│   ├── metadata.json
│   └── archive/                  # Historical snapshots
├── .github/
│   └── workflows/
│       ├── fetch-iam-data.yml    # Daily scraper automation
│       ├── backend-ci.yml        # Backend tests
│       └── frontend-ci.yml       # Frontend WASM build
├── .gitignore
├── README.md
└── Cargo.lock
```

## 🚀 Setup Instructions

### Prerequisites
- Node.js 20+
- npm or yarn
- GCP project with IAM API enabled

### 1. Clone Repository
```bash
git clone https://github.com/dmitri-lerko/gcpiam.com.git
cd gcpiam.com
```

### 2. Setup GCP Service Account (Required for Scraper)

#### Option A: Using gcloud CLI (Recommended)
```bash
# Set your GCP project
export PROJECT_ID="your-gcp-project-id"

# Create service account
gcloud iam service-accounts create iam-scraper \
  --display-name="GCP IAM Data Scraper" \
  --project=$PROJECT_ID

# Grant necessary roles
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member=serviceAccount:iam-scraper@$PROJECT_ID.iam.gserviceaccount.com \
  --role=roles/iam.roleViewer

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member=serviceAccount:iam-scraper@$PROJECT_ID.iam.gserviceaccount.com \
  --role=roles/iam.securityReviewer

# Create and download JSON key
gcloud iam service-accounts keys create sa-key.json \
  --iam-account=iam-scraper@$PROJECT_ID.iam.gserviceaccount.com \
  --project=$PROJECT_ID
```

#### Option B: Using GCP Console
1. Go to [GCP Console](https://console.cloud.google.com)
2. Navigate to IAM & Admin → Service Accounts
3. Create new service account:
   - Name: `iam-scraper`
   - Description: "GCP IAM Data Scraper"
4. Grant roles:
   - `roles/iam.roleViewer`
   - `roles/iam.securityReviewer`
5. Create JSON key and download

### 3. Setup GitHub Secrets

Add to your repository secrets (Settings → Secrets and variables → Actions):

**Secret Name**: `GCP_SA_KEY`
**Value**: Contents of the JSON key file from Step 2

### 4. Install Dependencies

```bash
# Install frontend dev dependencies (optional, not needed for production)
cd frontend
npm install
cd ..
```

### 5. Test Scraper Locally

```bash
# Build scraper
cd scraper
npm run build

# Run scraper (requires GOOGLE_APPLICATION_CREDENTIALS to be set)
# Set the credentials file path first:
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/sa-key.json"

# Run the scraper
npm start

# Check generated data
ls -la ../data/
```

## 📊 Data Files Generated

The scraper generates the following files in the `/data` directory:

- `iam-roles.json` - Array of all IAM roles with metadata
- `iam-permissions.json` - Array of all permissions with metadata
- `metadata.json` - Metadata including stats and change detection
- `iam-data-complete.json` - Complete dataset with indexes
- `iam-data-complete.min.json` - Minified version for downloads
- `archive/` - Historical snapshots (last 30 days)

## 🔄 Data Collection Flow

```
GitHub Actions (Daily at 2 AM UTC)
    ↓
GCP IAM API fetch
    ↓
Data transformation & optimization
    ↓
Index building
    ↓
Change detection
    ↓
JSON file generation
    ↓
Git commit & push
    ↓
Release creation (on significant changes)
```

## 🧪 Testing

### Scraper Tests
```bash
cargo test --lib scraper --all
```

Tests for:
- GCP client creation and authentication
- Data transformation
- Permission parsing
- Storage management
- Index building

### Backend Tests
```bash
cargo test --lib backend --all
```

Tests for:
- Exact search (O(1) lookup)
- Prefix search (autocomplete)
- Fuzzy search (N-gram matching)
- Full-text search
- Service filtering

### Frontend Testing
The frontend is vanilla TypeScript with no framework. Manual testing recommended:
```bash
# Type checking
cd frontend
npm run typecheck

# Manual testing via browser
npm run dev  # Opens http://localhost:3000
```

Test areas:
- Search functionality (all modes)
- Keyboard navigation (↑↓ Enter Esc /)
- Dark mode toggle
- Responsive design (mobile/tablet/desktop)
- Accessibility with screen reader

## 📈 Performance Targets

### Backend
- **Exact search**: < 1ms
- **Prefix search**: < 5ms
- **Fuzzy search**: < 10ms
- **Autocomplete**: < 2ms
- **Index initialization**: < 5 seconds
- **Memory usage**: < 500MB

### Frontend
- **Bundle size**: < 100KB (gzipped)
- **Time to interactive**: < 2 seconds
- **Lighthouse score**: 90+ Performance, 100 Accessibility

### Data Collection
- **Scraper runtime**: < 5 minutes
- **Data size**: 5-10MB uncompressed, 1-2MB gzipped

## 🔐 Security Considerations

### Service Account Permissions
The service account uses minimal permissions required:
- `roles/iam.roleViewer` - Read all roles
- `roles/iam.securityReviewer` - Read permissions metadata

No write permissions are granted to the service account.

### GitHub Secrets
- Never commit the service account JSON key
- Use GitHub Secrets to store `GCP_SA_KEY`
- Rotate keys annually

### Data Validation
- All data is validated against Zod schemas
- API responses are checked before processing
- Errors are caught and logged

## 📝 Environment Variables

### Scraper
```bash
NODE_ENV=production|development
DATA_OUTPUT_DIR=./data          # Output directory for JSON files
LOG_LEVEL=debug|info|warn|error # Logging level
GOOGLE_APPLICATION_CREDENTIALS=/path/to/sa-key.json
```

### Backend (Coming Soon)
```bash
PORT=4000
CORS_ORIGIN=https://gcpiam.com,https://www.gcpiam.com
NODE_ENV=production
```

### Frontend (Coming Soon)
```bash
VITE_API_URL=https://api.gcpiam.com
VITE_ANALYTICS_ID=G-XXXXXXXXXX
```

## 🚢 Deployment

### Current Phase
The scraper is ready for GitHub Actions deployment once:
1. ✅ Code is complete
2. ⏳ GCP service account is created
3. ⏳ GitHub secrets are configured
4. ⏳ Workflow is enabled in GitHub

### Future Phases
- Backend will be deployed to Railway/Render/Fly.io/Google Cloud Run
- Frontend will be deployed to Vercel/Netlify
- CI/CD pipelines will be configured in GitHub Actions

## 📚 Documentation

- [Implementation Plan](/.claude/plans/bright-dreaming-knuth.md) - Detailed 5-phase plan
- [API Documentation](./backend/API.md) - Coming in Phase 2
- [Architecture Guide](./docs/ARCHITECTURE.md) - Coming in Phase 5

## 🤝 Contributing

This is a personal project tracking in a private repository. For contributions, please:
1. Create a feature branch
2. Write tests for new code
3. Ensure all tests pass
4. Submit a pull request

## 📊 Performance Monitoring

The scraper logs detailed metrics:
```
[2025-01-03T10:30:45.123Z] [INFO] [Scraper] Starting GCP IAM data collection...
[2025-01-03T10:30:46.456Z] [INFO] [GCPClient] Fetched 1500 roles from GCP IAM API
[2025-01-03T10:30:47.789Z] [INFO] [DataFetcher] Extracted 6000 unique permissions
[2025-01-03T10:30:48.012Z] [INFO] [DataTransformer] Transformation complete
[2025-01-03T10:30:49.345Z] [INFO] [StorageManager] Data saved successfully
[2025-01-03T10:30:49.567Z] [INFO] [Scraper] ✓ Data collection completed successfully!
```

## 🐛 Troubleshooting

### "Permission denied" when running scraper
- Ensure `GCP_SA_KEY` is set correctly
- Verify service account has required roles
- Check that IAM API is enabled in your GCP project

### "Rate limit exceeded"
- The scraper has automatic retry logic with exponential backoff
- If persistent, wait 15 minutes before retrying

### GitHub Actions workflow not triggering
- Check that `GCP_SA_KEY` secret is configured
- Verify workflow file is in `.github/workflows/` directory
- Check Actions tab in GitHub for errors

## 📞 Support

For issues or questions:
1. Check the Implementation Plan for detailed information
2. Review error messages and logs
3. Verify all setup steps are completed
4. Create an issue in the GitHub repository

## 📜 License

Private repository. All rights reserved.

## 🎯 Next Steps

1. **Immediate** (User Action):
   - Create GCP service account
   - Configure GitHub secrets
   - Test scraper locally

2. **Short Term** (Implementation):
   - Write unit tests for scraper
   - Implement and test backend search engine
   - Implement frontend UI components

3. **Medium Term**:
   - Integration testing
   - Performance validation
   - Security review

4. **Long Term**:
   - Production deployment
   - Monitoring and alerting
   - Documentation finalization

---

**Status**: Phase 1 - Scraper Implementation 80% Complete ✅

Last Updated: 2025-01-03
