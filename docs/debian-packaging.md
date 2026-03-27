# Debian Packaging and APT Distribution

This document covers the full lifecycle of building `pg_no_seqscan` as a Debian package and distributing it via an S3-backed APT repository.

## Overview

```
Tag push (vX.Y.Z)
      │
      ▼
GitHub Actions builds one .deb per PostgreSQL version (14–18)
      │
      ▼
.deb files are attached to the GitHub Release as assets
      │
      ▼
[Optional] deb-s3 uploads them to the S3-backed APT repository
      │
      ▼
Client machines run: apt-get install postgresql-16-pg-no-seqscan
```

The packages follow Debian's PostgreSQL naming convention:
`postgresql-{pgversion}-pg-no-seqscan_{version}_amd64.deb`

---

## 1. Publishing a new release

### 1.1 Create and push a version tag

The release workflow triggers on tags matching `vX.Y.Z`. Before tagging:

1. Bump `version` in `Cargo.toml` to the target version (e.g. `0.2.0`)
2. Update `CHANGELOG.md`
3. Commit, merge to `main`, then tag:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The GitHub Actions workflow (`.github/workflows/release.yml`) will:
- Create a GitHub Release named `v0.2.0` with auto-generated notes
- Build a `.deb` for each supported PostgreSQL version (14–18)
- Attach all five `.deb` files to the release as assets

### 1.2 Download the .deb files

Once the workflow completes, assets are available on the GitHub Releases page or via:

```bash
gh release download v0.2.0 --pattern "*.deb"
```

---

## 2. Setting up the S3-backed APT repository

This is a one-time setup. The APT repository is a specific directory structure on S3 with index files that `apt` understands. A raw bucket of `.deb` files is **not** enough — the index files are mandatory.

### 2.1 Prerequisites

```bash
gem install deb-s3
# or
pip install deb-s3  # not official, prefer the gem
```

AWS credentials with read/write access to the S3 bucket must be configured (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, or an IAM role).

### 2.2 Generate a GPG key for signing

APT clients refuse to use unsigned repositories. Generate a dedicated signing key:

```bash
gpg --batch --gen-key <<EOF
Key-Type: RSA
Key-Length: 4096
Name-Real: pg-no-seqscan APT Repository
Name-Email: infra@doctolib.com
Expire-Date: 0
%no-protection
EOF

# Note the key fingerprint
gpg --list-keys infra@doctolib.com
```

Export the public key to S3 (clients will need this):

```bash
gpg --armor --export infra@doctolib.com > pgp-key.asc
aws s3 cp pgp-key.asc s3://doctolib-apt-repo/pgp-key.asc --acl public-read
```

Store the private key securely (e.g. AWS Secrets Manager or a secrets manager your CI has access to).

### 2.3 S3 bucket configuration

The bucket must be either:
- **Publicly readable** (simplest), or
- Accessible via VPC endpoint / CloudFront with signed URLs (more secure)

For a simple internal setup, enabling "Block Public Access" off for the bucket and making objects public works. For production, use an S3 bucket policy that allows read from your VPC CIDR.

---

## 3. Publishing .deb files to the APT repository

Run this for each `.deb` file after a release. In practice this is automated in CI.

```bash
# Download release assets
gh release download v0.2.0 --pattern "*.deb"

# Upload each .deb to the APT repository
for deb in *.deb; do
  deb-s3 upload \
    --bucket doctolib-apt-repo \
    --codename stable \
    --component main \
    --arch amd64 \
    --sign infra@doctolib.com \
    "$deb"
done
```

`deb-s3 upload` will:
1. Upload the `.deb` to `pool/main/`
2. Download the existing `Packages` index (if any)
3. Add the new package's metadata (checksums, dependencies, description)
4. Re-upload `Packages`, `Packages.gz`, `Release`, `InRelease`, `Release.gpg`

The resulting S3 structure looks like:

