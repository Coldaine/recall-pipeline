---
doc_type: guide
subsystem: storage
version: 1.0.0
status: active
owners: Patrick
last_reviewed: 2025-11-27
---
# PostgreSQL Extension Installation Guide

Quick reference for installing TimescaleDB and pgvector on Debian 12 with PostgreSQL 15.

## Prerequisites

- Debian 12 (Bookworm)
- PostgreSQL 15
- SSH access to server
- Sudo privileges

## Installation Steps

### 1. Install TimescaleDB

```bash
# Add TimescaleDB repository
curl -s https://packagecloud.io/install/repositories/timescale/timescaledb/script.deb.sh | sudo bash

# Update package list
sudo apt-get update

# Install TimescaleDB for PostgreSQL 15 (open-source version)
sudo apt-get install timescaledb-2-oss-postgresql-15

# Run tuning script (optional but recommended)
sudo timescaledb-tune --pg-config=/usr/bin/pg_config

# Restart PostgreSQL
sudo systemctl restart postgresql
# OR for Docker:
docker restart postgres-recall
```

### 2. Install pgvector

```bash
# Install pgvector for PostgreSQL 15
sudo apt-get install postgresql-15-pgvector

# If not available via apt, build from source:
sudo apt-get install build-essential postgresql-server-dev-15 git
cd /tmp
git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git
cd pgvector
make
sudo make install
```

### 3. Enable Extensions in Database

```bash
# Connect to database
docker exec -it postgres-recall psql -U recall -d recall

# Enable extensions
CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS vector;

# Verify installation
\dx

# Test vector operations
SELECT '[1,2,3]'::vector <-> '[4,5,6]'::vector AS distance;

# Exit
\q
```

### 4. Apply Schema Upgrade

```bash
# Copy upgrade script to server
scp agents/database/upgrade_schema_extensions.sql coldaine@192.168.1.49:/tmp/

# SSH to server
ssh coldaine@192.168.1.49

# Apply upgrade script
docker exec -i postgres-recall psql -U recall -d recall < /tmp/upgrade_schema_extensions.sql
```

### 5. Verify Upgrade

```bash
# Connect to database
docker exec -it postgres-recall psql -U recall -d recall

# Check extensions
SELECT * FROM pg_extension WHERE extname IN ('timescaledb', 'vector');

# Check hypertables
SELECT * FROM timescaledb_information.hypertables;

# Check tables
\dt

# Check frames table structure
\d frames

# Exit
\q
```

## Troubleshooting

### TimescaleDB not found
```bash
# Check if package is available
apt-cache search timescaledb | grep postgresql-15

# Check PostgreSQL version
psql --version
```

### pgvector not found
```bash
# Check if package is available
apt-cache search pgvector | grep postgresql-15

# If not available, build from source (see step 2)
```

### Extension creation fails
```bash
# Check PostgreSQL logs
docker logs postgres-recall

# Check extension files exist
ls -la /usr/lib/postgresql/15/lib/ | grep -E 'timescale|vector'
```

## References

- [TimescaleDB Installation](https://docs.tigerdata.com/self-hosted/latest/install/installation-linux/)
- [pgvector GitHub](https://github.com/pgvector/pgvector)
- [TimescaleDB Package Repository](https://packagecloud.io/timescale/timescaledb)
