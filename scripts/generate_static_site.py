#!/usr/bin/env python3
"""
GCP IAM Static Site Generator

Fetches all GCP IAM roles and permissions, generates:
- JSON data file for the search backend
- Static HTML pages for each role and permission
- sitemap.xml for SEO
"""

import json
import subprocess
import sys
import os
from datetime import datetime
from pathlib import Path
from html import escape
from urllib.parse import quote

# Configuration
BASE_URL = "https://gcpiam.com"
OUTPUT_DIR = Path(__file__).parent.parent / "data"
STATIC_DIR = OUTPUT_DIR / "static"
ROLES_DIR = STATIC_DIR / "roles"
PERMISSIONS_DIR = STATIC_DIR / "permissions"


def get_token():
    """Get GCP access token from gcloud."""
    result = subprocess.run(['gcloud', 'auth', 'print-access-token'], capture_output=True, text=True)
    if result.returncode != 0:
        print("Error: Could not get access token. Run 'gcloud auth login' first.", file=sys.stderr)
        sys.exit(1)
    return result.stdout.strip()


def fetch_url(url, token, method='GET', data=None):
    """Fetch URL with authentication."""
    import urllib.request

    req = urllib.request.Request(url, method=method)
    req.add_header('Authorization', f'Bearer {token}')
    if data:
        req.add_header('Content-Type', 'application/json')
        req.data = json.dumps(data).encode()

    with urllib.request.urlopen(req) as response:
        return json.loads(response.read().decode())


def fetch_all_roles(token):
    """Fetch all predefined roles from GCP IAM API."""
    all_roles = []
    page_token = None
    page_num = 0

    while True:
        page_num += 1
        url = "https://iam.googleapis.com/v1/roles?pageSize=1000&view=FULL"
        if page_token:
            url += f"&pageToken={page_token}"

        data = fetch_url(url, token)
        roles = data.get('roles', [])
        all_roles.extend(roles)
        print(f"  Fetched page {page_num}: {len(roles)} roles (total: {len(all_roles)})", file=sys.stderr)

        page_token = data.get('nextPageToken')
        if not page_token:
            break

    return all_roles


def fetch_permission_metadata(token, permissions_batch):
    """Fetch metadata for a batch of permissions."""
    # Note: The queryTestablePermissions API requires a resource context
    # For simplicity, we'll extract what we can from role data
    return {}


def build_dataset(roles):
    """Build the complete dataset with bidirectional references."""
    # Build permission -> roles mapping
    permission_to_roles = {}
    all_permissions = set()

    for role in roles:
        role_name = role.get('name', '')
        perms = role.get('includedPermissions', [])
        for perm in perms:
            all_permissions.add(perm)
            if perm not in permission_to_roles:
                permission_to_roles[perm] = []
            permission_to_roles[perm].append({
                'name': role_name,
                'title': role.get('title', ''),
                'stage': role.get('stage', 'GA'),
            })

    # Build roles data
    roles_data = []
    for role in roles:
        roles_data.append({
            'name': role.get('name', ''),
            'title': role.get('title', ''),
            'description': role.get('description', ''),
            'stage': role.get('stage', 'GA'),
            'included_permissions': role.get('includedPermissions', []),
            'etag': role.get('etag', ''),
        })

    # Build permissions data with roles that grant them
    permissions_data = []
    for perm in sorted(all_permissions):
        parts = perm.split('.')
        permissions_data.append({
            'name': perm,
            'service': parts[0] if parts else '',
            'resource': parts[1] if len(parts) > 1 else '',
            'action': parts[2] if len(parts) > 2 else '',
            'granted_by_roles': permission_to_roles.get(perm, []),
        })

    return {
        'roles': roles_data,
        'permissions': permissions_data,
        'metadata': {
            'total_roles': len(roles_data),
            'total_permissions': len(permissions_data),
            'last_updated': datetime.utcnow().isoformat() + 'Z',
            'source': 'Google Cloud IAM API',
        }
    }