```
s3://doctolib-apt-repo/
├── pgp-key.asc
├── dists/
│   └── stable/
│       ├── Release
│       ├── InRelease          ← verified by apt-get update
│       ├── Release.gpg
│       └── main/
│           └── binary-amd64/
│               ├── Packages
│               └── Packages.gz
└── pool/
    └── main/
        ├── postgresql-14-pg-no-seqscan_0.2.0_amd64.deb
        ├── postgresql-15-pg-no-seqscan_0.2.0_amd64.deb
        ├── postgresql-16-pg-no-seqscan_0.2.0_amd64.deb
        ├── postgresql-17-pg-no-seqscan_0.2.0_amd64.deb
        └── postgresql-18-pg-no-seqscan_0.2.0_amd64.deb
```

### Automating publication in CI

Add a job to the release workflow after all `.deb` files are uploaded to GitHub:

```yaml
publish-to-apt:
  name: 📤 Publish to APT repository
  needs: build-deb
  runs-on: ubuntu-latest
  environment: production  # gate behind a manual approval if needed
  steps:
    - name: Install deb-s3
      run: gem install deb-s3

    - name: Download release assets
      env:
        GH_TOKEN: ${{ github.token }}
      run: gh release download "${{ github.ref_name }}" --pattern "*.deb"

    - name: Import GPG signing key
      run: echo "${{ secrets.APT_GPG_PRIVATE_KEY }}" | gpg --import

    - name: Publish to S3 APT repository
      env:
        AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
        AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
      run: |
        for deb in *.deb; do
          deb-s3 upload \
            --bucket "${{ vars.APT_S3_BUCKET }}" \
            --codename stable \
            --component main \
            --arch amd64 \
            --sign "${{ secrets.APT_GPG_KEY_ID }}" \
            "$deb"
        done
```

Required GitHub secrets/vars:
| Name | Type | Value |
|------|------|-------|
| `APT_GPG_PRIVATE_KEY` | Secret | Armored private key (`gpg --armor --export-secret-keys`) |
| `APT_GPG_KEY_ID` | Secret | Email or fingerprint of the signing key |
| `AWS_ACCESS_KEY_ID` | Secret | AWS credentials with S3 write access |
| `AWS_SECRET_ACCESS_KEY` | Secret | AWS credentials |
| `APT_S3_BUCKET` | Variable | S3 bucket name |

---

## 4. Client setup

On any machine that needs to install `pg_no_seqscan`:

### 4.1 Trust the repository's GPG key (one-time)

```bash
curl -fsSL https://doctolib-apt-repo.s3.eu-west-1.amazonaws.com/pgp-key.asc \
  | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/pg-no-seqscan.gpg
```

### 4.2 Add the APT source (one-time)

```bash
echo "deb https://doctolib-apt-repo.s3.eu-west-1.amazonaws.com stable main" \
  | sudo tee /etc/apt/sources.list.d/pg-no-seqscan.list
```

### 4.3 Install

```bash
sudo apt-get update
sudo apt-get install postgresql-16-pg-no-seqscan
```

Replace `16` with your PostgreSQL version.

### 4.4 Configure PostgreSQL

Add to `postgresql.conf` (typically `/etc/postgresql/16/main/postgresql.conf`):

```
shared_preload_libraries = 'pg_no_seqscan'
```

Restart PostgreSQL and create the extension:

```sql
CREATE EXTENSION pg_no_seqscan;
```

---

## 5. Installing a specific version

By default `apt-get install` picks the latest. To pin a version:

```bash
apt-get install postgresql-16-pg-no-seqscan=0.1.0
```

To list available versions:

```bash
apt-cache policy postgresql-16-pg-no-seqscan
```

---

## 6. What each .deb installs

| File | Destination |
|------|-------------|
| `pg_no_seqscan.so` | `$(pg_config --pkglibdir)/` |
| `pg_no_seqscan.control` | `$(pg_config --sharedir)/extension/` |
| `pg_no_seqscan--{version}.sql` | `$(pg_config --sharedir)/extension/` |

These paths resolve to something like `/usr/lib/postgresql/16/lib/` and `/usr/share/postgresql/16/extension/` on a standard Debian/Ubuntu PostgreSQL install.
