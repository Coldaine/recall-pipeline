-- Hierarchy tables for summarization drill-down
-- Frame → Activity → Project → Day

-- Activities table
CREATE TABLE IF NOT EXISTS activities (
    id VARCHAR PRIMARY KEY,
    deployment_id VARCHAR NOT NULL,
    start_at TIMESTAMPTZ,
    end_at TIMESTAMPTZ,
    summary TEXT,
    project_id VARCHAR
);

CREATE INDEX IF NOT EXISTS idx_activities_deployment ON activities(deployment_id);
CREATE INDEX IF NOT EXISTS idx_activities_project ON activities(project_id);
CREATE INDEX IF NOT EXISTS idx_activities_time ON activities(start_at, end_at);

-- Projects table
CREATE TABLE IF NOT EXISTS projects (
    id VARCHAR PRIMARY KEY,
    name VARCHAR,
    summary TEXT
);

CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);

-- Day summaries table
CREATE TABLE IF NOT EXISTS day_summaries (
    id VARCHAR PRIMARY KEY,
    deployment_id VARCHAR,  -- NULL for cross-deployment
    date DATE NOT NULL,
    summary TEXT
);

CREATE INDEX IF NOT EXISTS idx_day_summaries_deployment ON day_summaries(deployment_id);
CREATE INDEX IF NOT EXISTS idx_day_summaries_date ON day_summaries(date);
CREATE UNIQUE INDEX IF NOT EXISTS idx_day_summaries_unique ON day_summaries(deployment_id, date);