def generate_html_head(title, description, canonical_path):
    """Generate HTML head section with SEO meta tags."""
    return f'''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{escape(title)} | GCP IAM Reference</title>
    <meta name="description" content="{escape(description)}">
    <link rel="canonical" href="{BASE_URL}{canonical_path}">
    <meta property="og:title" content="{escape(title)}">
    <meta property="og:description" content="{escape(description)}">
    <meta property="og:url" content="{BASE_URL}{canonical_path}">
    <meta property="og:type" content="website">
    <style>
        :root {{
            --bg-primary: #ffffff;
            --bg-secondary: #f5f5f5;
            --text-primary: #1a1a1a;
            --text-secondary: #666666;
            --border-color: #e0e0e0;
            --accent: #1a73e8;
        }}
        @media (prefers-color-scheme: dark) {{
            :root {{
                --bg-primary: #1a1a1a;
                --bg-secondary: #2d2d2d;
                --text-primary: #e0e0e0;
                --text-secondary: #b0b0b0;
                --border-color: #404040;
                --accent: #8ab4f8;
            }}
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            padding: 2rem;
            max-width: 1000px;
            margin: 0 auto;
        }}
        h1 {{ color: var(--accent); margin-bottom: 0.5rem; word-break: break-word; }}
        h2 {{ margin-top: 2rem; margin-bottom: 1rem; border-bottom: 2px solid var(--border-color); padding-bottom: 0.5rem; }}
        .subtitle {{ color: var(--text-secondary); margin-bottom: 1rem; }}
        .description {{ background: var(--bg-secondary); padding: 1rem; border-radius: 8px; margin-bottom: 1.5rem; }}
        .badge {{ display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 0.85rem; margin-right: 0.5rem; }}
        .badge-ga {{ background: #e8f5e9; color: #2e7d32; }}
        .badge-beta {{ background: #fff3e0; color: #e65100; }}
        .badge-alpha {{ background: #e3f2fd; color: #1565c0; }}
        .badge-deprecated {{ background: #ffebee; color: #c62828; }}
        .list {{ list-style: none; }}
        .list li {{ padding: 0.75rem 1rem; border-bottom: 1px solid var(--border-color); }}
        .list li:hover {{ background: var(--bg-secondary); }}
        .list a {{ color: var(--accent); text-decoration: none; }}
        .list a:hover {{ text-decoration: underline; }}
        .role-title {{ color: var(--text-secondary); font-size: 0.9rem; }}
        .count {{ color: var(--text-secondary); font-size: 0.9rem; }}
        .back-link {{ display: inline-block; margin-bottom: 1rem; color: var(--accent); text-decoration: none; }}
        .back-link:hover {{ text-decoration: underline; }}
        @media (prefers-color-scheme: dark) {{
            .badge-ga {{ background: #1b3d20; color: #81c784; }}
            .badge-beta {{ background: #3d2f1f; color: #ffb74d; }}
            .badge-alpha {{ background: #1e3a5f; color: #90caf9; }}
            .badge-deprecated {{ background: #3d1f1f; color: #ef9a9a; }}
        }}
    </style>
</head>
<body>
    <a href="/" class="back-link">&larr; Back to Search</a>
'''


def generate_html_footer():
    """Generate HTML footer."""
    return '''
    <footer style="margin-top: 3rem; padding-top: 1rem; border-top: 1px solid var(--border-color); color: var(--text-secondary); font-size: 0.85rem;">
        <p>Data sourced from <a href="https://cloud.google.com/iam/docs/understanding-roles" style="color: var(--accent);">Google Cloud IAM API</a>.
        Updated daily.</p>
    </footer>
</body>
</html>'''


def get_stage_badge(stage):
    """Get HTML badge for stage."""
    stage_lower = stage.lower() if stage else 'ga'
    badge_class = f'badge-{stage_lower}' if stage_lower in ['ga', 'beta', 'alpha', 'deprecated'] else 'badge-ga'
    return f'<span class="badge {badge_class}">{escape(stage or "GA")}</span>'


def generate_permission_page(perm_data):
    """Generate static HTML page for a permission."""
    name = perm_data['name']
    service = perm_data['service']
    resource = perm_data['resource']
    action = perm_data['action']
    roles = perm_data.get('granted_by_roles', [])

    title = name
    description = f"GCP IAM permission {name} - granted by {len(roles)} roles. Service: {service}, Resource: {resource}, Action: {action}."

    html = generate_html_head(title, description, f"/permissions/{quote(name)}")

    html += f'''
    <h1>{escape(name)}</h1>
    <p class="subtitle">GCP IAM Permission</p>

    <div class="description">
        <p><strong>Service:</strong> {escape(service)}</p>
        <p><strong>Resource:</strong> {escape(resource)}</p>
        <p><strong>Action:</strong> {escape(action)}</p>
    </div>

    <h2>Roles that grant this permission <span class="count">({len(roles)})</span></h2>
'''

    if roles:
        html += '<ul class="list">\n'
        for role in sorted(roles, key=lambda r: r['name']):
            role_path = role['name'].replace('roles/', '')
            html += f'''    <li>
        <a href="/roles/{quote(role_path)}">{escape(role['name'])}</a>
        {get_stage_badge(role.get('stage', 'GA'))}
        <span class="role-title">{escape(role.get('title', ''))}</span>
    </li>\n'''
        html += '</ul>\n'
    else:
        html += '<p style="color: var(--text-secondary);">No predefined roles grant this permission directly.</p>\n'

    html += generate_html_footer()
    return html


def generate_role_page(role_data, permission_to_roles):
    """Generate static HTML page for a role."""
    name = role_data['name']
    title = role_data.get('title', '')
    description = role_data.get('description', '')
    stage = role_data.get('stage', 'GA')
    permissions = role_data.get('included_permissions', [])

    meta_description = f"{title} - {description[:150]}..." if len(description) > 150 else f"{title} - {description}"

    role_path = name.replace('roles/', '')
    html = generate_html_head(f"{title} ({name})", meta_description, f"/roles/{quote(role_path)}")

    html += f'''
    <h1>{escape(name)}</h1>
    <p class="subtitle">{escape(title)} {get_stage_badge(stage)}</p>

    <div class="description">
        <p>{escape(description)}</p>
    </div>

    <h2>Included Permissions <span class="count">({len(permissions)})</span></h2>
'''

    if permissions:
        html += '<ul class="list">\n'
        for perm in sorted(permissions):
            html += f'    <li><a href="/permissions/{quote(perm)}">{escape(perm)}</a></li>\n'
        html += '</ul>\n'
    else:
        html += '<p style="color: var(--text-secondary);">This role has no permissions.</p>\n'

    html += generate_html_footer()
    return html


def generate_sitemap(roles, permissions):
    """Generate sitemap.xml."""
    now = datetime.utcnow().strftime('%Y-%m-%d')

    xml = '''<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>{base}/</loc>
        <lastmod>{date}</lastmod>
        <changefreq>daily</changefreq>
        <priority>1.0</priority>
    </url>
'''.format(base=BASE_URL, date=now)

    # Add role pages
    for role in roles:
        role_path = role['name'].replace('roles/', '')
        xml += f'''    <url>
        <loc>{BASE_URL}/roles/{quote(role_path)}</loc>
        <lastmod>{now}</lastmod>
        <changefreq>weekly</changefreq>
        <priority>0.8</priority>
    </url>
'''

    # Add permission pages
    for perm in permissions:
        xml += f'''    <url>
        <loc>{BASE_URL}/permissions/{quote(perm['name'])}</loc>
        <lastmod>{now}</lastmod>
        <changefreq>weekly</changefreq>
        <priority>0.7</priority>
    </url>
'''

    xml += '</urlset>\n'
    return xml


def generate_index_page(metadata):
    """Generate index page with stats."""
    html = generate_html_head(
        "GCP IAM Permissions & Roles Search",
        f"Search {metadata['total_permissions']} GCP IAM permissions across {metadata['total_roles']} roles. Find which roles grant specific permissions.",
        "/"
    )

    html += f'''
    <h1>GCP IAM Search</h1>
    <p class="subtitle">Search Google Cloud IAM roles and permissions</p>

    <div class="description">
        <p><strong>{metadata['total_permissions']:,}</strong> permissions across <strong>{metadata['total_roles']:,}</strong> predefined roles</p>
        <p>Last updated: {metadata['last_updated'][:10]}</p>
    </div>

    <div id="app">
        <noscript>
            <p>Enable JavaScript for interactive search, or browse:</p>
            <ul>
                <li><a href="/roles/">Browse all roles</a></li>
                <li><a href="/permissions/">Browse all permissions</a></li>
            </ul>
        </noscript>
    </div>

    <script src="/app.js" type="module"></script>
'''
    html += generate_html_footer()
    return html


def main():
    print("GCP IAM Static Site Generator", file=sys.stderr)
    print("=" * 40, file=sys.stderr)

    # Create output directories
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    STATIC_DIR.mkdir(parents=True, exist_ok=True)
    ROLES_DIR.mkdir(parents=True, exist_ok=True)
    PERMISSIONS_DIR.mkdir(parents=True, exist_ok=True)

    # Get token
    print("\n1. Authenticating with GCP...", file=sys.stderr)
    token = get_token()
    print("   OK", file=sys.stderr)

    # Fetch roles
    print("\n2. Fetching roles from GCP IAM API...", file=sys.stderr)
    roles = fetch_all_roles(token)
    print(f"   Fetched {len(roles)} roles", file=sys.stderr)

    # Build dataset
    print("\n3. Building dataset...", file=sys.stderr)
    dataset = build_dataset(roles)
    print(f"   {dataset['metadata']['total_roles']} roles", file=sys.stderr)
    print(f"   {dataset['metadata']['total_permissions']} permissions", file=sys.stderr)

    # Save JSON data
    print("\n4. Saving JSON data...", file=sys.stderr)
    json_path = OUTPUT_DIR / "iam-data.json"
    with open(json_path, 'w') as f:
        json.dump(dataset, f, indent=2)
    print(f"   Saved to {json_path}", file=sys.stderr)

    # Build permission->roles lookup for role pages
    perm_to_roles = {p['name']: p['granted_by_roles'] for p in dataset['permissions']}

    # Generate static pages
    print("\n5. Generating static HTML pages...", file=sys.stderr)

    # Permission pages
    print("   Generating permission pages...", file=sys.stderr)
    for i, perm in enumerate(dataset['permissions']):
        html = generate_permission_page(perm)
        # Use URL-safe filename
        filename = perm['name'].replace('/', '_') + '.html'
        filepath = PERMISSIONS_DIR / filename
        with open(filepath, 'w') as f:
            f.write(html)
        if (i + 1) % 1000 == 0:
            print(f"      {i + 1}/{len(dataset['permissions'])} permissions", file=sys.stderr)
    print(f"   Generated {len(dataset['permissions'])} permission pages", file=sys.stderr)

    # Role pages
    print("   Generating role pages...", file=sys.stderr)
    for i, role in enumerate(dataset['roles']):
        html = generate_role_page(role, perm_to_roles)
        # Use URL-safe filename (remove roles/ prefix)
        role_name = role['name'].replace('roles/', '')
        filename = role_name.replace('/', '_') + '.html'
        filepath = ROLES_DIR / filename
        with open(filepath, 'w') as f:
            f.write(html)
        if (i + 1) % 500 == 0:
            print(f"      {i + 1}/{len(dataset['roles'])} roles", file=sys.stderr)
    print(f"   Generated {len(dataset['roles'])} role pages", file=sys.stderr)

    # Generate sitemap
    print("\n6. Generating sitemap.xml...", file=sys.stderr)
    sitemap = generate_sitemap(dataset['roles'], dataset['permissions'])
    sitemap_path = STATIC_DIR / "sitemap.xml"
    with open(sitemap_path, 'w') as f:
        f.write(sitemap)
    print(f"   Saved to {sitemap_path}", file=sys.stderr)

    # Generate index
    print("\n7. Generating index.html...", file=sys.stderr)
    index_html = generate_index_page(dataset['metadata'])
    index_path = STATIC_DIR / "index.html"
    with open(index_path, 'w') as f:
        f.write(index_html)
    print(f"   Saved to {index_path}", file=sys.stderr)

    print("\n" + "=" * 40, file=sys.stderr)
    print("Done!", file=sys.stderr)
    print(f"\nGenerated files:", file=sys.stderr)
    print(f"  - {json_path}", file=sys.stderr)
    print(f"  - {len(dataset['permissions'])} permission pages in {PERMISSIONS_DIR}", file=sys.stderr)
    print(f"  - {len(dataset['roles'])} role pages in {ROLES_DIR}", file=sys.stderr)
    print(f"  - {sitemap_path}", file=sys.stderr)
    print(f"  - {index_path}", file=sys.stderr)


if __name__ == '__main__':
    main()
